use super::GameService;
use crate::engine::BoardEngine;
use crate::error::{AppError, AppResult};
use crate::model::{MinesweeperGame, Point, UserInfo, UserStatsDto};
use crate::repository::MinesweeperRepository;
use crate::telemetry::metrics::MinesweeperMetrics;
use async_trait::async_trait;
use std::sync::Arc;

pub struct MinesweeperService {
    repo: Arc<dyn MinesweeperRepository>,
    engine: Arc<dyn BoardEngine>,
}

impl MinesweeperService {
    pub fn new(repo: Arc<dyn MinesweeperRepository>, engine: Arc<dyn BoardEngine>) -> Self {
        Self { repo, engine }
    }

    async fn fetch_game(&self, id: i32) -> AppResult<MinesweeperGame> {
        if id == 0 {
            return Err(AppError::BadRequest("Game ID is required".to_string()));
        }
        self.repo
            .get_game(id)
            .await?
            .ok_or_else(|| AppError::NotFound(id.to_string()))
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
        let game = MinesweeperGame::new(cols, rows, mines);
        self.repo.save(game.clone()).await?;

        if let Some(user_info) = user {
            self.repo.add_mapping(&user_info.sub, game.id).await?;
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
        self.check_ownership(id, user).await?;
        let mut game = self.fetch_game(id).await?;

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
            self.repo.save(game.clone()).await?;
        }

        let reveal_points = self.engine.get_reveal_points(&game, point);
        let updated_game = self.repo.add_moves(id, &reveal_points).await?;

        match updated_game {
            Some(g) => {
                MinesweeperMetrics::record_move(&g);
                Ok(g)
            }
            None => Err(AppError::NotFound(id.to_string())),
        }
    }

    async fn toggle_flag(
        &self,
        id: i32,
        point: Point,
        user: Option<UserInfo>,
    ) -> AppResult<MinesweeperGame> {
        self.check_ownership(id, user).await?;
        let game = self.fetch_game(id).await?;

        if !game.is_valid_point(&point) {
            return Err(AppError::BadRequest(
                "Coordinates out of bounds".to_string(),
            ));
        }

        if game.is_game_over() || game.is_point_revealed(&point) {
            return Ok(game);
        }

        let updated_game = if game.is_point_flagged(&point) {
            self.repo.remove_flag(id, point).await?
        } else {
            self.repo.add_flag(id, point).await?
        };

        updated_game.ok_or_else(|| AppError::NotFound(id.to_string()))
    }

    async fn get_user_games(&self, user: UserInfo) -> AppResult<Vec<MinesweeperGame>> {
        let game_ids = self.repo.get_game_ids_by_user_id(&user.sub).await?;
        self.repo.get_games_by_ids(&game_ids).await
    }

    async fn get_user_stats(&self, user: UserInfo) -> AppResult<UserStatsDto> {
        let game_ids = self.repo.get_game_ids_by_user_id(&user.sub).await?;
        let games = self.repo.get_games_by_ids(&game_ids).await?;

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
