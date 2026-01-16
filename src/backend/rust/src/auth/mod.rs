pub mod client;
pub mod identity;
mod state;

use actix_web::cookie::Key;
use actix_web::web;
use tracing::warn;

use crate::settings::Settings;

pub use client::GoogleOAuthClient;
pub use identity::IdentityExt;
pub use state::{build_state, parse_state};

pub async fn init_google_client(settings: &Settings) -> Option<web::Data<GoogleOAuthClient>> {
    if settings.auth.google_client_id.trim().is_empty()
        && settings.auth.google_client_secret.trim().is_empty()
    {
        warn!("Google OAuth disabled: GOOGLE_CLIENT_ID/GOOGLE_CLIENT_SECRET not set");
        return None;
    }

    match GoogleOAuthClient::new(&settings.auth).await {
        Ok(client) => Some(web::Data::new(client)),
        Err(e) => {
            warn!("Failed to initialize Google OAuth client: {}", e);
            None
        }
    }
}

pub fn get_session_key(settings: &Settings) -> Key {
    let key_bytes = settings.server.session_secret_key.as_bytes();
    if key_bytes.len() >= 64 {
        Key::from(key_bytes)
    } else {
        warn!("SESSION_SECRET_KEY is too short; using a random key. Sessions will not persist across restarts.");
        Key::generate()
    }
}
