use actix_identity::IdentityMiddleware;
use actix_web::cookie::Key;
use actix_web::{test, web, App};
use criterion::{criterion_group, criterion_main, Criterion};
use rust_backend::engine::MinesweeperEngine;
use rust_backend::model::{MakeMoveRequest, MinesweeperGameDto};
use rust_backend::repository::{
    InMemoryGameRepository, MinesweeperRepository, MongoGameRepository,
};
use rust_backend::service::{GameService, MinesweeperService};
use rust_backend::settings::{
    AuthSettings, DatabaseSettings, ServerSettings, Settings, TelemetrySettings,
};
use rust_backend::startup::build_session_middleware;
use rust_backend::startup::configure_app;
use rust_backend::telemetry::metrics::MinesweeperMetrics;
use std::sync::Arc;
use testcontainers::clients::Cli;
use testcontainers::core::WaitFor;
use testcontainers::Image;

#[derive(Default)]
pub struct MongoImage;

impl Image for MongoImage {
    type Args = Vec<String>;

    fn name(&self) -> String {
        "mongo".to_string()
    }

    fn tag(&self) -> String {
        "4".to_string()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout("Waiting for connections")]
    }

    fn expose_ports(&self) -> Vec<u16> {
        vec![27017]
    }
}

#[derive(Default)]
pub struct RedisImage;

impl Image for RedisImage {
    type Args = Vec<String>;

    fn name(&self) -> String {
        "redis".to_string()
    }

    fn tag(&self) -> String {
        "7".to_string()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout("Ready to accept connections")]
    }

    fn expose_ports(&self) -> Vec<u16> {
        vec![6379]
    }
}

async fn setup_app(
    repo: Arc<dyn MinesweeperRepository>,
    hot_repo: Option<Arc<rust_backend::repository::RedisGameRepository>>,
) -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    // Initialize metrics once
    MinesweeperMetrics::init();

    let settings = Settings {
        environment: "development".to_string(),
        server: ServerSettings {
            port: 8080,
            secure_cookies: false,
            allowed_origins: vec![],
            session_secret_key: "a".repeat(64),
            rate_limit_period_ms: 1,
            rate_limit_burst_size: 10000000,
        },
        database: DatabaseSettings {
            addr: None,
            name: "TestDB".to_string(),
        },
        redis: rust_backend::settings::RedisSettings { addr: None, ..Default::default() },
        auth: AuthSettings {
            google_client_id: "id".to_string(),
            google_client_secret: "secret".to_string(),
            google_redirect_uri: None,
        },
        telemetry: TelemetrySettings {
            otlp_endpoint: "".to_string(),
        },
    };

    let engine = Arc::new(MinesweeperEngine);
    let service: Arc<dyn GameService> =
        Arc::new(MinesweeperService::new(repo.clone(), hot_repo, engine, settings.redis.clone()));

    let repo_data = web::Data::new(repo);
    let service_data = web::Data::new(service);
    let settings_data = web::Data::new(settings.clone());
    let secret_key = Key::generate();

    test::init_service(
        App::new()
            .wrap(IdentityMiddleware::default())
            .wrap(build_session_middleware(secret_key.clone(), false))
            .configure(|c| configure_app(c, repo_data, service_data, None, settings_data)),
    )
    .await
}

fn api_benchmarks(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    let docker = Cli::default();

    // Setup repositories
    let in_memory_repo = Arc::new(InMemoryGameRepository::new());

    // Setup Mongo
    let mongo_node = docker.run(MongoImage);
    let mongo_port = mongo_node.get_host_port_ipv4(27017);
    let mongo_url = format!("mongodb://localhost:{}", mongo_port);
    let mongo_repo = rt.block_on(async {
        MongoGameRepository::new(&mongo_url, "MinesweeperBenchmark")
            .await
            .expect("Failed to create Mongo repo")
    });
    let mongo_repo: Arc<dyn MinesweeperRepository> = Arc::new(mongo_repo);

    // Setup Redis for Hybrid
    let redis_node = docker.run(RedisImage);
    let redis_port = redis_node.get_host_port_ipv4(6379);
    let redis_url = format!("redis://localhost:{}", redis_port);
    let redis_repo = rt.block_on(async {
        rust_backend::repository::RedisGameRepository::new(&redis_url, 24 * 60 * 60)
            .await
            .expect("Failed to create Redis repo")
    });
    let redis_repo: Arc<rust_backend::repository::RedisGameRepository> = Arc::new(redis_repo);

    type BenchmarkConfig = (
        &'static str,
        Arc<dyn MinesweeperRepository>,
        Option<Arc<rust_backend::repository::RedisGameRepository>>,
    );

    let benchmark_configs: Vec<BenchmarkConfig> = vec![
        ("InMemory", in_memory_repo, None),
        ("Mongo", mongo_repo.clone(), None),
        ("Hybrid (Mongo + Redis)", mongo_repo, Some(redis_repo)),
    ];

    for (repo_type, repo, hot_repo) in benchmark_configs {
        let app = rt.block_on(setup_app(repo.clone(), hot_repo));
        let peer_addr = "127.0.0.1:12345"
            .parse()
            .expect("valid peer addr");

        // Create a game to use for existing game benchmarks
        let req = test::TestRequest::get()
            .uri("/game/new")
            .peer_addr(peer_addr)
            .to_request();
        let initial_game: MinesweeperGameDto = rt.block_on(async {
            let resp = test::call_service(&app, req).await;
            test::read_body_json(resp).await
        });
        let game_id = initial_game.id;

        let mut group = c.benchmark_group(format!("Minesweeper API ({})", repo_type));

        group.bench_function("create_new_game", |b| {
            b.to_async(&rt).iter(|| async {
                let req = test::TestRequest::get()
                    .uri("/game/new")
                    .peer_addr(peer_addr)
                    .to_request();
                let resp = test::call_service(&app, req).await;
                if !resp.status().is_success() {
                    panic!("create_new_game failed with status: {}", resp.status());
                }
                let _resp: MinesweeperGameDto = test::read_body_json(resp).await;
            });
        });

        group.bench_function("get_existing_game", |b| {
            b.to_async(&rt).iter(|| async {
                let req = test::TestRequest::get()
                    .uri(&format!("/game/{}", game_id))
                    .peer_addr(peer_addr)
                    .to_request();
                let resp = test::call_service(&app, req).await;
                if !resp.status().is_success() {
                    panic!("get_existing_game failed with status: {}", resp.status());
                }
                let _resp: MinesweeperGameDto = test::read_body_json(resp).await;
            });
        });

        group.bench_function("make_move", |b| {
            b.to_async(&rt).iter(|| async {
                let req_body = MakeMoveRequest {
                    x: 1,
                    y: 1,
                    game_id: Some(game_id),
                };
                let req = test::TestRequest::post()
                    .uri(&format!("/game/{}", game_id))
                    .peer_addr(peer_addr)
                    .set_json(&req_body)
                    .to_request();
                let resp = test::call_service(&app, req).await;
                if !resp.status().is_success() {
                    panic!("make_move failed with status: {}", resp.status());
                }
                let _resp: MinesweeperGameDto = test::read_body_json(resp).await;
            });
        });

        group.bench_function("toggle_flag", |b| {
            b.to_async(&rt).iter(|| async {
                let req_body = MakeMoveRequest {
                    x: 2,
                    y: 2,
                    game_id: Some(game_id),
                };
                let req = test::TestRequest::post()
                    .uri(&format!("/game/flag/{}", game_id))
                    .peer_addr(peer_addr)
                    .set_json(&req_body)
                    .to_request();
                let resp = test::call_service(&app, req).await;
                if !resp.status().is_success() {
                    panic!("toggle_flag failed with status: {}", resp.status());
                }
                let _resp: MinesweeperGameDto = test::read_body_json(resp).await;
            });
        });

        group.finish();
    }
}

criterion_group!(benches, api_benchmarks);
criterion_main!(benches);
