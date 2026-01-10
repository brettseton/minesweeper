use crate::auth::IdentityExt;
use crate::error::{AppError, AppResult};
use crate::model::MinesweeperGameDto;
use crate::service::GameService;
use actix_identity::Identity;
use actix_web::{web, HttpResponse};
use std::sync::Arc;

pub const SCOPE_USER: &str = "/user";

pub const PATH_GAMES: &str = "/games";
pub const PATH_STATS: &str = "/stats";

pub async fn user_games(
    service: web::Data<Arc<dyn GameService>>,
    identity: Identity,
) -> AppResult<HttpResponse> {
    let user = identity.user_info().ok_or(AppError::Unauthorized)?;
    let games = service.get_user_games(user).await?;
    let dtos: Vec<_> = games
        .into_iter()
        .map(|g| MinesweeperGameDto::from(&g))
        .collect();
    Ok(HttpResponse::Ok().json(dtos))
}

pub async fn user_stats(
    service: web::Data<Arc<dyn GameService>>,
    identity: Identity,
) -> AppResult<HttpResponse> {
    let user = identity.user_info().ok_or(AppError::Unauthorized)?;
    let stats = service.get_user_stats(user).await?;
    Ok(HttpResponse::Ok().json(stats))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(SCOPE_USER)
            .route(PATH_GAMES, web::get().to(user_games))
            .route(PATH_STATS, web::get().to(user_stats)),
    );
}
