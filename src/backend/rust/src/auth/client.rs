use actix_web::{web, HttpRequest};
use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::{ClientId, ClientSecret, IssuerUrl};

use crate::api::auth::{PATH_CALLBACK, SCOPE_ACCOUNT};
use crate::error::{AppError, AppResult};
use crate::settings::AuthSettings;

pub struct GoogleOAuthClient {
    pub client: CoreClient,
}

impl GoogleOAuthClient {
    pub async fn new(settings: &AuthSettings) -> anyhow::Result<Self> {
        let client_id = &settings.google_client_id;
        let client_secret = &settings.google_client_secret;
        let issuer_url = IssuerUrl::new("https://accounts.google.com".to_string())?;

        let provider_metadata = CoreProviderMetadata::discover_async(
            issuer_url,
            openidconnect::reqwest::async_http_client,
        )
        .await?;

        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(client_id.clone()),
            Some(ClientSecret::new(client_secret.clone())),
        );

        Ok(Self { client })
    }
}

pub fn get_callback_url(req: &HttpRequest) -> AppResult<String> {
    let settings = req
        .app_data::<web::Data<crate::settings::Settings>>()
        .ok_or_else(|| AppError::Internal("Settings missing from app data".to_string()))?;

    if let Some(ref overridden) = settings.auth.google_redirect_uri {
        let overridden = overridden.trim();
        if !overridden.is_empty() {
            return Ok(overridden.to_string());
        }
    }

    if let Some(ref origin) = settings.server.public_origin {
        let origin = origin.trim_end_matches('/').trim();
        if !origin.is_empty() {
            return Ok(format!("{origin}{SCOPE_ACCOUNT}{PATH_CALLBACK}"));
        }
    }

    Err(AppError::Internal(
        "OAuth callback URL not configured (set GOOGLE_REDIRECT_URI or SERVER__PUBLIC_ORIGIN)"
            .to_string(),
    ))
}
