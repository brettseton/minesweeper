pub mod game;

pub use game::MinesweeperService;

use crate::error::AppResult;
use crate::model::{MinesweeperGame, Point, UserInfo, UserStatsDto};
use async_trait::async_trait;

#[async_trait]
pub trait GameService: Send + Sync {
    async fn get_game(&self, id: i32) -> AppResult<MinesweeperGame>;
    async fn create_game(
        &self,
        cols: usize,
        rows: usize,
        mines: usize,
        user: Option<UserInfo>,
    ) -> AppResult<MinesweeperGame>;
    async fn make_move(
        &self,
        id: i32,
        point: Point,
        user: Option<UserInfo>,
    ) -> AppResult<MinesweeperGame>;
    async fn toggle_flag(
        &self,
        id: i32,
        point: Point,
        user: Option<UserInfo>,
    ) -> AppResult<MinesweeperGame>;
    async fn get_user_games(&self, user: UserInfo) -> AppResult<Vec<MinesweeperGame>>;
    async fn get_user_stats(&self, user: UserInfo) -> AppResult<UserStatsDto>;
}
