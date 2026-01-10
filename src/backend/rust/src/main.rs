use actix_web::web;
use rust_backend::engine::MinesweeperEngine;
use rust_backend::service::MinesweeperService;
use rust_backend::settings::Settings;
use rust_backend::startup::Application;
use rust_backend::{auth, repository, telemetry};
use std::sync::Arc;
use tracing::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let settings = Settings::new().expect("Failed to load settings");
    telemetry::init_telemetry(&settings);

    let repo = repository::init_repository(&settings.database).await;
    let repo_data = web::Data::new(repo.clone());

    let engine = Arc::new(MinesweeperEngine);
    let game_service: Arc<dyn rust_backend::service::GameService> =
        Arc::new(MinesweeperService::new(repo.clone(), engine));
    let service_data = web::Data::new(game_service);

    let google_client = auth::init_google_client(&settings).await;
    let session_key = auth::get_session_key(&settings);

    info!("Starting server at http://0.0.0.0:{}", settings.server.port);

    let app = Application::build(
        repo_data,
        service_data,
        google_client,
        settings,
        session_key,
    )
    .await?;

    app.run_until_stopped().await
}
