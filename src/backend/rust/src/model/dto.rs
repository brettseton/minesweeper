use super::board::{BoardState, Point};
use super::game::{GameStatus, MinesweeperGame};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MinesweeperGameDto {
    pub id: i32,
    pub board: Vec<Vec<BoardState>>,
    pub mine_count: usize,
    pub flag_points: HashSet<Point>,
    pub status: GameStatus,
    pub created_at: DateTime<Utc>,
}

impl From<&MinesweeperGame> for MinesweeperGameDto {
    fn from(game: &MinesweeperGame) -> Self {
        let mut dto_board = vec![vec![BoardState::Unknown; game.rows]; game.cols];

        // Apply Flags
        for flag in &game.flag_points {
            dto_board[flag.x][flag.y] = BoardState::Flag;
        }

        // Apply Moves
        for mv in &game.moves {
            dto_board[mv.x][mv.y] = game.board[mv.x][mv.y];
        }

        let status = if game.is_game_lost() {
            GameStatus::Lost
        } else if game.is_game_won() {
            GameStatus::Won
        } else {
            GameStatus::InProgress
        };

        MinesweeperGameDto {
            id: game.id,
            board: dto_board,
            mine_count: game.mine_count(),
            flag_points: game.flag_points.clone(),
            status,
            created_at: game.created_at,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MakeMoveRequest {
    pub x: usize,
    pub y: usize,
    #[serde(rename = "gameId")]
    pub game_id: Option<i32>,
}
