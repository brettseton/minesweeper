pub mod client;
pub mod identity;

use actix_web::cookie::Key;
use actix_web::web;
use tracing::warn;

use crate::settings::Settings;

pub use client::GoogleOAuthClient;
pub use identity::IdentityExt;

pub async fn init_google_client(settings: &Settings) -> Option<web::Data<GoogleOAuthClient>> {
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
