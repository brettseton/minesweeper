use criterion::{criterion_group, criterion_main, Criterion};
use rust_backend::engine::{BoardEngine, MinesweeperEngine};
use rust_backend::model::{BoardState, MinesweeperGame, Point};
use std::collections::HashSet;

fn engine_benchmarks(c: &mut Criterion) {
    let engine = MinesweeperEngine;

    let mut group = c.benchmark_group("Minesweeper Engine");

    group.bench_function("new_game_10x10_10", |b| {
        b.iter(|| MinesweeperGame::new(10, 10, 10));
    });

    group.bench_function("new_game_100x100_1000", |b| {
        b.iter(|| MinesweeperGame::new(100, 100, 1000));
    });

    group.bench_function("generate_mines_10x10_10", |b| {
        b.iter(|| {
            let mut game = MinesweeperGame::new(10, 10, 10);
            engine.generate_mines(&mut game, Point { x: 0, y: 0 });
        });
    });

    group.bench_function("generate_mines_100x100_1000", |b| {
        b.iter(|| {
            let mut game = MinesweeperGame::new(100, 100, 1000);
            engine.generate_mines(&mut game, Point { x: 0, y: 0 });
        });
    });

    // GetZeroMoves benchmarks
    let game_10x10 = setup_zero_moves_game(10, 10);
    group.bench_function("get_zero_moves_10x10", |b| {
        b.iter(|| engine.get_reveal_points(&game_10x10, Point { x: 2, y: 2 }));
    });

    let game_100x100 = setup_zero_moves_game(100, 100);
    group.bench_function("get_zero_moves_100x100", |b| {
        b.iter(|| engine.get_reveal_points(&game_100x100, Point { x: 2, y: 2 }));
    });

    let game_1000x1000 = setup_zero_moves_game(1000, 1000);
    group.bench_function("get_zero_moves_1000x1000", |b| {
        b.iter(|| engine.get_reveal_points(&game_1000x1000, Point { x: 2, y: 2 }));
    });

    group.finish();
}

fn setup_zero_moves_game(cols: usize, rows: usize) -> MinesweeperGame {
    let mut game = MinesweeperGame::new(cols, rows, 1);
    // Manually setup a game where (0,0) is a mine and everything else is safe/zero
    game.board = vec![vec![BoardState::Zero; rows]; cols];
    game.board[0][0] = BoardState::Mine;
    game.mine_points = HashSet::new();
    game.mine_points.insert(Point { x: 0, y: 0 });
    game.mines_generated = true;

    // Update neighbors of (0,0)
    for dx in -1..=1 {
        for dy in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = 0 as isize + dx;
            let ny = 0 as isize + dy;
            if nx >= 0 && nx < cols as isize && ny >= 0 && ny < rows as isize {
                game.board[nx as usize][ny as usize] = BoardState::One;
            }
        }
    }
    game
}

criterion_group!(benches, engine_benchmarks);
criterion_main!(benches);
