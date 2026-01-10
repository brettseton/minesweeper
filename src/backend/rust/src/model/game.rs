use super::board::{BoardState, Point};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    InProgress,
    Won,
    Lost,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MinesweeperGame {
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

impl MinesweeperGame {
    pub fn new(cols: usize, rows: usize, mines: usize) -> Self {
        use rand::Rng;
        MinesweeperGame {
            id: rand::thread_rng().gen_range(1..i32::MAX),
            board: vec![vec![BoardState::Zero; rows]; cols],
            moves: HashSet::new(),
            mine_points: HashSet::new(),
            flag_points: HashSet::new(),
            created_at: Utc::now(),
            mines_generated: false,
            cols,
            rows,
            mine_count_target: mines,
        }
    }

    pub fn mine_count(&self) -> usize {
        if self.mines_generated {
            self.mine_points.len()
        } else {
            self.mine_count_target
        }
    }

    pub fn is_valid_point(&self, p: &Point) -> bool {
        p.x < self.cols && p.y < self.rows
    }

    pub fn is_point_revealed(&self, p: &Point) -> bool {
        self.moves.contains(p)
    }

    pub fn is_point_flagged(&self, p: &Point) -> bool {
        self.flag_points.contains(p)
    }

    pub fn is_game_won(&self) -> bool {
        if !self.mines_generated || self.is_game_lost() {
            return false;
        }

        let total_cells = self.cols * self.rows;
        self.moves.len() == total_cells - self.mine_points.len()
    }

    pub fn is_game_lost(&self) -> bool {
        if !self.mines_generated {
            return false;
        }

        self.mine_points.iter().any(|p| self.moves.contains(p))
    }

    pub fn is_game_over(&self) -> bool {
        self.is_game_won() || self.is_game_lost()
    }
}
