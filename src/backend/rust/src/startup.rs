use crate::api;
use crate::auth::GoogleOAuthClient;
use crate::service::GameService;
use crate::settings::Settings;
use actix_cors::Cors;
pub use actix_identity::IdentityMiddleware;
pub use actix_session::{
    config::PersistentSession, storage::CookieSessionStore, SessionMiddleware,
};
use actix_web::{cookie::Key, middleware, web, App, HttpServer};
use actix_web_opentelemetry::RequestTracing;
use std::sync::Arc;

pub struct Application {
    server: actix_web::dev::Server,
}

pub fn configure_app(
    cfg: &mut web::ServiceConfig,
    repo_data: web::Data<Arc<dyn crate::repository::MinesweeperRepository>>,
    service_data: web::Data<Arc<dyn GameService>>,
    google_client: Option<web::Data<GoogleOAuthClient>>,
    settings_data: web::Data<Settings>,
) {
    cfg.app_data(repo_data)
        .app_data(service_data)
        .app_data(settings_data)
        .configure(|c| {
            if let Some(ref client) = google_client {
                c.app_data(client.clone());
            }
        })
        .configure(api::config_auth)
        .configure(api::config_game)
        .configure(api::config_user);
}

pub fn build_session_middleware(
    secret_key: Key,
    secure: bool,
) -> SessionMiddleware<CookieSessionStore> {
    SessionMiddleware::builder(CookieSessionStore::default(), secret_key)
        .cookie_name("minesweeper-session".to_string())
        .cookie_secure(secure)
        .cookie_same_site(actix_web::cookie::SameSite::Lax)
        .cookie_http_only(true)
        .cookie_path("/".to_string())
        .session_lifecycle(PersistentSession::default())
        .build()
}

pub fn build_cors_middleware(allowed_origins: &[String]) -> Cors {
    let mut cors = Cors::default()
        .allow_any_method()
        .allow_any_header()
        .supports_credentials()
        .max_age(3600);

    if allowed_origins.is_empty() {
        cors = cors.allow_any_origin();
    } else {
        for origin in allowed_origins {
            cors = cors.allowed_origin(origin);
        }
    }

    cors
}

impl Application {
    pub async fn build(
        repo_data: web::Data<Arc<dyn crate::repository::MinesweeperRepository>>,
        service_data: web::Data<Arc<dyn GameService>>,
        google_client: Option<web::Data<GoogleOAuthClient>>,
        settings: Settings,
        session_key: Key,
    ) -> std::io::Result<Self> {
        let address = format!("0.0.0.0:{}", settings.server.port);
        let settings_data = web::Data::new(settings.clone());

        let server = HttpServer::new(move || {
            App::new()
                .wrap(RequestTracing::new())
                .wrap(IdentityMiddleware::default())
                .wrap(build_session_middleware(
                    session_key.clone(),
                    settings.server.secure_cookies,
                ))
                .wrap(build_cors_middleware(&settings.server.allowed_origins))
                .wrap(middleware::Logger::default())
                .configure(|c| {
                    configure_app(
                        c,
                        repo_data.clone(),
                        service_data.clone(),
                        google_client.clone(),
                        settings_data.clone(),
                    )
                })
        })
        .bind(address)?
        .run();

        Ok(Self { server })
    }

    pub async fn run_until_stopped(self) -> std::io::Result<()> {
        self.server.await
    }
}
