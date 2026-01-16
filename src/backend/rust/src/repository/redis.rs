use crate::error::{AppError, AppResult};
use crate::model::{MinesweeperGame, Point};
use crate::repository::GameRepository;
use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use redis::Script;

pub struct RedisGameRepository {
    manager: ConnectionManager,
    ttl_seconds: u64,
}

impl RedisGameRepository {
    pub async fn new(redis_uri: &str, ttl_seconds: u64) -> AppResult<Self> {
        let client = redis::Client::open(redis_uri)
            .map_err(|e| AppError::Internal(format!("Failed to connect to Redis: {}", e)))?;
        let manager = ConnectionManager::new(client).await.map_err(|e| {
            AppError::Internal(format!("Failed to create Redis connection manager: {}", e))
        })?;
        Ok(Self {
            manager,
            ttl_seconds,
        })
    }

    fn game_key(id: i32) -> String {
        format!("game:{}", id)
    }

    fn snapshot_key(id: i32) -> String {
        format!("game_snapshot_at:{}", id)
    }

    pub async fn get_game_refresh_ttl(&self, id: i32) -> AppResult<Option<MinesweeperGame>> {
        let mut conn = self.manager.clone();

        let key = Self::game_key(id);
        let val: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let Some(json) = val else {
            return Ok(None);
        };

        // Refresh TTL on read to keep active games alive.
        let _: () = conn
            .expire(&key, self.ttl_seconds as i64)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let game =
            serde_json::from_str(&json).map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(Some(game))
    }

    pub async fn delete_game_keys(&self, id: i32) -> AppResult<()> {
        let mut conn = self.manager.clone();
        let _: () = conn
            .del((Self::game_key(id), Self::snapshot_key(id)))
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Atomically checks whether a Mongo snapshot is due for this game, and records the
    /// snapshot time if so. Returns `true` when a snapshot should be written.
    pub async fn mark_snapshot_if_due(
        &self,
        id: i32,
        now_unix_seconds: i64,
        snapshot_interval_seconds: u64,
    ) -> AppResult<bool> {
        let mut conn = self.manager.clone();

        // Lua: if key missing OR now-last >= interval then set last=now EX ttl and return 1 else 0.
        let script = Script::new(
            r#"
local last = redis.call("GET", KEYS[1])
if (not last) or (tonumber(ARGV[1]) - tonumber(last) >= tonumber(ARGV[2])) then
  redis.call("SET", KEYS[1], ARGV[1], "EX", ARGV[3])
  return 1
end
return 0
"#,
        );

        let res: i32 = script
            .key(Self::snapshot_key(id))
            .arg(now_unix_seconds)
            .arg(snapshot_interval_seconds as i64)
            .arg(self.ttl_seconds as i64)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(res == 1)
    }
}

#[async_trait]
impl GameRepository for RedisGameRepository {
    async fn get_game(&self, id: i32) -> AppResult<Option<MinesweeperGame>> {
        let mut conn = self.manager.clone();
        
        let val: Option<String> = conn.get(Self::game_key(id)).await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        match val {
            Some(json) => {
                let game = serde_json::from_str(&json)
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                Ok(Some(game))
            }
            None => Ok(None),
        }
    }

    async fn get_games_by_ids(&self, ids: &[i32]) -> AppResult<Vec<MinesweeperGame>> {
        let mut conn = self.manager.clone();

        let keys: Vec<String> = ids.iter().map(|id| Self::game_key(*id)).collect();
        if keys.is_empty() {
            return Ok(vec![]);
        }

        let vals: Vec<Option<String>> = conn.mget(&keys).await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let mut games = vec![];
        for val in vals.into_iter().flatten() {
            let game = serde_json::from_str(&val)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            games.push(game);
        }
        Ok(games)
    }

    async fn save(&self, game: MinesweeperGame) -> AppResult<()> {
        let mut conn = self.manager.clone();

        let json = serde_json::to_string(&game)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Store with 24h TTL
        let _: () = conn
            .set_ex(Self::game_key(game.id), json, self.ttl_seconds)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: i32) -> AppResult<()> {
        let mut conn = self.manager.clone();

        let _: () = conn.del(Self::game_key(id)).await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn add_moves(&self, id: i32, points: &[Point]) -> AppResult<Option<MinesweeperGame>> {
        let mut game = match self.get_game(id).await? {
            Some(g) => g,
            None => return Ok(None),
        };

        for p in points {
            game.moves.insert(*p);
        }

        self.save(game.clone()).await?;
        Ok(Some(game))
    }

    async fn add_flag(&self, id: i32, point: Point) -> AppResult<Option<MinesweeperGame>> {
        let mut game = match self.get_game(id).await? {
            Some(g) => g,
            None => return Ok(None),
        };

        game.flag_points.insert(point);
        self.save(game.clone()).await?;
        Ok(Some(game))
    }

    async fn remove_flag(&self, id: i32, point: Point) -> AppResult<Option<MinesweeperGame>> {
        let mut game = match self.get_game(id).await? {
            Some(g) => g,
            None => return Ok(None),
        };

        game.flag_points.remove(&point);
        self.save(game.clone()).await?;
        Ok(Some(game))
    }
}
