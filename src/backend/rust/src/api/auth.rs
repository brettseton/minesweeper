use crate::auth::client::{get_callback_url, GoogleOAuthClient};
use crate::error::{AppError, AppResult};
use crate::model::UserInfo;
use actix_identity::Identity;
use actix_session::Session;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use openidconnect::core::CoreResponseType;
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, CsrfToken, Nonce, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, Scope, TokenResponse,
};
use serde::Deserialize;
use tracing::{info, warn};

pub const SCOPE_ACCOUNT: &str = "/account";
pub const PATH_LOGIN: &str = "/google-login";
pub const PATH_CALLBACK: &str = "/callback";
pub const PATH_LOGOUT: &str = "/google-logout";
pub const PATH_STATUS: &str = "/status";

pub async fn google_login(
    google_client: web::Data<GoogleOAuthClient>,
    session: Session,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let callback_url = get_callback_url(&req);
    let client = google_client.client.clone().set_redirect_uri(
        RedirectUrl::new(callback_url).map_err(|e| AppError::Internal(e.to_string()))?,
    );

    let (auth_url, csrf_token, nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    session
        .insert("csrf_token", csrf_token.secret().to_string())
        .map_err(|e| AppError::Internal(format!("Failed to insert csrf_token: {}", e)))?;
    session
        .insert("nonce", nonce.secret().to_string())
        .map_err(|e| AppError::Internal(format!("Failed to insert nonce: {}", e)))?;
    session
        .insert("pkce_verifier", pkce_verifier.secret().to_string())
        .map_err(|e| AppError::Internal(format!("Failed to insert pkce_verifier: {}", e)))?;

    Ok(HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish())
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
    let csrf_token: String = session
        .get("csrf_token")
        .map_err(|e| AppError::BadRequest(format!("Session error: {}", e)))?
        .ok_or_else(|| {
            warn!("Missing CSRF token in session. Ensure you are using the correct hostname and not the IP address.");
            AppError::BadRequest("Missing CSRF token".to_string())
        })?;

    let nonce: String = session
        .get("nonce")
        .map_err(|e| AppError::BadRequest(format!("Session error: {}", e)))?
        .ok_or_else(|| AppError::BadRequest("Missing nonce".to_string()))?;

    let pkce_verifier_str: String = session
        .get("pkce_verifier")
        .map_err(|e| AppError::BadRequest(format!("Session error: {}", e)))?
        .ok_or_else(|| AppError::BadRequest("Missing pkce_verifier".to_string()))?;

    session.remove("csrf_token");
    session.remove("nonce");
    session.remove("pkce_verifier");

    if csrf_token != params.state {
        warn!(
            "CSRF token mismatch: expected {}, got {}",
            csrf_token, params.state
        );
        return Err(AppError::BadRequest("CSRF token mismatch".to_string()));
    }

    let callback_url = get_callback_url(&req);
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

    Ok(HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish())
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
