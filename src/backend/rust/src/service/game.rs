use super::GameService;
use crate::engine::BoardEngine;
use crate::error::{AppError, AppResult};
use crate::model::{MinesweeperGame, Point, UserInfo, UserStatsDto};
use crate::repository::{GameRepository, MinesweeperRepository, RedisGameRepository};
use crate::telemetry::metrics::MinesweeperMetrics;
use async_trait::async_trait;
use chrono::Utc;
use crate::settings::RedisSettings;
use std::sync::Arc;

pub struct MinesweeperService {
    repo: Arc<dyn MinesweeperRepository>,
    hot_repo: Option<Arc<RedisGameRepository>>,
    redis: RedisSettings,
    engine: Arc<dyn BoardEngine>,
}

impl MinesweeperService {
    pub fn new(
        repo: Arc<dyn MinesweeperRepository>,
        hot_repo: Option<Arc<RedisGameRepository>>,
        engine: Arc<dyn BoardEngine>,
        redis: RedisSettings,
    ) -> Self {
        Self {
            repo,
            hot_repo,
            redis,
            engine,
        }
    }

    async fn fetch_game(&self, id: i32) -> AppResult<MinesweeperGame> {
        if id == 0 {
            return Err(AppError::BadRequest("Game ID is required".to_string()));
        }

        // 1. Try Hot Store (Redis)
        if let Some(ref hot) = self.hot_repo {
            match hot.get_game_refresh_ttl(id).await {
                Ok(Some(game)) => {
                    tracing::info!(game_id = id, "Redis hot-store hit");
                    return Ok(game);
                }
                Ok(None) => {
                    tracing::debug!(game_id = id, "Redis hot-store miss");
                }
                Err(e) => {
                    tracing::warn!(game_id = id, error = %e, "Redis hot-store read failed; falling back to cold store");
                }
            }
        }

        // 2. Fallback to Cold Store (Mongo)
        let game = self
            .repo
            .get_game(id)
            .await?
            .ok_or_else(|| AppError::NotFound(id.to_string()))?;

        // 3. Rehydrate Hot Store (best effort)
        if let Some(ref hot) = self.hot_repo {
            if let Err(e) = hot.save(game.clone()).await {
                tracing::warn!(game_id = id, error = %e, "Failed to rehydrate Redis from cold store");
            } else {
                tracing::info!(game_id = id, "Rehydrated Redis hot-store from cold store");
            }
        }

        Ok(game)
    }

    async fn fetch_game_for_write(&self, id: i32) -> AppResult<MinesweeperGame> {
        if self.hot_repo.is_some() && self.redis.required_for_writes {
            // If Redis is configured as the active store, writes should fail fast when Redis is down
            // to avoid split-brain / inconsistent state across stores.
            if let Some(ref hot) = self.hot_repo {
                match hot.get_game_refresh_ttl(id).await {
                    Ok(Some(game)) => return Ok(game),
                    Ok(None) => {
                        // Attempt rehydrate from cold and then proceed.
                    }
                    Err(e) => {
                        return Err(AppError::ServiceUnavailable(format!(
                            "Redis unavailable for writes: {e}"
                        )));
                    }
                }
            }
        }

        self.fetch_game(id).await
    }

    async fn check_ownership(&self, game_id: i32, user: Option<UserInfo>) -> AppResult<()> {
        let owner_id = self.repo.get_game_owner(game_id).await?;

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
        self.fetch_game(id).await
    }

    async fn create_game(
        &self,
        cols: usize,
        rows: usize,
        mines: usize,
        user: Option<UserInfo>,
    ) -> AppResult<MinesweeperGame> {
        if cols * rows > 2500 {
            return Err(AppError::BadRequest(
                "Board dimensions too large (max 2500 cells)".to_string(),
            ));
        }

        if mines >= cols * rows {
            return Err(AppError::BadRequest(
                "Too many mines for the given board size".to_string(),
            ));
        }

        let game = MinesweeperGame::new(cols, rows, mines);

        // Avoid inconsistent state when Redis is configured as required for writes:
        // either we successfully write both hot+cold (and mapping), or we roll back.
        if let Some(ref hot) = self.hot_repo {
            if self.redis.required_for_writes {
                tracing::info!(game_id = game.id, "Writing game to Redis hot store (required)");
                hot.save(game.clone()).await.map_err(|e| {
                    AppError::ServiceUnavailable(format!("Redis unavailable for writes: {e}"))
                })?;

                if let Err(e) = self.repo.save(game.clone()).await {
                    tracing::warn!(
                        game_id = game.id,
                        error = %e,
                        "Cold-store write failed after Redis write; attempting Redis rollback"
                    );
                    let _ = hot.delete_game_keys(game.id).await;
                    return Err(e);
                }
            } else {
                // Persist initial snapshot to cold store for history + rehydration.
                self.repo.save(game.clone()).await?;

                tracing::info!(game_id = game.id, "Writing game to Redis hot store");
                match hot.save(game.clone()).await {
                    Ok(()) => {}
                    Err(e) => {
                        tracing::warn!(
                            game_id = game.id,
                            error = %e,
                            "Failed to write game to Redis hot store"
                        );
                    }
                }
            }
        } else {
            // No Redis configured: cold store is source of truth.
            self.repo.save(game.clone()).await?;
        }

        if let Some(user_info) = user {
            if let Err(e) = self.repo.add_mapping(&user_info.sub, game.id).await {
                tracing::warn!(
                    game_id = game.id,
                    user_id = %user_info.sub,
                    error = %e,
                    "Failed to persist user->game mapping; attempting rollback"
                );
                let _ = self.repo.delete(game.id).await;
                if let Some(ref hot) = self.hot_repo {
                    let _ = hot.delete_game_keys(game.id).await;
                }
                return Err(e);
            }
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
        let mut game = self.fetch_game_for_write(id).await?;
        self.check_ownership(id, user).await?;

        if !game.is_valid_point(&point) {
            return Err(AppError::BadRequest(
                "Coordinates out of bounds".to_string(),
            ));
        }

        if game.is_game_over() || game.is_point_revealed(&point) || game.is_point_flagged(&point) {
            return Ok(game);
        }

        if !game.mines_generated {
            self.engine.generate_mines(&mut game, point);
            // Save initial state to active store (Redis) if available, otherwise fall back to cold.
            if let Some(ref hot) = self.hot_repo {
                tracing::info!(game_id = game.id, "Writing game to Redis hot store");
                match hot.save(game.clone()).await {
                    Ok(()) => {}
                    Err(e) if self.redis.required_for_writes => {
                        return Err(AppError::ServiceUnavailable(format!(
                            "Redis unavailable for writes: {e}"
                        )));
                    }
                    Err(e) => {
                        tracing::warn!(game_id = game.id, error = %e, "Failed to write game to Redis hot store");
                        // Ensure cold store has a usable snapshot even if Redis is down.
                        self.repo.save(game.clone()).await?;
                    }
                }
            } else {
                self.repo.save(game.clone()).await?;
            }
        }

        let reveal_points = self.engine.get_reveal_points(&game, point);

        // Apply moves to local object for Game Over check
        for p in &reveal_points {
            game.moves.insert(*p);
        }

        let is_over = game.is_game_over();

        if is_over {
            // GAME OVER: Save final state to Cold Store and remove from Hot Store
            self.repo.save(game.clone()).await?;
            if let Some(ref hot) = self.hot_repo {
                if let Err(e) = hot.delete_game_keys(id).await {
                    tracing::warn!(game_id = id, error = %e, "Failed to delete Redis hot-store keys after game over");
                }
            }
            MinesweeperMetrics::record_move(&game);
            return Ok(game);
        }

        // STILL IN PROGRESS: Redis is source of truth when configured.
        if let Some(ref hot) = self.hot_repo {
            match hot.save(game.clone()).await {
                Ok(()) => {}
                Err(e) if self.redis.required_for_writes => {
                    return Err(AppError::ServiceUnavailable(format!(
                        "Redis unavailable for writes: {e}"
                    )));
                }
                Err(e) => {
                    tracing::warn!(game_id = id, error = %e, "Failed to write updated game to Redis hot store");
                    // Optional fallback: if Redis isn't required, persist to cold store so progress isn't lost.
                    self.repo.save(game.clone()).await?;
                }
            }

            // Best-effort snapshotting to Mongo at most once per interval.
            let now = Utc::now().timestamp();
            if self.redis.snapshot_interval_seconds > 0 {
                match hot
                    .mark_snapshot_if_due(id, now, self.redis.snapshot_interval_seconds)
                    .await
                {
                    Ok(true) => {
                        tracing::info!(game_id = id, "Persisting Mongo snapshot for active game");
                        self.repo.save(game.clone()).await?;
                    }
                    Ok(false) => {}
                    Err(e) => {
                        tracing::warn!(game_id = id, error = %e, "Snapshot gate check failed");
                    }
                }
            }
        } else {
            // No Redis configured: write through to cold store.
            self.repo.add_moves(id, &reveal_points).await?;
        }

        MinesweeperMetrics::record_move(&game);
        Ok(game)
    }

    async fn toggle_flag(
        &self,
        id: i32,
        point: Point,
        user: Option<UserInfo>,
    ) -> AppResult<MinesweeperGame> {
        let mut game = self.fetch_game_for_write(id).await?;
        self.check_ownership(id, user).await?;

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

        if let Some(ref hot) = self.hot_repo {
            match hot.save(game.clone()).await {
                Ok(()) => {}
                Err(e) if self.redis.required_for_writes => {
                    return Err(AppError::ServiceUnavailable(format!(
                        "Redis unavailable for writes: {e}"
                    )));
                }
                Err(e) => {
                    tracing::warn!(game_id = id, error = %e, "Failed to write updated game to Redis hot store");
                    self.repo.save(game.clone()).await?;
                }
            }

            let now = Utc::now().timestamp();
            if self.redis.snapshot_interval_seconds > 0 {
                match hot
                    .mark_snapshot_if_due(id, now, self.redis.snapshot_interval_seconds)
                    .await
                {
                    Ok(true) => {
                        tracing::info!(game_id = id, "Persisting Mongo snapshot for active game");
                        self.repo.save(game.clone()).await?;
                    }
                    Ok(false) => {}
                    Err(e) => {
                        tracing::warn!(game_id = id, error = %e, "Snapshot gate check failed");
                    }
                }
            }
        } else {
            // No Redis configured: write through to cold store.
            if game.is_point_flagged(&point) {
                // We already toggled locally; re-apply in cold store by using existing methods.
                // Since we don't have the previous state, use save() to persist the full game.
                self.repo.save(game.clone()).await?;
            } else {
                self.repo.save(game.clone()).await?;
            }
        }

        Ok(game)
    }

    async fn get_user_games(&self, user: UserInfo) -> AppResult<Vec<MinesweeperGame>> {
        let ids = self.repo.get_game_ids_by_user_id(&user.sub).await?;
        if ids.is_empty() {
            return Ok(vec![]);
        }

        // Prefer active state from Redis when available, but always fall back to cold store.
        let mut games = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(ref hot) = self.hot_repo {
                match hot.get_game_refresh_ttl(id).await {
                    Ok(Some(game)) => {
                        games.push(game);
                        continue;
                    }
                    Ok(None) => {}
                    Err(e) => {
                        tracing::warn!(
                            game_id = id,
                            error = %e,
                            "Redis hot-store read failed while building user games list"
                        );
                    }
                }
            }

            if let Some(game) = self.repo.get_game(id).await? {
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
