mod common;

use actix_web::test;
use chrono::Utc;
use common::*;
use once_cell::sync::Lazy;
use rust_backend::model::{BoardState, MakeMoveRequest, MinesweeperGameDto, Point};
use rust_backend::repository::{
    InMemoryGameRepository, MinesweeperRepository, MongoGameRepository,
};
use std::sync::Arc;
use testcontainers::clients::Cli;
use testcontainers::Container;

use rust_backend::engine::{BoardEngine, MinesweeperEngine};

async fn get_point_by_type(
    repo: &Arc<dyn MinesweeperRepository>,
    game_id: i32,
    match_fn: impl Fn(BoardState) -> bool,
) -> Option<Point> {
    let mut game = repo
        .get_game(game_id)
        .await
        .expect("repo.get_game failed")
        .expect("game not found");
    if !game.mines_generated {
        let engine = MinesweeperEngine;
        engine.generate_mines(&mut game, Point { x: 0, y: 0 });
        repo.save(game.clone()).await.expect("repo.save failed");
    }
    for (x, row) in game.board.iter().enumerate() {
        for (y, cell) in row.iter().enumerate() {
            if match_fn(*cell) {
                return Some(Point { x, y });
            }
        }
    }
    None
}

macro_rules! define_api_tests {
    ($setup_fn:ident) => {
        #[actix_web::test]
        async fn create_new_game_sets_created_at() {
            let (app, _repo, _node) = $setup_fn().await;

            let start_time = Utc::now();
            let req = post().uri(&uri_new_game(10, 10, 10)).to_request();
            let resp: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            assert!(resp.created_at >= start_time - chrono::Duration::seconds(10));
            assert!(resp.created_at <= Utc::now() + chrono::Duration::seconds(10));
        }

        #[actix_web::test]
        async fn create_new_game_associates_with_user() {
            let (app, _repo, _node) = $setup_fn().await;

            // Header based auth injection
            let req = post()
                .uri(&uri_new_game(10, 10, 10))
                .insert_header((X_MOCK_AUTH, "true"))
                .to_request();
            let new_game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            let req = test::TestRequest::get()
                .uri(&uri_user_games())
                .insert_header((X_MOCK_AUTH, "true"))
                .to_request();
            let user_games: Vec<MinesweeperGameDto> =
                test::call_and_read_body_json(&app, req).await;

            assert!(user_games.iter().any(|g| g.id == new_game.id));
        }

        #[actix_web::test]
        async fn create_new_game_matches_get_call() {
            let (app, _repo, _node) = $setup_fn().await;

            let new_game: MinesweeperGameDto = test::call_and_read_body_json(
                &app,
                post().uri(&uri_new_game(10, 100, 10)).to_request(),
            )
            .await;
            let game: MinesweeperGameDto = test::call_and_read_body_json(
                &app,
                test::TestRequest::get()
                    .uri(&uri_game(new_game.id))
                    .to_request(),
            )
            .await;

            assert_eq!(new_game.id, game.id);
            assert_eq!(new_game.mine_count, game.mine_count);
            assert_eq!(new_game.board.len(), 10);
            assert_eq!(new_game.board[0].len(), 100);

            for row in &new_game.board {
                for cell in row {
                    assert_eq!(*cell, BoardState::Unknown);
                }
            }
        }

        #[actix_web::test]
        async fn toggle_flag_on_and_off_returns_correct_board_state() {
            let (app, _repo, _node) = $setup_fn().await;

            let new_game: MinesweeperGameDto = test::call_and_read_body_json(
                &app,
                post().uri(&uri_new_game(10, 10, 10)).to_request(),
            )
            .await;
            let req_body = MakeMoveRequest {
                x: 0,
                y: 0,
                game_id: Some(new_game.id),
            };

            let req = post()
                .uri(&uri_flag(new_game.id))
                .set_json(&req_body)
                .to_request();
            let game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            assert_eq!(game.board[0][0], BoardState::Flag);

            let req = post()
                .uri(&uri_flag(new_game.id))
                .set_json(&req_body)
                .to_request();
            let game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            assert_eq!(game.board[0][0], BoardState::Unknown);
        }

        #[actix_web::test]
        async fn click_on_flag_returns_correct_board_state() {
            let (app, repo, _node) = $setup_fn().await;

            let new_game: MinesweeperGameDto = test::call_and_read_body_json(
                &app,
                post().uri(&uri_new_game(10, 10, 5)).to_request(),
            )
            .await;
            let safe_point = get_point_by_type(&repo, new_game.id, |s| {
                s != BoardState::Mine && s != BoardState::Zero
            })
            .await
            .expect("No safe point found");
            let req_body = MakeMoveRequest {
                x: safe_point.x,
                y: safe_point.y,
                game_id: Some(new_game.id),
            };

            let req = post()
                .uri(&uri_flag(new_game.id))
                .set_json(&req_body)
                .to_request();
            let flagged_game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            let req = post()
                .uri(&uri_game(new_game.id))
                .set_json(&req_body)
                .to_request();
            let game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            assert_eq!(flagged_game.board, game.board);
        }

        #[actix_web::test]
        async fn zero_wave_does_not_reveal_flagged_cells() {
            let (app, repo, _node) = $setup_fn().await;

            let new_game: MinesweeperGameDto = test::call_and_read_body_json(
                &app,
                post().uri(&uri_new_game(10, 10, 1)).to_request(),
            )
            .await;

            let zero_point = get_point_by_type(&repo, new_game.id, |s| s == BoardState::Zero)
                .await
                .expect("No zero point found");

            let mut neighbor = None;
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = zero_point.x as isize + dx;
                    let ny = zero_point.y as isize + dy;
                    if (0..10).contains(&nx) && (0..10).contains(&ny) {
                        neighbor = Some(Point {
                            x: nx as usize,
                            y: ny as usize,
                        });
                        break;
                    }
                }
                if neighbor.is_some() {
                    break;
                }
            }
            let neighbor = neighbor.expect("No neighbor found for zero point");

            // Flag a cell that would otherwise be revealed by the zero-wave.
            let req_body = MakeMoveRequest {
                x: neighbor.x,
                y: neighbor.y,
                game_id: Some(new_game.id),
            };
            let req = post()
                .uri(&uri_flag(new_game.id))
                .set_json(&req_body)
                .to_request();
            let flagged_game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;
            assert_eq!(flagged_game.board[neighbor.x][neighbor.y], BoardState::Flag);

            // Click the zero cell; the flagged neighbor should remain flagged.
            let req_body = MakeMoveRequest {
                x: zero_point.x,
                y: zero_point.y,
                game_id: Some(new_game.id),
            };
            let req = post()
                .uri(&uri_game(new_game.id))
                .set_json(&req_body)
                .to_request();
            let game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            assert!(game.flag_points.contains(&neighbor));
            assert_eq!(game.board[neighbor.x][neighbor.y], BoardState::Flag);
        }

        #[actix_web::test]
        async fn move_on_numbered_space_returns_correct_board_state() {
            let (app, repo, _node) = $setup_fn().await;

            let new_game: MinesweeperGameDto = test::call_and_read_body_json(
                &app,
                post().uri(&uri_new_game(10, 10, 5)).to_request(),
            )
            .await;
            let number_point = get_point_by_type(&repo, new_game.id, |s| {
                s != BoardState::Mine && s != BoardState::Zero
            })
            .await
            .expect("No number point found");
            let req_body = MakeMoveRequest {
                x: number_point.x,
                y: number_point.y,
                game_id: Some(new_game.id),
            };

            let req = post()
                .uri(&uri_game(new_game.id))
                .set_json(&req_body)
                .to_request();
            let game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            let cell_state = game.board[number_point.x][number_point.y];
            assert!(matches!(
                cell_state,
                BoardState::One
                    | BoardState::Two
                    | BoardState::Three
                    | BoardState::Four
                    | BoardState::Five
                    | BoardState::Six
                    | BoardState::Seven
                    | BoardState::Eight
            ));
        }

        #[actix_web::test]
        async fn move_on_mine_space_returns_correct_board_state() {
            let (app, repo, _node) = $setup_fn().await;

            let new_game: MinesweeperGameDto = test::call_and_read_body_json(
                &app,
                post().uri(&uri_new_game(10, 10, 5)).to_request(),
            )
            .await;
            let mine_point = get_point_by_type(&repo, new_game.id, |s| s == BoardState::Mine)
                .await
                .expect("No mine found");
            let req_body = MakeMoveRequest {
                x: mine_point.x,
                y: mine_point.y,
                game_id: Some(new_game.id),
            };

            let req = post()
                .uri(&uri_game(new_game.id))
                .set_json(&req_body)
                .to_request();
            let game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            assert_eq!(game.board[mine_point.x][mine_point.y], BoardState::Mine);
        }

        #[actix_web::test]
        async fn move_after_game_over_doesnt_change_board_state() {
            let (app, repo, _node) = $setup_fn().await;

            let new_game: MinesweeperGameDto = test::call_and_read_body_json(
                &app,
                post().uri(&uri_new_game(10, 10, 5)).to_request(),
            )
            .await;
            let mine_point = get_point_by_type(&repo, new_game.id, |s| s == BoardState::Mine)
                .await
                .expect("No mine found");
            let req_body_mine = MakeMoveRequest {
                x: mine_point.x,
                y: mine_point.y,
                game_id: Some(new_game.id),
            };

            let req = post()
                .uri(&uri_game(new_game.id))
                .set_json(&req_body_mine)
                .to_request();
            let game_over: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            let safe_point = get_point_by_type(&repo, new_game.id, |s| s != BoardState::Mine)
                .await
                .expect("No safe point found");
            let req_body_safe = MakeMoveRequest {
                x: safe_point.x,
                y: safe_point.y,
                game_id: Some(new_game.id),
            };

            let req = post()
                .uri(&uri_game(new_game.id))
                .set_json(&req_body_safe)
                .to_request();
            let game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            assert_eq!(game_over.board, game.board);
        }

        #[actix_web::test]
        async fn toggle_flag_on_revealed_space_doesnt_change_board_state() {
            let (app, repo, _node) = $setup_fn().await;

            let new_game: MinesweeperGameDto = test::call_and_read_body_json(
                &app,
                post().uri(&uri_new_game(10, 10, 5)).to_request(),
            )
            .await;
            let number_point = get_point_by_type(&repo, new_game.id, |s| {
                s != BoardState::Mine && s != BoardState::Zero
            })
            .await
            .expect("No number point found");
            let req_body = MakeMoveRequest {
                x: number_point.x,
                y: number_point.y,
                game_id: Some(new_game.id),
            };

            let req = post()
                .uri(&uri_game(new_game.id))
                .insert_header((X_MOCK_AUTH, "true"))
                .set_json(&req_body)
                .to_request();
            let revealed_game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            let req = post()
                .uri(&uri_flag(new_game.id))
                .insert_header((X_MOCK_AUTH, "true"))
                .set_json(&req_body)
                .to_request();
            let flagged_game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            assert_eq!(revealed_game.board, flagged_game.board);
        }

        #[actix_web::test]
        async fn get_game_returns_not_found_when_id_does_not_exist() {
            let (app, _repo, _node) = $setup_fn().await;

            let req = test::TestRequest::get().uri(&uri_game(999999)).to_request();
            let resp = test::call_service(&app, req).await;

            assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
        }

        #[actix_web::test]
        async fn move_returns_bad_request_when_coordinates_are_out_of_bounds() {
            let (app, _repo, _node) = $setup_fn().await;

            let new_game: MinesweeperGameDto = test::call_and_read_body_json(
                &app,
                post().uri(&uri_new_game(10, 10, 5)).to_request(),
            )
            .await;

            let req_body = MakeMoveRequest {
                x: 10,
                y: 10,
                game_id: Some(new_game.id),
            };
            let req = post()
                .uri(&uri_game(new_game.id))
                .set_json(&req_body)
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
        }

        #[actix_web::test]
        async fn move_on_zero_square_reveals_multiple_squares() {
            let (app, repo, _node) = $setup_fn().await;

            let new_game: MinesweeperGameDto = test::call_and_read_body_json(
                &app,
                post().uri(&uri_new_game(10, 10, 5)).to_request(),
            )
            .await;
            let zero_point = get_point_by_type(&repo, new_game.id, |s| s == BoardState::Zero)
                .await
                .expect("No zero point found");
            let req_body = MakeMoveRequest {
                x: zero_point.x,
                y: zero_point.y,
                game_id: Some(new_game.id),
            };

            let req = post()
                .uri(&uri_game(new_game.id))
                .set_json(&req_body)
                .to_request();
            let updated_game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            let revealed_count = updated_game
                .board
                .iter()
                .flatten()
                .filter(|&&s| s != BoardState::Unknown)
                .count();
            assert!(revealed_count > 1);
        }

        #[actix_web::test]
        async fn history_reports_won_games_correctly() {
            let (app, repo, _node) = $setup_fn().await;

            let new_game: MinesweeperGameDto = test::call_and_read_body_json(
                &app,
                post()
                    .uri(&uri_new_game(3, 3, 1))
                    .insert_header((X_MOCK_AUTH, "true"))
                    .to_request(),
            )
            .await;

            // Get safe points
            let safe_points_vec = {
                let mut game = repo
                    .get_game(new_game.id)
                    .await
                    .expect("repo.get_game failed")
                    .expect("game not found");
                if !game.mines_generated {
                    let engine = MinesweeperEngine;
                    engine.generate_mines(&mut game, Point { x: 0, y: 0 });
                    repo.save(game.clone()).await.expect("repo.save failed");
                }
                let mut points = Vec::new();
                for x in 0..game.cols {
                    for y in 0..game.rows {
                        let p = Point { x, y };
                        if !game.mine_points.contains(&p) {
                            points.push(p);
                        }
                    }
                }
                points
            };

            // Make moves on all safe points
            for point in safe_points_vec {
                let req_body = MakeMoveRequest {
                    x: point.x,
                    y: point.y,
                    game_id: Some(new_game.id),
                };
                let req = post()
                    .uri(&uri_game(new_game.id))
                    .insert_header((X_MOCK_AUTH, "true"))
                    .set_json(&req_body)
                    .to_request();
                let _game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;
            }

            // Check history
            let req = test::TestRequest::get()
                .uri(&uri_user_games())
                .insert_header((X_MOCK_AUTH, "true"))
                .to_request();
            let user_games: Vec<MinesweeperGameDto> =
                test::call_and_read_body_json(&app, req).await;

            let my_game = user_games
                .iter()
                .find(|g| g.id == new_game.id)
                .expect("Game not found in history");
            assert_eq!(my_game.status, rust_backend::model::GameStatus::Won);
        }

        #[actix_web::test]
        async fn mock_auth_works() {
            let (app, _repo, _node) = $setup_fn().await;

            // Make a request without login but with Mock header
            let req = post()
                .uri(&uri_new_game(10, 10, 10))
                .insert_header((X_MOCK_AUTH, "true"))
                .to_request();
            let new_game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            let req = test::TestRequest::get()
                .uri(&uri_user_games())
                .insert_header((X_MOCK_AUTH, "true"))
                .to_request();

            let user_games: Vec<MinesweeperGameDto> =
                test::call_and_read_body_json(&app, req).await;

            assert!(user_games.iter().any(|g| g.id == new_game.id));
        }

        #[actix_web::test]
        async fn prevent_editing_others_games() {
            let (app, _repo, _node) = $setup_fn().await;

            // User 1 creates a game
            let req = post()
                .uri(&uri_new_game(10, 10, 10))
                .insert_header((X_MOCK_AUTH, "true"))
                .insert_header(("X-User-Sub", "user-1"))
                .to_request();
            let game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            // User 2 tries to make a move on User 1's game
            let req_body = MakeMoveRequest {
                x: 0,
                y: 0,
                game_id: Some(game.id),
            };
            let req = post()
                .uri(&uri_game(game.id))
                .insert_header((X_MOCK_AUTH, "true"))
                .insert_header(("X-User-Sub", "user-2"))
                .set_json(&req_body)
                .to_request();
            let resp = test::call_service(&app, req).await;

            assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
        }

        #[actix_web::test]
        async fn create_game_fails_when_dimensions_too_large() {
            let (app, _repo, _node) = $setup_fn().await;

            let req = post().uri(&uri_new_game(51, 50, 10)).to_request();
            let resp = test::call_service(&app, req).await;

            assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
        }

        #[actix_web::test]
        async fn create_game_fails_when_too_many_mines() {
            let (app, _repo, _node) = $setup_fn().await;

            let req = post().uri(&uri_new_game(10, 10, 100)).to_request();
            let resp = test::call_service(&app, req).await;

            assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
        }
    };
}

mod in_memory_tests {
    use super::*;

    async fn setup() -> (
        impl actix_web::dev::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
        >,
        Arc<dyn MinesweeperRepository>,
        Option<bool>,
    ) {
        let repo = Arc::new(InMemoryGameRepository::new());
        let app = create_test_app(repo.clone()).await;
        (app, repo, None)
    }

    define_api_tests!(setup);

    #[actix_web::test]
    async fn prevent_reading_others_games() {
        let (app, _repo, _node) = setup().await;

        // User 1 creates a game
        let req = post()
            .uri(&uri_new_game(10, 10, 10))
            .insert_header((X_MOCK_AUTH, "true"))
            .insert_header(("X-User-Sub", "user-1"))
            .to_request();
        let game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

        // Unauthenticated read should be rejected for owned games
        let req = test::TestRequest::get()
            .uri(&uri_game(game.id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);

        // Wrong user should be rejected
        let req = test::TestRequest::get()
            .uri(&uri_game(game.id))
            .insert_header((X_MOCK_AUTH, "true"))
            .insert_header(("X-User-Sub", "user-2"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);

        // Owner should succeed
        let req = test::TestRequest::get()
            .uri(&uri_game(game.id))
            .insert_header((X_MOCK_AUTH, "true"))
            .insert_header(("X-User-Sub", "user-1"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }
}

mod redis_tests {
    use super::*;
    use async_trait::async_trait;
    use rand::Rng;
    use redis::aio::ConnectionManager;
    use redis::AsyncCommands;
    use rust_backend::error::{AppError, AppResult};
    use rust_backend::model::{MinesweeperGame, Point};
    use rust_backend::repository::{GameRepository, UserGameRepository};
    use testcontainers::core::WaitFor;
    use testcontainers::Image;

    static DOCKER: Lazy<Cli> = Lazy::new(Cli::default);

    #[derive(Default)]
    struct RedisImage;

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

    struct RedisOnlyRepository {
        manager: ConnectionManager,
        namespace: String,
    }

    impl RedisOnlyRepository {
        async fn new(redis_uri: &str, namespace: String) -> AppResult<Self> {
            let client = redis::Client::open(redis_uri)
                .map_err(|e| AppError::Internal(format!("Failed to connect to Redis: {e}")))?;
            let manager = ConnectionManager::new(client).await.map_err(|e| {
                AppError::Internal(format!("Failed to create Redis connection manager: {e}"))
            })?;
            Ok(Self { manager, namespace })
        }

        fn game_key(&self, id: i32) -> String {
            format!("{}:game:{}", self.namespace, id)
        }

        fn game_owner_key(&self, id: i32) -> String {
            format!("{}:game_owner:{}", self.namespace, id)
        }

        fn user_games_key(&self, user_id: &str) -> String {
            format!("{}:user_games:{}", self.namespace, user_id)
        }

        const TTL_SECONDS: u64 = 24 * 60 * 60;
    }

    #[async_trait]
    impl GameRepository for RedisOnlyRepository {
        async fn get_game(&self, id: i32) -> AppResult<Option<MinesweeperGame>> {
            let mut conn = self.manager.clone();
            let val: Option<String> = conn
                .get(self.game_key(id))
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            match val {
                Some(json) => {
                    let game = serde_json::from_str(&json)
                        .map_err(|e| AppError::Internal(e.to_string()))?;
                    Ok(Some(game))
                }
                None => Ok(None),
            }
        }

        async fn get_games_by_ids(&self, ids: &[i32]) -> AppResult<Vec<MinesweeperGame>> {
            if ids.is_empty() {
                return Ok(vec![]);
            }

            let mut conn = self.manager.clone();
            let keys: Vec<String> = ids.iter().map(|id| self.game_key(*id)).collect();
            let vals: Vec<Option<String>> = conn
                .mget(keys)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;

            let mut games = Vec::new();
            for val in vals.into_iter().flatten() {
                let game =
                    serde_json::from_str(&val).map_err(|e| AppError::Internal(e.to_string()))?;
                games.push(game);
            }
            Ok(games)
        }

        async fn save(&self, game: MinesweeperGame) -> AppResult<()> {
            let mut conn = self.manager.clone();
            let json =
                serde_json::to_string(&game).map_err(|e| AppError::Internal(e.to_string()))?;
            let _: () = conn
                .set_ex(self.game_key(game.id), json, Self::TTL_SECONDS)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            Ok(())
        }

        async fn delete(&self, id: i32) -> AppResult<()> {
            let mut conn = self.manager.clone();
            let _: () = conn
                .del(self.game_key(id))
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let _: () = conn
                .del(self.game_owner_key(id))
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            Ok(())
        }

        async fn add_moves(&self, id: i32, points: &[Point]) -> AppResult<Option<MinesweeperGame>> {
            let mut game = match self.get_game(id).await? {
                Some(g) => g,
                None => return Ok(None),
            };

            for p in points {
                game.moves.insert(*p);
            }

            self.save(game.clone()).await?;
            Ok(Some(game))
        }

        async fn add_flag(&self, id: i32, point: Point) -> AppResult<Option<MinesweeperGame>> {
            let mut game = match self.get_game(id).await? {
                Some(g) => g,
                None => return Ok(None),
            };

            game.flag_points.insert(point);
            self.save(game.clone()).await?;
            Ok(Some(game))
        }

        async fn remove_flag(&self, id: i32, point: Point) -> AppResult<Option<MinesweeperGame>> {
            let mut game = match self.get_game(id).await? {
                Some(g) => g,
                None => return Ok(None),
            };

            game.flag_points.remove(&point);
            self.save(game.clone()).await?;
            Ok(Some(game))
        }
    }

    #[async_trait]
    impl UserGameRepository for RedisOnlyRepository {
        async fn add_mapping(&self, user_id: &str, game_id: i32) -> AppResult<()> {
            let mut conn = self.manager.clone();

            let _: () = conn
                .sadd(self.user_games_key(user_id), game_id.to_string())
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let _: () = conn
                .expire(self.user_games_key(user_id), Self::TTL_SECONDS as i64)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;

            let _: () = conn
                .set_ex(self.game_owner_key(game_id), user_id, Self::TTL_SECONDS)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;

            Ok(())
        }

        async fn get_game_ids_by_user_id(&self, user_id: &str) -> AppResult<Vec<i32>> {
            let mut conn = self.manager.clone();
            let ids: Vec<String> = conn
                .smembers(self.user_games_key(user_id))
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            Ok(ids
                .into_iter()
                .filter_map(|s| s.parse::<i32>().ok())
                .collect())
        }

        async fn get_game_owner(&self, game_id: i32) -> AppResult<Option<String>> {
            let mut conn = self.manager.clone();
            let owner: Option<String> = conn
                .get(self.game_owner_key(game_id))
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            Ok(owner)
        }

        async fn get_games_by_user_id(&self, user_id: &str) -> AppResult<Vec<MinesweeperGame>> {
            let ids = self.get_game_ids_by_user_id(user_id).await?;
            self.get_games_by_ids(&ids).await
        }
    }

    async fn setup() -> (
        impl actix_web::dev::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
        >,
        Arc<dyn MinesweeperRepository>,
        Option<Container<'static, RedisImage>>,
    ) {
        let node = DOCKER.run(RedisImage);
        let host_port = node.get_host_port_ipv4(6379);
        let redis_url = format!("redis://localhost:{host_port}");

        let namespace = format!("MinesweeperTest_{}", rand::thread_rng().gen::<u64>());
        let repo = RedisOnlyRepository::new(&redis_url, namespace)
            .await
            .expect("Failed to create Redis repo");
        let repo_arc: Arc<dyn MinesweeperRepository> = Arc::new(repo);
        let app = create_test_app(repo_arc.clone()).await;

        // Keep the container alive for the duration of the test; drop cleans it up.
        (app, repo_arc, Some(node))
    }

    define_api_tests!(setup);
}

mod mongo_tests {
    use super::*;
    use rand::Rng;

    static DOCKER: Lazy<Cli> = Lazy::new(Cli::default);

    async fn setup() -> (
        impl actix_web::dev::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
        >,
        Arc<dyn MinesweeperRepository>,
        Option<Container<'static, MongoImage>>,
    ) {
        // Run a fresh container per test to guarantee cleanup on drop (no global caching).
        let node = DOCKER.run(MongoImage);

        let host_port = node.get_host_port_ipv4(27017);
        let url = format!("mongodb://localhost:{}", host_port);

        // Use a unique database per test to allow parallel execution on a shared container
        let db_name = format!("MinesweeperTest_{}", rand::thread_rng().gen::<u32>());
        let repo = MongoGameRepository::new(&url, &db_name)
            .await
            .expect("Failed to create Mongo repo");
        let repo_arc: Arc<dyn MinesweeperRepository> = Arc::new(repo);
        let app = create_test_app(repo_arc.clone()).await;

        // Keep the container alive for the duration of the test; drop cleans it up.
        (app, repo_arc, Some(node))
    }

    define_api_tests!(setup);
}

#[cfg(test)]
mod settings_tests {
    use rust_backend::settings::{
        AuthSettings, DatabaseSettings, ServerSettings, Settings, TelemetrySettings,
    };

    #[test]
    fn settings_validation_fails_in_production_with_short_key() {
        let settings = Settings {
            environment: "production".to_string(),
            server: ServerSettings {
                port: 8080,
                secure_cookies: true,
                public_origin: None,
                allowed_origins: vec![],
                cors_supports_credentials: false,
                cross_origin_cookies: false,
                trusted_proxies: vec![],
                session_secret_key: "short".to_string(),
                rate_limit_period_ms: 50,
                rate_limit_burst_size: 50,
            },
            database: DatabaseSettings {
                addr: None,
                name: "test".to_string(),
            },
            redis: rust_backend::settings::RedisSettings {
                addr: None,
                ..Default::default()
            },
            auth: AuthSettings {
                google_client_id: String::new(),
                google_client_secret: String::new(),
                google_redirect_uri: None,
            },
            telemetry: TelemetrySettings {
                otlp_endpoint: "http://localhost:4317".to_string(),
            },
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn settings_validation_fails_even_in_development_with_short_key() {
        let settings = Settings {
            environment: "development".to_string(),
            server: ServerSettings {
                port: 8080,
                secure_cookies: false,
                public_origin: None,
                allowed_origins: vec![],
                cors_supports_credentials: false,
                cross_origin_cookies: false,
                trusted_proxies: vec![],
                session_secret_key: "short".to_string(),
                rate_limit_period_ms: 50,
                rate_limit_burst_size: 50,
            },
            database: DatabaseSettings {
                addr: None,
                name: "test".to_string(),
            },
            redis: rust_backend::settings::RedisSettings {
                addr: None,
                ..Default::default()
            },
            auth: AuthSettings {
                google_client_id: String::new(),
                google_client_secret: String::new(),
                google_redirect_uri: None,
            },
            telemetry: TelemetrySettings {
                otlp_endpoint: "http://localhost:4317".to_string(),
            },
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn settings_validation_fails_in_production_when_secure_cookies_disabled() {
        let settings = Settings {
            environment: "production".to_string(),
            server: ServerSettings {
                port: 8080,
                secure_cookies: false,
                public_origin: None,
                allowed_origins: vec!["https://example.com".to_string()],
                cors_supports_credentials: true,
                cross_origin_cookies: false,
                trusted_proxies: vec![],
                session_secret_key: "a".repeat(64),
                rate_limit_period_ms: 50,
                rate_limit_burst_size: 50,
            },
            database: DatabaseSettings {
                addr: None,
                name: "test".to_string(),
            },
            redis: rust_backend::settings::RedisSettings {
                addr: None,
                ..Default::default()
            },
            auth: AuthSettings {
                google_client_id: "".to_string(),
                google_client_secret: "".to_string(),
                google_redirect_uri: None,
            },
            telemetry: TelemetrySettings {
                otlp_endpoint: "http://localhost:4317".to_string(),
            },
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn settings_validation_fails_in_production_when_allowed_origins_missing() {
        let settings = Settings {
            environment: "production".to_string(),
            server: ServerSettings {
                port: 8080,
                secure_cookies: true,
                public_origin: None,
                allowed_origins: vec![],
                cors_supports_credentials: true,
                cross_origin_cookies: false,
                trusted_proxies: vec![],
                session_secret_key: "a".repeat(64),
                rate_limit_period_ms: 50,
                rate_limit_burst_size: 50,
            },
            database: DatabaseSettings {
                addr: None,
                name: "test".to_string(),
            },
            redis: rust_backend::settings::RedisSettings {
                addr: None,
                ..Default::default()
            },
            auth: AuthSettings {
                google_client_id: "".to_string(),
                google_client_secret: "".to_string(),
                google_redirect_uri: None,
            },
            telemetry: TelemetrySettings {
                otlp_endpoint: "http://localhost:4317".to_string(),
            },
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn settings_validation_fails_in_production_when_allowed_origins_has_wildcard() {
        let settings = Settings {
            environment: "production".to_string(),
            server: ServerSettings {
                port: 8080,
                secure_cookies: true,
                public_origin: None,
                allowed_origins: vec!["*".to_string()],
                cors_supports_credentials: true,
                cross_origin_cookies: false,
                trusted_proxies: vec![],
                session_secret_key: "a".repeat(64),
                rate_limit_period_ms: 50,
                rate_limit_burst_size: 50,
            },
            database: DatabaseSettings {
                addr: None,
                name: "test".to_string(),
            },
            redis: rust_backend::settings::RedisSettings {
                addr: None,
                ..Default::default()
            },
            auth: AuthSettings {
                google_client_id: "".to_string(),
                google_client_secret: "".to_string(),
                google_redirect_uri: None,
            },
            telemetry: TelemetrySettings {
                otlp_endpoint: "http://localhost:4317".to_string(),
            },
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn settings_validation_fails_in_production_when_allowed_origins_is_not_http_origin() {
        let settings = Settings {
            environment: "production".to_string(),
            server: ServerSettings {
                port: 8080,
                secure_cookies: true,
                public_origin: None,
                allowed_origins: vec!["example.com".to_string()],
                cors_supports_credentials: true,
                cross_origin_cookies: false,
                trusted_proxies: vec![],
                session_secret_key: "a".repeat(64),
                rate_limit_period_ms: 50,
                rate_limit_burst_size: 50,
            },
            database: DatabaseSettings {
                addr: None,
                name: "test".to_string(),
            },
            redis: rust_backend::settings::RedisSettings {
                addr: None,
                ..Default::default()
            },
            auth: AuthSettings {
                google_client_id: "".to_string(),
                google_client_secret: "".to_string(),
                google_redirect_uri: None,
            },
            telemetry: TelemetrySettings {
                otlp_endpoint: "http://localhost:4317".to_string(),
            },
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn settings_validation_succeeds_in_production_with_secure_cookies_and_allowed_origins() {
        let settings = Settings {
            environment: "production".to_string(),
            server: ServerSettings {
                port: 8080,
                secure_cookies: true,
                public_origin: None,
                allowed_origins: vec!["https://example.com".to_string()],
                cors_supports_credentials: true,
                cross_origin_cookies: false,
                trusted_proxies: vec![],
                session_secret_key: "a".repeat(64),
                rate_limit_period_ms: 50,
                rate_limit_burst_size: 50,
            },
            database: DatabaseSettings {
                addr: None,
                name: "test".to_string(),
            },
            redis: rust_backend::settings::RedisSettings {
                addr: None,
                ..Default::default()
            },
            auth: AuthSettings {
                google_client_id: "".to_string(),
                google_client_secret: "".to_string(),
                google_redirect_uri: None,
            },
            telemetry: TelemetrySettings {
                otlp_endpoint: "http://localhost:4317".to_string(),
            },
        };

        assert!(settings.validate().is_ok());
    }
}

#[cfg(test)]
mod xsrf_tests {
    use super::common::*;
    use actix_web::http::StatusCode;
    use actix_web::test;
    use rust_backend::model::MakeMoveRequest;
    use rust_backend::model::MinesweeperGameDto;
    use rust_backend::repository::InMemoryGameRepository;
    use std::sync::Arc;

    #[actix_web::test]
    async fn unsafe_requests_without_xsrf_header_are_allowed_when_origin_is_same() {
        let repo = Arc::new(InMemoryGameRepository::new());
        let app = create_test_app(repo.clone()).await;

        // Create a game first (POST).
        let new_game: MinesweeperGameDto =
            test::call_and_read_body_json(&app, post().uri(&uri_new_game(10, 10, 10)).to_request())
                .await;

        // POST without XSRF header/cookie, but with same-origin Origin header.
        let req_body = MakeMoveRequest {
            x: 0,
            y: 0,
            game_id: Some(new_game.id),
        };
        let req = test::TestRequest::post()
            .uri(&uri_game(new_game.id))
            .insert_header(("Host", "localhost"))
            .insert_header(("Origin", "http://localhost"))
            .set_json(&req_body)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_ne!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[actix_web::test]
    async fn unsafe_requests_with_xsrf_header_still_work() {
        let repo = Arc::new(InMemoryGameRepository::new());
        let app = create_test_app(repo.clone()).await;

        let new_game: MinesweeperGameDto =
            test::call_and_read_body_json(&app, post().uri(&uri_new_game(10, 10, 10)).to_request())
                .await;

        let req_body = MakeMoveRequest {
            x: 0,
            y: 0,
            game_id: Some(new_game.id),
        };
        let req = post()
            .uri(&uri_game(new_game.id))
            .set_json(&req_body)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_ne!(resp.status(), StatusCode::FORBIDDEN);
    }
}
