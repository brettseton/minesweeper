use crate::auth::IdentityExt;
use crate::error::AppResult;
use crate::model::{MakeMoveRequest, MinesweeperGameDto, Point};
use crate::service::GameService;
use actix_identity::Identity;
use actix_web::{web, HttpResponse};
use std::sync::Arc;

pub const SCOPE_GAME: &str = "/game";

pub const PATH_NEW: &str = "/new";
pub const PATH_NEW_CUSTOM: &str = "/new/{cols}/{rows}/{mines}";
pub const PATH_FLAG: &str = "/flag";
pub const PATH_FLAG_ID: &str = "/flag/{id}";
pub const PATH_ID: &str = "/{id}";

pub async fn get_game(
    id: Option<web::Path<i32>>,
    identity: Option<Identity>,
    service: web::Data<Arc<dyn GameService>>,
) -> AppResult<HttpResponse> {
    let id = id.map(|p| p.into_inner()).unwrap_or(0);

    if id == 0 {
        return new_game_default(service, identity).await;
    }

    let game = service.get_game(id).await?;
    Ok(HttpResponse::Ok().json(MinesweeperGameDto::from(&game)))
}

pub async fn new_game_default(
    service: web::Data<Arc<dyn GameService>>,
    identity: Option<Identity>,
) -> AppResult<HttpResponse> {
    new_game_custom(web::Path::from((10, 10, 10)), service, identity).await
}

pub async fn new_game_custom(
    path: web::Path<(usize, usize, usize)>,
    service: web::Data<Arc<dyn GameService>>,
    identity: Option<Identity>,
) -> AppResult<HttpResponse> {
    let (cols, rows, mines) = path.into_inner();
    let user = identity.and_then(|id| id.user_info());

    let game = service.create_game(cols, rows, mines, user).await?;
    Ok(HttpResponse::Ok().json(MinesweeperGameDto::from(&game)))
}

pub async fn make_move(
    path: Option<web::Path<i32>>,
    req_body: web::Json<MakeMoveRequest>,
    identity: Option<Identity>,
    service: web::Data<Arc<dyn GameService>>,
) -> AppResult<HttpResponse> {
    let (game_id, point) = extract_request_params(path, req_body.into_inner());
    let user = identity.and_then(|id| id.user_info());
    let game = service.make_move(game_id, point, user).await?;
    Ok(HttpResponse::Ok().json(MinesweeperGameDto::from(&game)))
}

pub async fn toggle_flag(
    path: Option<web::Path<i32>>,
    req_body: web::Json<MakeMoveRequest>,
    identity: Option<Identity>,
    service: web::Data<Arc<dyn GameService>>,
) -> AppResult<HttpResponse> {
    let (game_id, point) = extract_request_params(path, req_body.into_inner());
    let user = identity.and_then(|id| id.user_info());
    let game = service.toggle_flag(game_id, point, user).await?;
    Ok(HttpResponse::Ok().json(MinesweeperGameDto::from(&game)))
}

fn extract_request_params(
    path: Option<web::Path<i32>>,
    req: MakeMoveRequest,
) -> (i32, Point) {
    let game_id = path
        .map(|p| p.into_inner())
        .or(req.game_id)
        .unwrap_or(0);
    (game_id, Point { x: req.x, y: req.y })
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(SCOPE_GAME)
            .route(PATH_NEW, web::get().to(get_game))
            .route(PATH_NEW_CUSTOM, web::get().to(new_game_custom))
            .route(PATH_FLAG, web::post().to(toggle_flag))
            .route(PATH_FLAG_ID, web::post().to(toggle_flag))
            .route("", web::get().to(get_game))
            .route(PATH_ID, web::get().to(get_game))
            .route("", web::post().to(make_move))
            .route(PATH_ID, web::post().to(make_move)),
    );
}
