use actix_web::body::BoxBody;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, ResponseError};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use futures_util::future::{ready, LocalBoxFuture, Ready};
use rand::RngCore;
use std::net::{IpAddr, SocketAddr};

use crate::error::AppError;

pub const XSRF_COOKIE_NAME: &str = "XSRF-TOKEN";
pub const XSRF_HEADER_NAME: &str = "X-XSRF-TOKEN";

#[derive(Clone)]
pub struct XsrfMiddleware {
    secure_cookie: bool,
    cookie_same_site: SameSite,
    public_origin: Option<String>,
    allowed_origins: Vec<String>,
    trusted_proxies: Vec<TrustedProxy>,
}

impl XsrfMiddleware {
    pub fn new(
        secure_cookie: bool,
        cookie_same_site: SameSite,
        public_origin: Option<String>,
        allowed_origins: Vec<String>,
        trusted_proxies: Vec<String>,
    ) -> Self {
        let trusted_proxies = trusted_proxies
            .into_iter()
            .filter_map(|s| match TrustedProxy::parse(&s) {
                Ok(p) => Some(p),
                Err(e) => {
                    tracing::warn!("Ignoring invalid trusted proxy entry '{s}': {e}");
                    None
                }
            })
            .collect();

        Self {
            secure_cookie,
            cookie_same_site,
            public_origin: public_origin.map(|s| s.trim().trim_end_matches('/').to_string()),
            allowed_origins: allowed_origins
                .into_iter()
                .filter_map(|s| normalize_origin(&s))
                .collect(),
            trusted_proxies,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for XsrfMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = XsrfMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(XsrfMiddlewareService {
            service,
            secure_cookie: self.secure_cookie,
            cookie_same_site: self.cookie_same_site,
            public_origin: self.public_origin.clone(),
            allowed_origins: self.allowed_origins.clone(),
            trusted_proxies: self.trusted_proxies.clone(),
        }))
    }
}

pub struct XsrfMiddlewareService<S> {
    service: S,
    secure_cookie: bool,
    cookie_same_site: SameSite,
    public_origin: Option<String>,
    allowed_origins: Vec<String>,
    trusted_proxies: Vec<TrustedProxy>,
}

impl<S, B> Service<ServiceRequest> for XsrfMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let method = req.method().clone();
        let secure_cookie = self.secure_cookie;
        let cookie_same_site = self.cookie_same_site;
        let public_origin = self.public_origin.clone();
        let allowed_origins = self.allowed_origins.clone();
        let trusted_proxies = self.trusted_proxies.clone();

        let existing_token = req.cookie(XSRF_COOKIE_NAME).map(|c| c.value().to_string());
        let token = existing_token.clone().unwrap_or_else(generate_xsrf_token);
        let should_set_cookie = existing_token.is_none();

        // For unsafe methods, require header token to match cookie token.
        let is_unsafe = matches!(
            method,
            actix_web::http::Method::POST
                | actix_web::http::Method::PUT
                | actix_web::http::Method::PATCH
                | actix_web::http::Method::DELETE
        );
        if is_unsafe {
            let header_token = req
                .headers()
                .get(XSRF_HEADER_NAME)
                .and_then(|h| h.to_str().ok());

            // If the XSRF header is missing (common when Angular XSRF interceptor isn't enabled),
            // fall back to an Origin/Referer same-origin check. This still blocks CSRF because
            // browsers set Origin/Referer to the attacking site on cross-site requests.
            let passes_origin_check = is_same_origin(
                &req,
                public_origin.as_deref(),
                &allowed_origins,
                &trusted_proxies,
                secure_cookie,
            );

            if header_token != Some(token.as_str()) && !passes_origin_check {
                let mut resp = AppError::Forbidden.error_response();
                if should_set_cookie {
                    let cookie = build_xsrf_cookie(&token, secure_cookie, cookie_same_site);
                    let _ = resp.add_cookie(&cookie);
                }
                return Box::pin(async move { Ok(req.into_response(resp.map_into_boxed_body())) });
            }
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let mut res = fut.await?.map_into_boxed_body();

            if should_set_cookie {
                let cookie = build_xsrf_cookie(&token, secure_cookie, cookie_same_site);
                res.response_mut().add_cookie(&cookie)?;
            }

            Ok(res)
        })
    }
}

fn build_xsrf_cookie(token: &str, secure_cookie: bool, same_site: SameSite) -> Cookie<'static> {
    Cookie::build(XSRF_COOKIE_NAME, token.to_string())
        .path("/")
        .http_only(false)
        .same_site(same_site)
        .secure(secure_cookie)
        .finish()
}

fn generate_xsrf_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn is_same_origin(
    req: &ServiceRequest,
    public_origin: Option<&str>,
    allowed_origins: &[String],
    trusted_proxies: &[TrustedProxy],
    secure_cookie: bool,
) -> bool {
    let expected = expected_origin(req, public_origin, trusted_proxies, secure_cookie);

    // Prefer Origin header for XHR/fetch; Referer is a fallback.
    if let Some(origin) = req
        .headers()
        .get("Origin")
        .and_then(|h| h.to_str().ok())
        .and_then(origin_authority)
    {
        return is_allowed_origin(origin.as_str(), expected.as_deref(), allowed_origins);
    }

    if let Some(referer_origin) = req
        .headers()
        .get("Referer")
        .and_then(|h| h.to_str().ok())
        .and_then(origin_authority)
    {
        return is_allowed_origin(referer_origin.as_str(), expected.as_deref(), allowed_origins);
    }

    false
}

fn is_allowed_origin(origin: &str, expected_origin: Option<&str>, allowed_origins: &[String]) -> bool {
    if expected_origin == Some(origin) {
        return true;
    }
    allowed_origins.iter().any(|allowed| allowed == origin)
}

fn expected_origin(
    req: &ServiceRequest,
    public_origin: Option<&str>,
    trusted_proxies: &[TrustedProxy],
    secure_cookie: bool,
) -> Option<String> {
    if let Some(origin) = public_origin {
        let origin = origin.trim().trim_end_matches('/');
        if !origin.is_empty() {
            return Some(origin.to_string());
        }
    }

    let is_trusted = req
        .peer_addr()
        .map(|sa| is_trusted_proxy(sa, trusted_proxies))
        .unwrap_or(false);

    let host = if is_trusted {
        req.headers()
            .get("X-Forwarded-Host")
            .and_then(|h| h.to_str().ok())
            .and_then(first_header_value)
            .or_else(|| {
                req.headers()
                    .get("Host")
                    .and_then(|h| h.to_str().ok())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
            })?
    } else {
        req.headers()
            .get("Host")
            .and_then(|h| h.to_str().ok())
            .map(str::trim)
            .filter(|s| !s.is_empty())?
    };

    let scheme = if secure_cookie {
        "https"
    } else if is_trusted {
        req.headers()
            .get("X-Forwarded-Proto")
            .and_then(|h| h.to_str().ok())
            .and_then(first_header_value)
            .filter(|s| *s == "http" || *s == "https")
            .unwrap_or("http")
    } else {
        "http"
    };

    Some(format!("{scheme}://{host}"))
}

fn origin_authority(origin: &str) -> Option<String> {
    let origin = origin.trim();
    let scheme_idx = origin.find("://")?;
    let after_scheme = scheme_idx + 3;
    let rest = &origin[after_scheme..];
    let host = rest.split('/').next()?.trim();
    if host.is_empty() {
        return None;
    }
    Some(format!("{}://{}", &origin[..scheme_idx], host))
}

fn normalize_origin(origin: &str) -> Option<String> {
    let origin = origin.trim().trim_end_matches('/');
    if origin.is_empty() {
        return None;
    }
    Some(origin.to_string())
}

fn is_trusted_proxy(peer: SocketAddr, trusted: &[TrustedProxy]) -> bool {
    let ip = peer.ip();
    trusted.iter().any(|t| t.contains(ip))
}

fn first_header_value(raw: &str) -> Option<&str> {
    let v = raw.split(',').next()?.trim();
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

#[cfg(test)]
mod tests {
    use super::is_allowed_origin;

    #[test]
    fn allows_expected_origin() {
        assert!(is_allowed_origin(
            "https://api.example.com",
            Some("https://api.example.com"),
            &[]
        ));
    }

    #[test]
    fn allows_configured_cross_origin() {
        assert!(is_allowed_origin(
            "https://app.example.com",
            Some("https://api.example.com"),
            &["https://app.example.com".to_string()]
        ));
    }

    #[test]
    fn rejects_unconfigured_origin() {
        assert!(!is_allowed_origin(
            "https://evil.example.com",
            Some("https://api.example.com"),
            &["https://app.example.com".to_string()]
        ));
    }
}

#[derive(Clone, Debug)]
enum TrustedProxy {
    Ip(IpAddr),
    Cidr { base: IpAddr, prefix: u8 },
}

impl TrustedProxy {
    fn parse(raw: &str) -> Result<Self, String> {
        let raw = raw.trim();
        if raw.is_empty() {
            return Err("empty".into());
        }

        if let Some((ip_str, prefix_str)) = raw.split_once('/') {
            let base: IpAddr = ip_str
                .trim()
                .parse()
                .map_err(|_| "invalid IP in CIDR".to_string())?;
            let prefix: u8 = prefix_str
                .trim()
                .parse()
                .map_err(|_| "invalid prefix length".to_string())?;
            let max = match base {
                IpAddr::V4(_) => 32,
                IpAddr::V6(_) => 128,
            };
            if prefix > max {
                return Err(format!("prefix out of range (max {max})"));
            }
            return Ok(Self::Cidr { base, prefix });
        }

        let ip: IpAddr = raw.parse().map_err(|_| "invalid IP".to_string())?;
        Ok(Self::Ip(ip))
    }

    fn contains(&self, ip: IpAddr) -> bool {
        match *self {
            TrustedProxy::Ip(allowed) => allowed == ip,
            TrustedProxy::Cidr { base, prefix } => match (base, ip) {
                (IpAddr::V4(base), IpAddr::V4(ip)) => {
                    let mask = if prefix == 0 {
                        0
                    } else {
                        u32::MAX << (32 - prefix)
                    };
                    (u32::from(base) & mask) == (u32::from(ip) & mask)
                }
                (IpAddr::V6(base), IpAddr::V6(ip)) => {
                    let mask = if prefix == 0 {
                        0
                    } else {
                        u128::MAX << (128 - prefix)
                    };
                    (u128::from(base) & mask) == (u128::from(ip) & mask)
                }
                _ => false,
            },
        }
    }
}
