use crate::model::{BoardState, MinesweeperGame, Point};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Persistence representation for a game (MongoDB + legacy Redis JSON).
///
/// This intentionally keeps the existing storage schema (PascalCase fields and `_id`).
/// Domain types (`model::MinesweeperGame`) should remain persistence-agnostic.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MinesweeperGameDocument {
    #[serde(rename = "_id")]
    pub id: i32,
    pub board: Vec<Vec<BoardState>>,
    pub moves: HashSet<Point>,
    pub mine_points: HashSet<Point>,
    pub flag_points: HashSet<Point>,
    pub created_at: DateTime<Utc>,
    pub mines_generated: bool,
    pub cols: usize,
    pub rows: usize,
    pub mine_count_target: usize,
}

impl From<MinesweeperGameDocument> for MinesweeperGame {
    fn from(doc: MinesweeperGameDocument) -> Self {
        MinesweeperGame {
            id: doc.id,
            board: doc.board,
            moves: doc.moves,
            mine_points: doc.mine_points,
            flag_points: doc.flag_points,
            created_at: doc.created_at,
            mines_generated: doc.mines_generated,
            cols: doc.cols,
            rows: doc.rows,
            mine_count_target: doc.mine_count_target,
        }
    }
}

impl From<&MinesweeperGame> for MinesweeperGameDocument {
    fn from(game: &MinesweeperGame) -> Self {
        Self {
            id: game.id,
            board: game.board.clone(),
            moves: game.moves.clone(),
            mine_points: game.mine_points.clone(),
            flag_points: game.flag_points.clone(),
            created_at: game.created_at,
            mines_generated: game.mines_generated,
            cols: game.cols,
            rows: game.rows,
            mine_count_target: game.mine_count_target,
        }
    }
}
