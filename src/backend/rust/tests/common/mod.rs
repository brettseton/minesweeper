use actix_web::dev::{Service, ServiceRequest, ServiceResponse};
use actix_web::{cookie::Key, test, web, App, HttpMessage};
use once_cell::sync::Lazy;
use rust_backend::api;
use rust_backend::repository::MinesweeperRepository;
use rust_backend::service::{GameService, MinesweeperService};
use rust_backend::startup::{build_session_middleware, configure_app, IdentityMiddleware};
use rust_backend::telemetry::metrics::MinesweeperMetrics;
use std::sync::Arc;

static INIT: Lazy<()> = Lazy::new(|| {
    MinesweeperMetrics::init();
});

pub const X_MOCK_AUTH: &str = "X-Mock-Auth";

pub fn uri_user_games() -> String {
    format!("{}{}", api::SCOPE_USER, api::PATH_GAMES)
}

pub fn uri_new_game(cols: usize, rows: usize, mines: usize) -> String {
    format!("{}{}", api::SCOPE_GAME, api::PATH_NEW_CUSTOM)
        .replace("{cols}", &cols.to_string())
        .replace("{rows}", &rows.to_string())
        .replace("{mines}", &mines.to_string())
}

pub fn uri_game(id: i32) -> String {
    format!("{}{}", api::SCOPE_GAME, api::PATH_ID).replace("{id}", &id.to_string())
}

pub fn uri_flag(id: i32) -> String {
    format!("{}{}", api::SCOPE_GAME, api::PATH_FLAG_ID).replace("{id}", &id.to_string())
}

/// Middleware that injects a mock identity if the X-Mock-Auth header is present.
pub fn mock_auth_middleware<S, B>(
    req: ServiceRequest,
    srv: &S,
) -> impl std::future::Future<Output = Result<ServiceResponse<B>, actix_web::Error>>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
{
    if req.headers().contains_key(X_MOCK_AUTH) {
        let sub = req
            .headers()
            .get("X-User-Sub")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("dev-user");

        let user_json = format!(
            r#"{{"sub":"{}","name":"Dev User","email":"dev@example.com"}}"#,
            sub
        );
        let _ = actix_identity::Identity::login(&req.extensions(), user_json);
    }
    srv.call(req)
}

use rust_backend::engine::MinesweeperEngine;
use rust_backend::settings::Settings;

pub async fn create_test_app(
    repo: Arc<dyn MinesweeperRepository>,
) -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    Lazy::force(&INIT);
    let settings = Settings::new().unwrap_or_else(|_| {
        // Fallback for tests if env vars aren't set
        Settings {
            server: rust_backend::settings::ServerSettings {
                port: 8080,
                secure_cookies: false,
                allowed_origins: vec![],
                session_secret_key: "a".repeat(64),
            },
            database: rust_backend::settings::DatabaseSettings {
                addr: None,
                name: "TestDB".to_string(),
            },
            auth: rust_backend::settings::AuthSettings {
                google_client_id: "id".to_string(),
                google_client_secret: "secret".to_string(),
                google_redirect_uri: None,
            },
            telemetry: rust_backend::settings::TelemetrySettings {
                otlp_endpoint: "http://localhost:4317".to_string(),
            },
        }
    });

    let repo_data = web::Data::new(repo.clone());
    let engine = Arc::new(MinesweeperEngine);
    let service: Arc<dyn GameService> = Arc::new(MinesweeperService::new(repo, engine));
    let service_data = web::Data::new(service);
    let settings_data = web::Data::new(settings.clone());
    let secret_key = Key::generate();

    test::init_service(
        App::new()
            .wrap_fn(mock_auth_middleware)
            .wrap(IdentityMiddleware::default())
            .wrap(build_session_middleware(secret_key.clone(), false))
            .configure(|c| configure_app(c, repo_data, service_data, None, settings_data)),
    )
    .await
}

#[derive(Default)]
pub struct MongoImage;

impl testcontainers::Image for MongoImage {
    type Args = Vec<String>;

    fn name(&self) -> String {
        "mongo".to_string()
    }

    fn tag(&self) -> String {
        "4".to_string()
    }

    fn ready_conditions(&self) -> Vec<testcontainers::core::WaitFor> {
        vec![testcontainers::core::WaitFor::message_on_stdout(
            "Waiting for connections",
        )]
    }

    fn expose_ports(&self) -> Vec<u16> {
        vec![27017]
    }
}
