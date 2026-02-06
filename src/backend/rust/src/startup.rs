use crate::api;
use crate::auth::GoogleOAuthClient;
use crate::middleware::xsrf::XsrfMiddleware;
use crate::service::GameService;
use crate::settings::Settings;
use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
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
        .app_data(settings_data.clone())
        .configure(|c| {
            if let Some(ref client) = google_client {
                c.app_data(client.clone());
            }
        });

    let scope = if google_client.is_some() {
        web::scope("")
            .configure(api::config_auth)
            .configure(api::config_game)
            .configure(api::config_user)
    } else {
        web::scope("")
            .configure(api::config_game)
            .configure(api::config_user)
    };

    if settings_data.server.rate_limit_period_ms > 0 {
        let rate_limit_config = match GovernorConfigBuilder::default()
            .period(std::time::Duration::from_millis(
                settings_data.server.rate_limit_period_ms,
            ))
            .burst_size(settings_data.server.rate_limit_burst_size)
            .finish()
        {
            Some(cfg) => cfg,
            None => {
                tracing::warn!("Invalid rate limit configuration; disabling rate limiting");
                cfg.service(scope);
                return;
            }
        };
        cfg.service(scope.wrap(Governor::new(&rate_limit_config)));
    } else {
        cfg.service(scope);
    }
}

pub fn build_session_middleware(
    secret_key: Key,
    secure: bool,
    same_site: actix_web::cookie::SameSite,
) -> SessionMiddleware<CookieSessionStore> {
    SessionMiddleware::builder(CookieSessionStore::default(), secret_key)
        .cookie_name("minesweeper-session".to_string())
        .cookie_secure(secure)
        .cookie_same_site(same_site)
        .cookie_http_only(true)
        .cookie_path("/".to_string())
        .session_lifecycle(PersistentSession::default())
        .build()
}

pub fn build_cors_middleware(allowed_origins: &[String], supports_credentials: bool) -> Cors {
    let mut cors = Cors::default()
        .allow_any_method()
        .allow_any_header()
        .max_age(3600);

    if supports_credentials {
        cors = cors.supports_credentials();
    }

    for origin in allowed_origins {
        cors = cors.allowed_origin(origin);
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
            let same_site = if settings.server.cross_origin_cookies {
                actix_web::cookie::SameSite::None
            } else {
                actix_web::cookie::SameSite::Strict
            };

            App::new()
                .app_data(web::JsonConfig::default().limit(16384)) // 16KB limit
                .wrap(RequestTracing::new())
                .wrap(XsrfMiddleware::new(
                    settings.server.secure_cookies,
                    same_site,
                    settings.server.public_origin.clone(),
                    settings.server.allowed_origins.clone(),
                    settings.server.trusted_proxies.clone(),
                ))
                .wrap(IdentityMiddleware::default())
                .wrap(build_session_middleware(
                    session_key.clone(),
                    settings.server.secure_cookies,
                    same_site,
                ))
                .wrap(build_cors_middleware(
                    &settings.server.allowed_origins,
                    settings.server.cors_supports_credentials,
                ))
                .wrap(
                    middleware::DefaultHeaders::new()
                        .add((
                            "Strict-Transport-Security",
                            "max-age=31536000; includeSubDomains",
                        ))
                        .add(("X-Content-Type-Options", "nosniff"))
                        .add(("X-Frame-Options", "DENY"))
                        .add(("Content-Security-Policy", "default-src 'self'")),
                )
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
