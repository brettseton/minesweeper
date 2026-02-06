use super::game_state_store::GameStateStore;
use super::GameService;
use crate::engine::BoardEngine;
use crate::error::{AppError, AppResult};
use crate::model::{MinesweeperGame, Point, UserInfo, UserStatsDto};
use crate::repository::MinesweeperRepository;
use crate::settings::RedisSettings;
use crate::telemetry::metrics::MinesweeperMetrics;
use async_trait::async_trait;
use std::sync::Arc;

pub struct MinesweeperService {
    store: GameStateStore,
    engine: Arc<dyn BoardEngine>,
}

const MAX_REDIS_WRITE_CONFLICT_RETRIES: usize = 8;

impl MinesweeperService {
    pub fn new(
        repo: Arc<dyn MinesweeperRepository>,
        hot_repo: Option<Arc<crate::repository::RedisGameRepository>>,
        engine: Arc<dyn BoardEngine>,
        redis: RedisSettings,
    ) -> Self {
        Self {
            store: GameStateStore::new(repo, hot_repo, redis),
            engine,
        }
    }

    async fn check_ownership(&self, game_id: i32, user: Option<UserInfo>) -> AppResult<()> {
        let owner_id = self.store.get_game_owner(game_id).await?;

        tracing::debug!(
            "Checking ownership: game_id={}, owner_id={:?}, user={:?}",
            game_id,
            owner_id,
            user
        );

        match (owner_id, user) {
            (Some(owner), Some(u)) if owner != u.sub => {
                tracing::warn!("Unauthorized: owner={}, user={}", owner, u.sub);
                Err(AppError::Unauthorized)
            }
            (Some(_), None) => {
                tracing::warn!("Unauthorized: game has owner but no user provided");
                Err(AppError::Unauthorized)
            }
            _ => Ok(()),
        }
    }
}

#[async_trait]
impl GameService for MinesweeperService {
    async fn get_game(&self, id: i32) -> AppResult<MinesweeperGame> {
        self.store.fetch_game(id).await
    }

    async fn get_game_for_user(
        &self,
        id: i32,
        user: Option<UserInfo>,
    ) -> AppResult<MinesweeperGame> {
        self.check_ownership(id, user).await?;
        self.store.fetch_game(id).await
    }

    async fn create_game(
        &self,
        cols: usize,
        rows: usize,
        mines: usize,
        user: Option<UserInfo>,
    ) -> AppResult<MinesweeperGame> {
        // Enforce explicit maximum bounds to prevent DoS/OOM.
        if cols == 0 || rows == 0 || cols > 100 || rows > 100 {
            return Err(AppError::BadRequest(
                "Invalid board dimensions (min 1x1, max 100x100)".to_string(),
            ));
        }

        let total_cells = cols
            .checked_mul(rows)
            .ok_or_else(|| AppError::BadRequest("Board dimensions result in overflow".to_string()))?;

        if total_cells > 2500 {
            return Err(AppError::BadRequest(
                "Board dimensions too large (max 2500 cells)".to_string(),
            ));
        }

        if mines == 0 || mines >= total_cells {
            return Err(AppError::BadRequest(
                "Invalid mine count (must be at least 1 and less than total cells)".to_string(),
            ));
        }

        let game = MinesweeperGame::new(cols, rows, mines);

        self.store.persist_new_game(&game).await?;

        if let Some(user_info) = user {
            self.store
                .associate_user_game(&user_info.sub, game.id)
                .await?;
        }

        MinesweeperMetrics::record_game_started();

        Ok(game)
    }

    async fn make_move(
        &self,
        id: i32,
        point: Point,
        user: Option<UserInfo>,
    ) -> AppResult<MinesweeperGame> {
        for attempt in 0..=MAX_REDIS_WRITE_CONFLICT_RETRIES {
            let write_ctx = self.store.fetch_game_for_write_with_hot_version(id).await?;
            let mut game = write_ctx.game;
            self.check_ownership(id, user.clone()).await?;

            if !game.is_valid_point(&point) {
                return Err(AppError::BadRequest(
                    "Coordinates out of bounds".to_string(),
                ));
            }

            if game.is_game_over()
                || game.is_point_revealed(&point)
                || game.is_point_flagged(&point)
            {
                return Ok(game);
            }

            if !game.mines_generated {
                self.engine.generate_mines(&mut game, point);
            }

            let mut reveal_points = self.engine.get_reveal_points(&game, point);
            // Never auto-reveal flagged cells (e.g., during zero-wave propagation).
            reveal_points.retain(|p| !game.is_point_flagged(p));

            // Apply moves to local object for Game Over check.
            for p in &reveal_points {
                game.moves.insert(*p);
            }

            let is_over = game.is_game_over();

            match self
                .store
                .persist_move_with_hot_version(
                    id,
                    &game,
                    &reveal_points,
                    is_over,
                    write_ctx.hot_version,
                )
                .await?
            {
                crate::service::game_state_store::PersistResult::Saved => {
                    MinesweeperMetrics::record_move(&game);
                    return Ok(game);
                }
                crate::service::game_state_store::PersistResult::Conflict => {
                    tracing::debug!(
                        game_id = id,
                        attempt = attempt,
                        "Redis optimistic write conflict; retrying"
                    );
                    continue;
                }
            }
        }

        Err(AppError::ServiceUnavailable(
            "Concurrent update; please retry".to_string(),
        ))
    }

    async fn toggle_flag(
        &self,
        id: i32,
        point: Point,
        user: Option<UserInfo>,
    ) -> AppResult<MinesweeperGame> {
        for attempt in 0..=MAX_REDIS_WRITE_CONFLICT_RETRIES {
            let write_ctx = self.store.fetch_game_for_write_with_hot_version(id).await?;
            let mut game = write_ctx.game;
            self.check_ownership(id, user.clone()).await?;

            if !game.is_valid_point(&point) {
                return Err(AppError::BadRequest(
                    "Coordinates out of bounds".to_string(),
                ));
            }

            if game.is_game_over() || game.is_point_revealed(&point) {
                return Ok(game);
            }

            if game.is_point_flagged(&point) {
                game.flag_points.remove(&point);
            } else {
                game.flag_points.insert(point);
            }

            match self
                .store
                .persist_toggle_flag_with_hot_version(&game, write_ctx.hot_version)
                .await?
            {
                crate::service::game_state_store::PersistResult::Saved => return Ok(game),
                crate::service::game_state_store::PersistResult::Conflict => {
                    tracing::debug!(
                        game_id = id,
                        attempt = attempt,
                        "Redis optimistic write conflict; retrying"
                    );
                    continue;
                }
            }
        }

        Err(AppError::ServiceUnavailable(
            "Concurrent update; please retry".to_string(),
        ))
    }

    async fn get_user_games(&self, user: UserInfo) -> AppResult<Vec<MinesweeperGame>> {
        let ids = self.store.get_game_ids_by_user_id(&user.sub).await?;
        if ids.is_empty() {
            return Ok(vec![]);
        }

        // Prefer active state from Redis when available, but always fall back to cold store.
        let mut games = Vec::with_capacity(ids.len());
        for id in ids {
            match self.store.get_game_from_hot_store_refresh_ttl(id).await {
                Ok(Some(game)) => {
                    games.push(game);
                    continue;
                }
                Ok(None) => {}
                Err(e) => tracing::warn!(
                    game_id = id,
                    error = %e,
                    "Redis hot-store read failed while building user games list"
                ),
            }

            if let Some(game) = self.store.get_game_from_cold_store(id).await? {
                games.push(game);
            }
        }

        Ok(games)
    }

    async fn get_user_stats(&self, user: UserInfo) -> AppResult<UserStatsDto> {
        let games = self.get_user_games(user).await?;

        let mut won = 0;
        let mut lost = 0;
        let mut in_progress = 0;

        for game in games {
            if game.is_game_won() {
                won += 1;
            } else if game.is_game_lost() {
                lost += 1;
            } else {
                in_progress += 1;
            }
        }

        Ok(UserStatsDto {
            won,
            lost,
            in_progress,
        })
    }
}
