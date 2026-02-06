use crate::error::{AppError, AppResult};
use crate::model::{MinesweeperGame, Point};
use crate::repository::GameRepository;
use crate::repository::MinesweeperGameDocument;
use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use redis::Script;

pub struct RedisGameRepository {
    manager: ConnectionManager,
    ttl_seconds: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimisticSaveResult {
    Saved { new_version: i64 },
    Conflict { current_version: i64 },
}

impl RedisGameRepository {
    fn deserialize_game(json: &str) -> AppResult<MinesweeperGame> {
        match serde_json::from_str::<MinesweeperGame>(json) {
            Ok(game) => Ok(game),
            Err(_) => {
                // Backward-compatible: older Redis values used the Mongo/legacy schema.
                let legacy: MinesweeperGameDocument =
                    serde_json::from_str(json).map_err(|e| AppError::Internal(e.to_string()))?;
                Ok(legacy.into())
            }
        }
    }

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

    fn version_key(id: i32) -> String {
        format!("game_version:{}", id)
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
        let _: () = conn
            .expire(Self::version_key(id), self.ttl_seconds as i64)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let game = Self::deserialize_game(&json)?;
        Ok(Some(game))
    }

    pub async fn get_game_with_version_refresh_ttl(
        &self,
        id: i32,
    ) -> AppResult<Option<(MinesweeperGame, i64)>> {
        let mut conn = self.manager.clone();

        let json_key = Self::game_key(id);
        let version_key = Self::version_key(id);

        let vals: Vec<Option<String>> = conn
            .mget(&[json_key.as_str(), version_key.as_str()])
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let json = vals.first().cloned().flatten();
        let version = vals.get(1).cloned().flatten();

        let Some(json) = json else {
            return Ok(None);
        };

        // Refresh TTL on read to keep active games alive.
        let _: () = conn
            .expire(json_key.as_str(), self.ttl_seconds as i64)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let _: () = conn
            .expire(version_key.as_str(), self.ttl_seconds as i64)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let game = Self::deserialize_game(&json)?;
        let version = version
            .as_deref()
            .unwrap_or("0")
            .parse::<i64>()
            .unwrap_or(0);
        Ok(Some((game, version)))
    }

    pub async fn delete_game_keys(&self, id: i32) -> AppResult<()> {
        let mut conn = self.manager.clone();
        let _: () = conn
            .del((
                Self::game_key(id),
                Self::version_key(id),
                Self::snapshot_key(id),
            ))
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

    pub async fn rehydrate_if_absent(&self, game: &MinesweeperGame) -> AppResult<()> {
        let mut conn = self.manager.clone();
        let json = serde_json::to_string(game).map_err(|e| AppError::Internal(e.to_string()))?;

        // Only populate Redis when the hot key is absent to avoid overwriting newer active state
        // with a potentially stale cold-store snapshot.
        let script = Script::new(
            r#"
if redis.call("EXISTS", KEYS[1]) == 1 then
  return 0
end
redis.call("SET", KEYS[1], ARGV[1], "EX", ARGV[2])
redis.call("SET", KEYS[2], "1", "EX", ARGV[2])
return 1
"#,
        );

        let _: i32 = script
            .key(Self::game_key(game.id))
            .key(Self::version_key(game.id))
            .arg(json)
            .arg(self.ttl_seconds as i64)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn save_if_version(
        &self,
        game: &MinesweeperGame,
        expected_version: i64,
    ) -> AppResult<OptimisticSaveResult> {
        let mut conn = self.manager.clone();
        let json = serde_json::to_string(game).map_err(|e| AppError::Internal(e.to_string()))?;

        // Atomic compare-and-set: only update the JSON blob if the version matches.
        let script = Script::new(
            r#"
local cur = tonumber(redis.call("GET", KEYS[2]) or "0")
if cur ~= tonumber(ARGV[1]) then
  return {0, cur}
end
local newv = cur + 1
redis.call("SET", KEYS[1], ARGV[2], "EX", ARGV[3])
redis.call("SET", KEYS[2], tostring(newv), "EX", ARGV[3])
return {1, newv}
"#,
        );

        let (ok, ver): (i64, i64) = script
            .key(Self::game_key(game.id))
            .key(Self::version_key(game.id))
            .arg(expected_version)
            .arg(json)
            .arg(self.ttl_seconds as i64)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(if ok == 1 {
            OptimisticSaveResult::Saved { new_version: ver }
        } else {
            OptimisticSaveResult::Conflict {
                current_version: ver,
            }
        })
    }
}

#[async_trait]
impl GameRepository for RedisGameRepository {
    async fn get_game(&self, id: i32) -> AppResult<Option<MinesweeperGame>> {
        let mut conn = self.manager.clone();

        let val: Option<String> = conn
            .get(Self::game_key(id))
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        match val {
            Some(json) => {
                let game = Self::deserialize_game(&json)?;
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

        let vals: Vec<Option<String>> = conn
            .mget(&keys)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let mut games = vec![];
        for val in vals.into_iter().flatten() {
            let game = Self::deserialize_game(&val)?;
            games.push(game);
        }
        Ok(games)
    }

    async fn save(&self, game: MinesweeperGame) -> AppResult<()> {
        let mut conn = self.manager.clone();

        let json = serde_json::to_string(&game).map_err(|e| AppError::Internal(e.to_string()))?;

        // Store with TTL. (This method is not concurrency-safe; prefer `save_if_version` for
        // high-concurrency write paths.)
        let _: () = conn
            .set_ex(Self::game_key(game.id), json, self.ttl_seconds)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: i32) -> AppResult<()> {
        let mut conn = self.manager.clone();

        let _: () = conn
            .del(Self::game_key(id))
            .await
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
