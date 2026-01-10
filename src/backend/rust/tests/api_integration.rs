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
    let mut game = repo.get_game(game_id).await.unwrap().unwrap();
    if !game.mines_generated {
        let engine = MinesweeperEngine;
        engine.generate_mines(&mut game, Point { x: 0, y: 0 });
        repo.save(game.clone()).await.unwrap();
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
            let req = test::TestRequest::get()
                .uri(&uri_new_game(10, 10, 10))
                .to_request();
            let resp: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            assert!(resp.created_at >= start_time - chrono::Duration::seconds(10));
            assert!(resp.created_at <= Utc::now() + chrono::Duration::seconds(10));
        }

        #[actix_web::test]
        async fn create_new_game_associates_with_user() {
            let (app, _repo, _node) = $setup_fn().await;

            // Header based auth injection
            let req = test::TestRequest::get()
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
                test::TestRequest::get()
                    .uri(&uri_new_game(10, 100, 10))
                    .to_request(),
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
                test::TestRequest::get()
                    .uri(&uri_new_game(10, 10, 10))
                    .to_request(),
            )
            .await;
            let req_body = MakeMoveRequest {
                x: 0,
                y: 0,
                game_id: Some(new_game.id),
            };

            let req = test::TestRequest::post()
                .uri(&uri_flag(new_game.id))
                .set_json(&req_body)
                .to_request();
            let game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            assert_eq!(game.board[0][0], BoardState::Flag);

            let req = test::TestRequest::post()
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
                test::TestRequest::get()
                    .uri(&uri_new_game(10, 10, 5))
                    .to_request(),
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

            let req = test::TestRequest::post()
                .uri(&uri_flag(new_game.id))
                .set_json(&req_body)
                .to_request();
            let flagged_game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            let req = test::TestRequest::post()
                .uri(&uri_game(new_game.id))
                .set_json(&req_body)
                .to_request();
            let game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            assert_eq!(flagged_game.board, game.board);
        }

        #[actix_web::test]
        async fn move_on_numbered_space_returns_correct_board_state() {
            let (app, repo, _node) = $setup_fn().await;

            let new_game: MinesweeperGameDto = test::call_and_read_body_json(
                &app,
                test::TestRequest::get()
                    .uri(&uri_new_game(10, 10, 5))
                    .to_request(),
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

            let req = test::TestRequest::post()
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
                test::TestRequest::get()
                    .uri(&uri_new_game(10, 10, 5))
                    .to_request(),
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

            let req = test::TestRequest::post()
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
                test::TestRequest::get()
                    .uri(&uri_new_game(10, 10, 5))
                    .to_request(),
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

            let req = test::TestRequest::post()
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

            let req = test::TestRequest::post()
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
                test::TestRequest::get()
                    .uri(&uri_new_game(10, 10, 5))
                    .to_request(),
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

            let req = test::TestRequest::post()
                .uri(&uri_game(new_game.id))
                .set_json(&req_body)
                .to_request();
            let revealed_game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            let req = test::TestRequest::post()
                .uri(&uri_flag(new_game.id))
                .set_json(&req_body)
                .to_request();
            let flagged_game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            assert_eq!(revealed_game.board, flagged_game.board);
        }

        #[actix_web::test]
        async fn get_game_returns_not_found_when_id_does_not_exist() {
            let (app, _repo, _node) = $setup_fn().await;

            let req = test::TestRequest::get()
                .uri(&uri_game(999999))
                .to_request();
            let resp = test::call_service(&app, req).await;

            assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
        }

        #[actix_web::test]
        async fn move_returns_bad_request_when_coordinates_are_out_of_bounds() {
            let (app, _repo, _node) = $setup_fn().await;

            let new_game: MinesweeperGameDto = test::call_and_read_body_json(
                &app,
                test::TestRequest::get()
                    .uri(&uri_new_game(10, 10, 5))
                    .to_request(),
            )
            .await;

            let req_body = MakeMoveRequest {
                x: 10,
                y: 10,
                game_id: Some(new_game.id),
            };
            let req = test::TestRequest::post()
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
                test::TestRequest::get()
                    .uri(&uri_new_game(10, 10, 1))
                    .to_request(),
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

            let req = test::TestRequest::post()
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

            // Create game (small 3x3 with 1 mine to make winning easy)
            let req = test::TestRequest::get()
                .uri(&uri_new_game(3, 3, 1))
                .insert_header((X_MOCK_AUTH, "true"))
                .to_request();
            let new_game: MinesweeperGameDto = test::call_and_read_body_json(&app, req).await;

            // Get safe points
            let safe_points_vec = {
                let mut game = repo.get_game(new_game.id).await.unwrap().unwrap();
                if !game.mines_generated {
                    let engine = MinesweeperEngine;
                    engine.generate_mines(&mut game, Point { x: 0, y: 0 });
                    repo.save(game.clone()).await.unwrap();
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
                let req = test::TestRequest::post()
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
            let req = test::TestRequest::get()
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
            let req = test::TestRequest::get()
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
            let req = test::TestRequest::post()
                .uri(&uri_game(game.id))
                .insert_header((X_MOCK_AUTH, "true"))
                .insert_header(("X-User-Sub", "user-2"))
                .set_json(&req_body)
                .to_request();
            let resp = test::call_service(&app, req).await;

            assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
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
}

mod mongo_tests {
    use super::*;
    use rand::Rng;
    use std::sync::OnceLock;

    static DOCKER: Lazy<Cli> = Lazy::new(Cli::default);
    static NODE: OnceLock<Container<'static, MongoImage>> = OnceLock::new();

    async fn setup() -> (
        impl actix_web::dev::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
        >,
        Arc<dyn MinesweeperRepository>,
        Option<bool>,
    ) {
        let node = NODE.get_or_init(|| {
            // Using a simple run call, ensuring we use unique DBs per test to avoid interference
            DOCKER.run(MongoImage)
        });

        let host_port = node.get_host_port_ipv4(27017);
        let url = format!("mongodb://localhost:{}", host_port);

        // Use a unique database per test to allow parallel execution on a shared container
        let db_name = format!("MinesweeperTest_{}", rand::thread_rng().gen::<u32>());
        let repo = MongoGameRepository::new(&url, &db_name)
            .await
            .expect("Failed to create Mongo repo");
        let repo_arc: Arc<dyn MinesweeperRepository> = Arc::new(repo);
        let app = create_test_app(repo_arc.clone()).await;

        (app, repo_arc, None)
    }

    define_api_tests!(setup);
}
