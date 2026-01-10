use crate::error::AppResult;
use crate::model::{MinesweeperGame, Point};
use crate::repository::{GameRepository, UserGameRepository};
use async_trait::async_trait;
use mongodb::{
    bson::doc, options::FindOneAndUpdateOptions, options::ReturnDocument, Client, Collection,
};
use tracing::instrument;

pub struct MongoGameRepository {
    collection: Collection<MinesweeperGame>,
    user_games_collection: Collection<UserGameMapping>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct UserGameMapping {
    #[serde(rename = "_id")]
    user_id: String,
    game_ids: Vec<i32>,
}

impl MongoGameRepository {
    pub async fn new(uri: &str, database: &str) -> mongodb::error::Result<Self> {
        let client = Client::with_uri_str(uri).await?;
        let db = client.database(database);
        let collection = db.collection::<MinesweeperGame>("Games");
        let user_games_collection = db.collection::<UserGameMapping>("UserGames");
        Ok(MongoGameRepository {
            collection,
            user_games_collection,
        })
    }

    async fn update_game(
        &self,
        id: i32,
        update: mongodb::bson::Document,
    ) -> AppResult<Option<MinesweeperGame>> {
        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();
        Ok(self
            .collection
            .find_one_and_update(doc! { "_id": id }, update, options)
            .await?)
    }
}

#[async_trait]
impl GameRepository for MongoGameRepository {
    #[instrument(skip(self))]
    async fn get_game(&self, id: i32) -> AppResult<Option<MinesweeperGame>> {
        Ok(self.collection.find_one(doc! { "_id": id }, None).await?)
    }

    #[instrument(skip(self))]
    async fn get_games_by_ids(&self, ids: &[i32]) -> AppResult<Vec<MinesweeperGame>> {
        use futures_util::TryStreamExt;
        let ids_bson = mongodb::bson::to_bson(ids)?;
        let cursor = self
            .collection
            .find(doc! { "_id": { "$in": ids_bson } }, None)
            .await?;
        Ok(cursor.try_collect().await?)
    }

    #[instrument(skip(self, game))]
    async fn save(&self, game: MinesweeperGame) -> AppResult<()> {
        let options = mongodb::options::ReplaceOptions::builder()
            .upsert(true)
            .build();
        self.collection
            .replace_one(doc! { "_id": game.id }, game, options)
            .await?;
        Ok(())
    }

    #[instrument(skip(self, points))]
    async fn add_moves(&self, id: i32, points: &[Point]) -> AppResult<Option<MinesweeperGame>> {
        let points_bson = mongodb::bson::to_bson(points)?;
        let update = doc! { "$addToSet": { "Moves": { "$each": points_bson } } };
        self.update_game(id, update).await
    }

    #[instrument(skip(self))]
    async fn add_flag(&self, id: i32, point: Point) -> AppResult<Option<MinesweeperGame>> {
        let point_bson = mongodb::bson::to_bson(&point)?;
        let update = doc! { "$addToSet": { "FlagPoints": point_bson } };
        self.update_game(id, update).await
    }

    #[instrument(skip(self))]
    async fn remove_flag(&self, id: i32, point: Point) -> AppResult<Option<MinesweeperGame>> {
        let point_bson = mongodb::bson::to_bson(&point)?;
        let update = doc! { "$pull": { "FlagPoints": point_bson } };
        self.update_game(id, update).await
    }
}

#[async_trait]
impl UserGameRepository for MongoGameRepository {
    async fn add_mapping(&self, user_id: &str, game_id: i32) -> AppResult<()> {
        let update = doc! { "$addToSet": { "game_ids": game_id } };
        let options = mongodb::options::FindOneAndUpdateOptions::builder()
            .upsert(true)
            .build();
        self.user_games_collection
            .find_one_and_update(doc! { "_id": user_id }, update, options)
            .await?;
        Ok(())
    }

    async fn get_game_ids_by_user_id(&self, user_id: &str) -> AppResult<Vec<i32>> {
        let mapping = self
            .user_games_collection
            .find_one(doc! { "_id": user_id }, None)
            .await?;
        Ok(mapping.map(|m| m.game_ids).unwrap_or_default())
    }

    async fn get_game_owner(&self, game_id: i32) -> AppResult<Option<String>> {
        let mapping = self
            .user_games_collection
            .find_one(doc! { "game_ids": game_id }, None)
            .await?;
        Ok(mapping.map(|m| m.user_id))
    }
}
