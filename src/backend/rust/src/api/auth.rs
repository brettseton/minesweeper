use crate::auth::client::{get_callback_url, GoogleOAuthClient};
use crate::auth::{build_state, new_browser_nonce, parse_state};
use crate::error::{AppError, AppResult};
use crate::model::UserInfo;
use actix_identity::Identity;
use actix_session::Session;
use actix_web::cookie::time::Duration;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, ResponseError};
use openidconnect::core::CoreResponseType;
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, CsrfToken, Nonce, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, Scope, TokenResponse,
};
use serde::Deserialize;
use tracing::info;

pub const SCOPE_ACCOUNT: &str = "/account";
pub const PATH_LOGIN: &str = "/google-login";
pub const PATH_CALLBACK: &str = "/callback";
pub const PATH_LOGOUT: &str = "/google-logout";
pub const PATH_STATUS: &str = "/status";

const OAUTH_BINDING_COOKIE_NAME: &str = "oauth-login-nonce";

fn build_oauth_binding_cookie(value: String, secure: bool) -> Cookie<'static> {
    Cookie::build(OAUTH_BINDING_COOKIE_NAME, value)
        .path(SCOPE_ACCOUNT)
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(10 * 60))
        .finish()
}

fn clear_oauth_binding_cookie(secure: bool) -> Cookie<'static> {
    Cookie::build(OAUTH_BINDING_COOKIE_NAME, "")
        .path(SCOPE_ACCOUNT)
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(0))
        .finish()
}

pub async fn google_login(
    google_client: web::Data<GoogleOAuthClient>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let nonce = Nonce::new_random();

    let callback_url = get_callback_url(&req)?;
    let client = google_client.client.clone().set_redirect_uri(
        RedirectUrl::new(callback_url).map_err(|e| AppError::Internal(e.to_string()))?,
    );

    let settings = req
        .app_data::<web::Data<crate::settings::Settings>>()
        .ok_or_else(|| AppError::Internal("Settings missing from app data".to_string()))?;

    // Bind the OAuth state to the browser to prevent login CSRF.
    // SameSite=Lax allows the cookie to be sent on the top-level Google redirect back to us.
    let browser_nonce = new_browser_nonce();
    let state = build_state(
        settings.as_ref(),
        nonce.secret(),
        pkce_verifier.secret(),
        &browser_nonce,
    )?;

    let (auth_url, _csrf_token, _nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            {
                let state = state.clone();
                move || CsrfToken::new(state.clone())
            },
            {
                let nonce = nonce.clone();
                move || nonce.clone()
            },
        )
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    let cookie = build_oauth_binding_cookie(browser_nonce, settings.server.secure_cookies);
    let mut resp = HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish();
    resp.add_cookie(&cookie)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(resp)
}

#[derive(Deserialize)]
pub struct AuthCallbackParams {
    code: String,
    state: String,
}

pub async fn google_callback(
    google_client: web::Data<GoogleOAuthClient>,
    params: web::Query<AuthCallbackParams>,
    session: Session,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let settings = req
        .app_data::<web::Data<crate::settings::Settings>>()
        .ok_or_else(|| AppError::Internal("Settings missing from app data".to_string()))?;
    let (nonce, pkce_verifier_str, browser_nonce_state) =
        parse_state(settings.as_ref(), &params.state)?;

    // Enforce state binding cookie to prevent login CSRF.
    let browser_nonce_cookie = req
        .cookie(OAUTH_BINDING_COOKIE_NAME)
        .map(|c| c.value().to_string());
    if browser_nonce_cookie.as_deref() != Some(browser_nonce_state.as_str()) {
        let mut resp = AppError::Forbidden.error_response();
        resp.add_cookie(&clear_oauth_binding_cookie(settings.server.secure_cookies))
            .map_err(|e| AppError::Internal(e.to_string()))?;
        return Ok(resp);
    }

    let callback_url = get_callback_url(&req)?;
    let client = google_client.client.clone().set_redirect_uri(
        RedirectUrl::new(callback_url).map_err(|e| AppError::Internal(e.to_string()))?,
    );

    let token_response = client
        .exchange_code(AuthorizationCode::new(params.code.clone()))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier_str))
        .request_async(openidconnect::reqwest::async_http_client)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to exchange code: {:?}", e)))?;

    let id_token = token_response
        .id_token()
        .ok_or_else(|| AppError::Internal("No ID token found".to_string()))?;

    let claims = id_token
        .claims(&client.id_token_verifier(), &Nonce::new(nonce))
        .map_err(|e| AppError::Internal(format!("Failed to verify ID token: {:?}", e)))?;

    let user_info = UserInfo {
        sub: claims.subject().to_string(),
        name: claims
            .name()
            .and_then(|n| n.get(None).map(|v| v.to_string())),
        email: claims.email().map(|e| e.to_string()),
    };

    let user_json = serde_json::to_string(&user_info)
        .map_err(|e| AppError::Internal(format!("Failed to serialize user info: {}", e)))?;

    session.renew();

    Identity::login(&req.extensions(), user_json)
        .map_err(|e| AppError::Internal(format!("Failed to login identity: {}", e)))?;

    info!("Successfully logged in user: {}", user_info.sub);

    let mut resp = HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish();
    resp.add_cookie(&clear_oauth_binding_cookie(settings.server.secure_cookies))
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(resp)
}

pub async fn google_logout(identity: Option<Identity>) -> HttpResponse {
    if let Some(id) = identity {
        id.logout();
    }
    HttpResponse::Ok().json(serde_json::json!({ "message": "Logged out" }))
}

pub async fn status(identity: Option<Identity>) -> HttpResponse {
    if let Some(id) = identity {
        if let Ok(user_json) = id.id() {
            if let Ok(user_info) = serde_json::from_str::<UserInfo>(&user_json) {
                return HttpResponse::Ok().json(serde_json::json!({
                    "isAuthenticated": true,
                    "name": user_info.name.unwrap_or_else(|| "User".to_string())
                }));
            }
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "isAuthenticated": false,
        "name": null
    }))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(SCOPE_ACCOUNT)
            .route(PATH_LOGIN, web::get().to(google_login))
            .route(PATH_CALLBACK, web::get().to(google_callback))
            .route(PATH_LOGOUT, web::post().to(google_logout))
            .route(PATH_STATUS, web::get().to(status)),
    );
}
