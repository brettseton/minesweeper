pub mod memory;
pub mod mongo;

use crate::error::AppResult;
use crate::model::{MinesweeperGame, Point};
use async_trait::async_trait;
use std::sync::Arc;

pub use memory::InMemoryGameRepository;
pub use mongo::MongoGameRepository;

#[async_trait]
pub trait GameRepository: Send + Sync {
    async fn get_game(&self, id: i32) -> AppResult<Option<MinesweeperGame>>;
    async fn get_games_by_ids(&self, ids: &[i32]) -> AppResult<Vec<MinesweeperGame>>;
    async fn save(&self, game: MinesweeperGame) -> AppResult<()>;
    async fn add_moves(&self, id: i32, points: &[Point]) -> AppResult<Option<MinesweeperGame>>;
    async fn add_flag(&self, id: i32, point: Point) -> AppResult<Option<MinesweeperGame>>;
    async fn remove_flag(&self, id: i32, point: Point) -> AppResult<Option<MinesweeperGame>>;
}

#[async_trait]
pub trait UserGameRepository: Send + Sync {
    async fn add_mapping(&self, user_id: &str, game_id: i32) -> AppResult<()>;
    async fn get_game_ids_by_user_id(&self, user_id: &str) -> AppResult<Vec<i32>>;
    async fn get_game_owner(&self, game_id: i32) -> AppResult<Option<String>>;
}

pub trait MinesweeperRepository: GameRepository + UserGameRepository {}
impl<T: GameRepository + UserGameRepository> MinesweeperRepository for T {}

pub async fn init_repository(
    settings: &crate::settings::DatabaseSettings,
) -> Arc<dyn MinesweeperRepository> {
    if let Some(ref addr) = settings.addr {
        let mongo_uri = format!("mongodb://{}", addr);
        tracing::info!("Using MongoDB at {}", mongo_uri);
        match MongoGameRepository::new(&mongo_uri, &settings.name).await {
            Ok(r) => Arc::new(r),
            Err(e) => {
                tracing::error!(
                    "Failed to connect to MongoDB: {}, falling back to In-Memory",
                    e
                );
                Arc::new(InMemoryGameRepository::new())
            }
        }
    } else {
        tracing::info!("Using In-Memory Repository");
        Arc::new(InMemoryGameRepository::new())
    }
}
