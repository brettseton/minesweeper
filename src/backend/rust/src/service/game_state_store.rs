use crate::error::{AppError, AppResult};
use crate::model::{MinesweeperGame, Point};
use crate::repository::{GameRepository, MinesweeperRepository, RedisGameRepository};
use crate::settings::RedisSettings;
use chrono::Utc;
use std::sync::Arc;

pub struct GameStateStore {
    reader: GameReader,
    writer: GameWriter,
    index: UserGameIndex,
}

impl GameStateStore {
    pub fn new(
        repo: Arc<dyn MinesweeperRepository>,
        hot_repo: Option<Arc<RedisGameRepository>>,
        redis: RedisSettings,
    ) -> Self {
        let snapshotter = Snapshotter {
            repo: repo.clone(),
            hot_repo: hot_repo.clone(),
            snapshot_interval_seconds: redis.snapshot_interval_seconds,
        };
        Self {
            reader: GameReader {
                repo: repo.clone(),
                hot_repo: hot_repo.clone(),
                required_for_writes: redis.required_for_writes,
            },
            writer: GameWriter {
                repo: repo.clone(),
                hot_repo: hot_repo.clone(),
                required_for_writes: redis.required_for_writes,
                snapshotter,
            },
            index: UserGameIndex { repo, hot_repo },
        }
    }

    pub async fn get_game_owner(&self, game_id: i32) -> AppResult<Option<String>> {
        self.index.get_game_owner(game_id).await
    }

    pub async fn get_game_ids_by_user_id(&self, user_id: &str) -> AppResult<Vec<i32>> {
        self.index.get_game_ids_by_user_id(user_id).await
    }

    pub async fn get_game_from_cold_store(
        &self,
        game_id: i32,
    ) -> AppResult<Option<MinesweeperGame>> {
        self.reader.get_game_from_cold_store(game_id).await
    }

    pub async fn get_game_from_hot_store_refresh_ttl(
        &self,
        game_id: i32,
    ) -> AppResult<Option<MinesweeperGame>> {
        self.reader
            .get_game_from_hot_store_refresh_ttl(game_id)
            .await
    }

    pub async fn fetch_game(&self, id: i32) -> AppResult<MinesweeperGame> {
        self.reader.fetch_game(id).await
    }

    pub async fn fetch_game_for_write_with_hot_version(&self, id: i32) -> AppResult<GameForWrite> {
        self.reader.fetch_game_for_write(id).await
    }

    pub async fn persist_new_game(&self, game: &MinesweeperGame) -> AppResult<()> {
        self.writer.persist_new_game(game).await
    }

    pub async fn associate_user_game(&self, user_id: &str, game_id: i32) -> AppResult<()> {
        self.index.associate_user_game(user_id, game_id).await
    }

    pub async fn persist_moves_without_hot_store(
        &self,
        game_id: i32,
        reveal_points: &[crate::model::Point],
    ) -> AppResult<()> {
        self.writer
            .persist_moves_without_hot_store(game_id, reveal_points)
            .await
    }

    pub async fn persist_toggle_flag_with_hot_version(
        &self,
        game: &MinesweeperGame,
        expected_hot_version: Option<i64>,
    ) -> AppResult<PersistResult> {
        if self.writer.hot_repo.is_some() {
            self.writer
                .persist_active_game_with_hot_version(game, expected_hot_version)
                .await
        } else {
            self.writer.persist_full_without_hot_store(game).await?;
            Ok(PersistResult::Saved)
        }
    }

    pub async fn persist_move_with_hot_version(
        &self,
        game_id: i32,
        game: &MinesweeperGame,
        reveal_points: &[Point],
        is_over: bool,
        expected_hot_version: Option<i64>,
    ) -> AppResult<PersistResult> {
        if is_over {
            return self
                .writer
                .persist_final_game_with_hot_version(game_id, game, expected_hot_version)
                .await;
        }

        if self.writer.hot_repo.is_some() {
            self.writer
                .persist_active_game_with_hot_version(game, expected_hot_version)
                .await
        } else {
            self.writer.persist_full_without_hot_store(game).await?;
            Ok(PersistResult::Saved)
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameForWrite {
    pub game: MinesweeperGame,
    pub hot_version: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistResult {
    Saved,
    Conflict,
}

struct GameReader {
    repo: Arc<dyn MinesweeperRepository>,
    hot_repo: Option<Arc<RedisGameRepository>>,
    required_for_writes: bool,
}

impl GameReader {
    async fn get_game_from_cold_store(&self, game_id: i32) -> AppResult<Option<MinesweeperGame>> {
        self.repo.get_game(game_id).await
    }

    async fn get_game_from_hot_store_refresh_ttl(
        &self,
        game_id: i32,
    ) -> AppResult<Option<MinesweeperGame>> {
        let Some(ref hot) = self.hot_repo else {
            return Ok(None);
        };
        hot.get_game_refresh_ttl(game_id).await
    }

    async fn fetch_game(&self, id: i32) -> AppResult<MinesweeperGame> {
        if id == 0 {
            return Err(AppError::BadRequest("Game ID is required".to_string()));
        }

        // 1) Try Hot Store (Redis)
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
                    tracing::warn!(
                        game_id = id,
                        error = %e,
                        "Redis hot-store read failed; falling back to cold store"
                    );
                }
            }
        }

        // 2) Fallback to Cold Store (Mongo or in-memory)
        let game = self
            .repo
            .get_game(id)
            .await?
            .ok_or_else(|| AppError::NotFound(id.to_string()))?;

        // 3) Rehydrate Hot Store (best effort)
        if let Some(ref hot) = self.hot_repo {
            if let Err(e) = hot.rehydrate_if_absent(&game).await {
                tracing::warn!(game_id = id, error = %e, "Failed to rehydrate Redis from cold store");
            } else {
                tracing::info!(game_id = id, "Rehydrated Redis hot-store from cold store");
            }
        }

        Ok(game)
    }

    async fn fetch_game_for_write(&self, id: i32) -> AppResult<GameForWrite> {
        if id == 0 {
            return Err(AppError::BadRequest("Game ID is required".to_string()));
        }

        if let Some(ref hot) = self.hot_repo {
            // Prefer the hot store for writes to support optimistic concurrency control.
            match hot.get_game_with_version_refresh_ttl(id).await {
                Ok(Some((game, version))) => {
                    return Ok(GameForWrite {
                        game,
                        hot_version: Some(version),
                    })
                }
                Ok(None) => {
                    // Try to rehydrate from cold store, but do not overwrite if Redis already has a newer value.
                    let game = self
                        .repo
                        .get_game(id)
                        .await?
                        .ok_or_else(|| AppError::NotFound(id.to_string()))?;

                    if let Err(e) = hot.rehydrate_if_absent(&game).await {
                        if self.required_for_writes {
                            return Err(AppError::ServiceUnavailable(format!(
                                "Redis unavailable for writes: {e}"
                            )));
                        }
                        tracing::warn!(
                            game_id = id,
                            error = %e,
                            "Failed to rehydrate Redis hot-store from cold store"
                        );
                        return Ok(GameForWrite {
                            game,
                            hot_version: None,
                        });
                    }

                    if let Ok(Some((game, version))) =
                        hot.get_game_with_version_refresh_ttl(id).await
                    {
                        return Ok(GameForWrite {
                            game,
                            hot_version: Some(version),
                        });
                    }

                    if self.required_for_writes {
                        return Err(AppError::ServiceUnavailable(
                            "Redis unavailable for writes".to_string(),
                        ));
                    }

                    return Ok(GameForWrite {
                        game,
                        hot_version: None,
                    });
                }
                Err(e) => {
                    if self.required_for_writes {
                        return Err(AppError::ServiceUnavailable(format!(
                            "Redis unavailable for writes: {e}"
                        )));
                    }
                    tracing::warn!(
                        game_id = id,
                        error = %e,
                        "Redis hot-store read failed for write; falling back to cold store"
                    );
                }
            }
        }

        Ok(GameForWrite {
            game: self.fetch_game(id).await?,
            hot_version: None,
        })
    }
}

struct Snapshotter {
    repo: Arc<dyn MinesweeperRepository>,
    hot_repo: Option<Arc<RedisGameRepository>>,
    snapshot_interval_seconds: u64,
}

impl Snapshotter {
    async fn maybe_snapshot(&self, game: &MinesweeperGame) {
        if self.snapshot_interval_seconds == 0 {
            return;
        }
        let Some(ref hot) = self.hot_repo else {
            return;
        };

        let now = Utc::now().timestamp();
        match hot
            .mark_snapshot_if_due(game.id, now, self.snapshot_interval_seconds)
            .await
        {
            Ok(true) => {
                tracing::info!(
                    game_id = game.id,
                    "Persisting cold-store snapshot for active game"
                );
                if let Err(e) = self.repo.save(game.clone()).await {
                    tracing::warn!(game_id = game.id, error = %e, "Failed to persist cold-store snapshot");
                }
            }
            Ok(false) => {}
            Err(e) => {
                tracing::warn!(game_id = game.id, error = %e, "Snapshot gate check failed");
            }
        }
    }
}

struct GameWriter {
    repo: Arc<dyn MinesweeperRepository>,
    hot_repo: Option<Arc<RedisGameRepository>>,
    required_for_writes: bool,
    snapshotter: Snapshotter,
}

impl GameWriter {
    async fn persist_new_game(&self, game: &MinesweeperGame) -> AppResult<()> {
        if let Some(ref hot) = self.hot_repo {
            if self.required_for_writes {
                tracing::info!(
                    game_id = game.id,
                    "Writing game to Redis hot store (required)"
                );
                match hot.save_if_version(game, 0).await {
                    Ok(crate::repository::redis::OptimisticSaveResult::Saved { .. }) => {}
                    Ok(crate::repository::redis::OptimisticSaveResult::Conflict { .. }) => {
                        return Err(AppError::Internal(
                            "Game ID collision; please retry".to_string(),
                        ));
                    }
                    Err(e) => {
                        return Err(AppError::ServiceUnavailable(format!(
                            "Redis unavailable for writes: {e}"
                        )));
                    }
                }

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
                if let Err(e) = hot.rehydrate_if_absent(game).await {
                    tracing::warn!(game_id = game.id, error = %e, "Failed to write game to Redis hot store");
                }
            }
        } else {
            // No Redis configured: cold store is source of truth.
            self.repo.save(game.clone()).await?;
        }

        Ok(())
    }

    async fn persist_active_game_with_hot_version(
        &self,
        game: &MinesweeperGame,
        expected_hot_version: Option<i64>,
    ) -> AppResult<PersistResult> {
        let Some(ref hot) = self.hot_repo else {
            return Ok(PersistResult::Saved);
        };

        let Some(expected_hot_version) = expected_hot_version else {
            // If we can't do OCC, fall back to the old behavior.
            match hot.save(game.clone()).await {
                Ok(()) => {}
                Err(e) if self.required_for_writes => {
                    return Err(AppError::ServiceUnavailable(format!(
                        "Redis unavailable for writes: {e}"
                    )));
                }
                Err(e) => {
                    tracing::warn!(
                        game_id = game.id,
                        error = %e,
                        "Failed to write updated game to Redis hot store"
                    );
                    self.repo.save(game.clone()).await?;
                }
            }

            self.snapshotter.maybe_snapshot(game).await;
            return Ok(PersistResult::Saved);
        };

        match hot.save_if_version(game, expected_hot_version).await {
            Ok(crate::repository::redis::OptimisticSaveResult::Saved { .. }) => {
                self.snapshotter.maybe_snapshot(game).await;
                Ok(PersistResult::Saved)
            }
            Ok(crate::repository::redis::OptimisticSaveResult::Conflict { .. }) => {
                Ok(PersistResult::Conflict)
            }
            Err(e) if self.required_for_writes => Err(AppError::ServiceUnavailable(format!(
                "Redis unavailable for writes: {e}"
            ))),
            Err(e) => {
                tracing::warn!(
                    game_id = game.id,
                    error = %e,
                    "Failed to write updated game to Redis hot store"
                );
                self.repo.save(game.clone()).await?;
                Ok(PersistResult::Saved)
            }
        }
    }

    async fn persist_moves_without_hot_store(
        &self,
        game_id: i32,
        reveal_points: &[crate::model::Point],
    ) -> AppResult<()> {
        let _ = self.repo.add_moves(game_id, reveal_points).await?;
        Ok(())
    }

    async fn persist_full_without_hot_store(&self, game: &MinesweeperGame) -> AppResult<()> {
        self.repo.save(game.clone()).await
    }

    async fn persist_final_game(&self, game_id: i32, game: &MinesweeperGame) -> AppResult<()> {
        self.repo.save(game.clone()).await?;
        if let Some(ref hot) = self.hot_repo {
            if let Err(e) = hot.delete_game_keys(game_id).await {
                tracing::warn!(
                    game_id = game_id,
                    error = %e,
                    "Failed to delete Redis hot-store keys after game over"
                );
            }
        }
        Ok(())
    }

    async fn persist_final_game_with_hot_version(
        &self,
        game_id: i32,
        game: &MinesweeperGame,
        expected_hot_version: Option<i64>,
    ) -> AppResult<PersistResult> {
        if let Some(ref hot) = self.hot_repo {
            if let Some(expected_hot_version) = expected_hot_version {
                match hot.save_if_version(game, expected_hot_version).await {
                    Ok(crate::repository::redis::OptimisticSaveResult::Saved { .. }) => {}
                    Ok(crate::repository::redis::OptimisticSaveResult::Conflict { .. }) => {
                        return Ok(PersistResult::Conflict);
                    }
                    Err(e) if self.required_for_writes => {
                        return Err(AppError::ServiceUnavailable(format!(
                            "Redis unavailable for writes: {e}"
                        )));
                    }
                    Err(e) => {
                        tracing::warn!(
                            game_id = game_id,
                            error = %e,
                            "Failed to write final game state to Redis hot store"
                        );
                    }
                }
            }
        }

        self.persist_final_game(game_id, game).await?;
        Ok(PersistResult::Saved)
    }
}

struct UserGameIndex {
    repo: Arc<dyn MinesweeperRepository>,
    hot_repo: Option<Arc<RedisGameRepository>>,
}

impl UserGameIndex {
    async fn get_game_owner(&self, game_id: i32) -> AppResult<Option<String>> {
        self.repo.get_game_owner(game_id).await
    }

    async fn get_game_ids_by_user_id(&self, user_id: &str) -> AppResult<Vec<i32>> {
        self.repo.get_game_ids_by_user_id(user_id).await
    }

    async fn associate_user_game(&self, user_id: &str, game_id: i32) -> AppResult<()> {
        if let Err(e) = self.repo.add_mapping(user_id, game_id).await {
            tracing::warn!(
                game_id = game_id,
                user_id = %user_id,
                error = %e,
                "Failed to persist user->game mapping; attempting rollback"
            );
            let _ = self.repo.delete(game_id).await;
            if let Some(ref hot) = self.hot_repo {
                let _ = hot.delete_game_keys(game_id).await;
            }
            return Err(e);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::BoardState;
    use crate::repository::InMemoryGameRepository;
    use chrono::Utc;
    use std::collections::HashSet;

    #[tokio::test]
    async fn non_terminal_move_without_hot_store_persists_full_game_state() {
        let repo: Arc<dyn MinesweeperRepository> = Arc::new(InMemoryGameRepository::new());
        let store = GameStateStore::new(repo.clone(), None, RedisSettings::default());

        let game_id = 4242;
        let reveal_point = Point { x: 0, y: 0 };
        let mine_point = Point { x: 2, y: 2 };

        let initial = MinesweeperGame {
            id: game_id,
            board: vec![vec![BoardState::Zero; 3]; 3],
            moves: HashSet::new(),
            mine_points: HashSet::new(),
            flag_points: HashSet::new(),
            created_at: Utc::now(),
            mines_generated: false,
            cols: 3,
            rows: 3,
            mine_count_target: 1,
        };
        store.persist_new_game(&initial).await.unwrap();

        let mut updated = initial.clone();
        updated.mines_generated = true;
        updated.mine_points.insert(mine_point);
        updated.board[0][1] = BoardState::One;
        updated.moves.insert(reveal_point);

        let result = store
            .persist_move_with_hot_version(game_id, &updated, &[reveal_point], false, None)
            .await
            .unwrap();
        assert_eq!(result, PersistResult::Saved);

        let persisted = repo
            .get_game(game_id)
            .await
            .unwrap()
            .expect("game should exist");

        assert!(persisted.moves.contains(&reveal_point));
        assert!(
            persisted.mines_generated,
            "first reveal metadata should be persisted"
        );
        assert_eq!(
            persisted.mine_points, updated.mine_points,
            "mine placement should not be lost"
        );
        assert_eq!(
            persisted.board, updated.board,
            "board values should reflect generated mines"
        );
    }
}
