use crate::model::MinesweeperGame;
use opentelemetry::global;
use opentelemetry::metrics::{Counter, Meter};
use std::sync::OnceLock;

pub static METRICS: OnceLock<MinesweeperMetrics> = OnceLock::new();

pub struct MinesweeperMetrics {
    pub games_started: Counter<u64>,
    pub games_won: Counter<u64>,
    pub games_lost: Counter<u64>,
    pub moves_made: Counter<u64>,
}

impl MinesweeperMetrics {
    pub fn new(meter: Meter) -> Self {
        MinesweeperMetrics {
            games_started: meter
                .u64_counter("minesweeper.games.started")
                .with_description("Number of games started")
                .init(),
            games_won: meter
                .u64_counter("minesweeper.games.won")
                .with_description("Number of games won")
                .init(),
            games_lost: meter
                .u64_counter("minesweeper.games.lost")
                .with_description("Number of games lost")
                .init(),
            moves_made: meter
                .u64_counter("minesweeper.moves.made")
                .with_description("Number of moves made")
                .init(),
        }
    }

    pub fn init() {
        let meter = global::meter("Minesweeper.Backend");
        let metrics = MinesweeperMetrics::new(meter);
        let _ = METRICS.set(metrics);
    }

    pub fn get_opt() -> Option<&'static MinesweeperMetrics> {
        METRICS.get()
    }

    pub fn record_game_started() {
        if let Some(m) = Self::get_opt() {
            m.games_started.add(1, &[]);
        }
    }

    pub fn record_move(game: &MinesweeperGame) {
        if let Some(m) = Self::get_opt() {
            m.moves_made.add(1, &[]);
            if game.is_game_won() {
                m.games_won.add(1, &[]);
            } else if game.is_game_lost() {
                m.games_lost.add(1, &[]);
            }
        }
    }
}
