use actix_web::{web, HttpRequest};
use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::{ClientId, ClientSecret, IssuerUrl};

use crate::api::auth::{PATH_CALLBACK, SCOPE_ACCOUNT};
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

pub fn get_callback_url(req: &HttpRequest) -> String {
    let settings = req
        .app_data::<web::Data<crate::settings::Settings>>()
        .unwrap();
    if let Some(ref overridden) = settings.auth.google_redirect_uri {
        return overridden.clone();
    }
    let conn = req.connection_info();
    format!(
        "{}://{}{}{}",
        conn.scheme(),
        conn.host(),
        SCOPE_ACCOUNT,
        PATH_CALLBACK
    )
}
