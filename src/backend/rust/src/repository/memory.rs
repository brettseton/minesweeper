use crate::error::{AppError, AppResult};
use crate::model::{MinesweeperGame, Point};
use crate::repository::{GameRepository, UserGameRepository};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct InMemoryGameRepository {
    games: Arc<RwLock<HashMap<i32, MinesweeperGame>>>,
    user_games: Arc<RwLock<HashMap<String, Vec<i32>>>>,
}

impl Default for InMemoryGameRepository {
    fn default() -> Self {
        InMemoryGameRepository {
            games: Arc::new(RwLock::new(HashMap::new())),
            user_games: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl InMemoryGameRepository {
    pub fn new() -> Self {
        Self::default()
    }

    fn update_game<F>(&self, id: i32, f: F) -> AppResult<Option<MinesweeperGame>>
    where
        F: FnOnce(&mut MinesweeperGame),
    {
        let mut games = self
            .games
            .write()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(game) = games.get_mut(&id) {
            f(game);
            Ok(Some(game.clone()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl GameRepository for InMemoryGameRepository {
    async fn get_game(&self, id: i32) -> AppResult<Option<MinesweeperGame>> {
        let games = self
            .games
            .read()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(games.get(&id).cloned())
    }

    async fn get_games_by_ids(&self, ids: &[i32]) -> AppResult<Vec<MinesweeperGame>> {
        let games = self
            .games
            .read()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(ids.iter().filter_map(|id| games.get(id).cloned()).collect())
    }

    async fn save(&self, game: MinesweeperGame) -> AppResult<()> {
        let mut games = self
            .games
            .write()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        games.insert(game.id, game);
        Ok(())
    }

    async fn add_moves(&self, id: i32, points: &[Point]) -> AppResult<Option<MinesweeperGame>> {
        self.update_game(id, |game| {
            for p in points {
                game.moves.insert(*p);
            }
        })
    }

    async fn add_flag(&self, id: i32, point: Point) -> AppResult<Option<MinesweeperGame>> {
        self.update_game(id, |game| {
            game.flag_points.insert(point);
        })
    }

    async fn remove_flag(&self, id: i32, point: Point) -> AppResult<Option<MinesweeperGame>> {
        self.update_game(id, |game| {
            game.flag_points.remove(&point);
        })
    }
}

#[async_trait]
impl UserGameRepository for InMemoryGameRepository {
    async fn add_mapping(&self, user_id: &str, game_id: i32) -> AppResult<()> {
        let mut user_games = self
            .user_games
            .write()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        user_games
            .entry(user_id.to_string())
            .or_default()
            .push(game_id);
        Ok(())
    }

    async fn get_game_ids_by_user_id(&self, user_id: &str) -> AppResult<Vec<i32>> {
        let user_games = self
            .user_games
            .read()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(user_games.get(user_id).cloned().unwrap_or_default())
    }

    async fn get_game_owner(&self, game_id: i32) -> AppResult<Option<String>> {
        let user_games = self
            .user_games
            .read()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        
        for (user_id, games) in user_games.iter() {
            if games.contains(&game_id) {
                return Ok(Some(user_id.clone()));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::BoardState;
    use chrono::Utc;
    use std::collections::HashSet;

    #[tokio::test]
    async fn test_in_memory_repo() {
        let repo = InMemoryGameRepository::new();
        let game = MinesweeperGame {
            id: 123,
            board: vec![vec![BoardState::Zero; 10]; 10],
            moves: HashSet::new(),
            mine_points: HashSet::new(),
            flag_points: HashSet::new(),
            created_at: Utc::now(),
            mines_generated: true,
            cols: 10,
            rows: 10,
            mine_count_target: 10,
        };

        repo.save(game.clone()).await.unwrap();
        let retrieved = repo.get_game(123).await.unwrap().unwrap();
        assert_eq!(retrieved.id, 123);

        let p = Point { x: 1, y: 1 };
        let updated = repo.add_moves(123, &[p]).await.unwrap().unwrap();
        assert!(updated.moves.contains(&p));

        let updated = repo.add_flag(123, p).await.unwrap().unwrap();
        assert!(updated.flag_points.contains(&p));

        let updated = repo.remove_flag(123, p).await.unwrap().unwrap();
        assert!(!updated.flag_points.contains(&p));
    }
}
