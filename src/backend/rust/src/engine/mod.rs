use crate::model::{BoardState, MinesweeperGame, Point};
use rand::Rng;
use std::collections::{HashSet, VecDeque};

pub trait BoardEngine: Send + Sync {
    fn generate_mines(&self, game: &mut MinesweeperGame, first_click: Point);
    fn get_reveal_points(&self, game: &MinesweeperGame, p: Point) -> Vec<Point>;
}

pub struct MinesweeperEngine;

impl BoardEngine for MinesweeperEngine {
    fn generate_mines(&self, game: &mut MinesweeperGame, first_click: Point) {
        if game.mines_generated {
            return;
        }

        let mut rng = rand::thread_rng();
        let mut mine_points = HashSet::new();

        let mut safe_zone = HashSet::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                let nx = first_click.x as isize + dx;
                let ny = first_click.y as isize + dy;

                if nx >= 0 && nx < game.cols as isize && ny >= 0 && ny < game.rows as isize {
                    safe_zone.insert(Point {
                        x: nx as usize,
                        y: ny as usize,
                    });
                }
            }
        }

        let max_mines = (game.cols * game.rows).saturating_sub(safe_zone.len());
        let mine_count = game.mine_count_target.min(max_mines);

        while mine_points.len() < mine_count {
            let x = rng.gen_range(0..game.cols);
            let y = rng.gen_range(0..game.rows);
            let point = Point { x, y };

            if safe_zone.contains(&point) || !mine_points.insert(point) {
                continue;
            }

            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }

                    let nx = x as isize + dx;
                    let ny = y as isize + dy;

                    if nx >= 0 && nx < game.cols as isize && ny >= 0 && ny < game.rows as isize {
                        let nx = nx as usize;
                        let ny = ny as usize;
                        game.board[nx][ny] = game.board[nx][ny].increment();
                    }
                }
            }
        }

        for p in &mine_points {
            game.board[p.x][p.y] = BoardState::Mine;
        }

        game.mine_points = mine_points;
        game.mines_generated = true;
    }

    fn get_reveal_points(&self, game: &MinesweeperGame, p: Point) -> Vec<Point> {
        if !game.mines_generated {
            return vec![p];
        }

        match game.board[p.x][p.y] {
            BoardState::Zero => self.get_zero_moves(game, p),
            BoardState::Mine => game.mine_points.iter().cloned().collect(),
            _ => vec![p],
        }
    }
}

impl MinesweeperEngine {
    fn get_zero_moves(&self, game: &MinesweeperGame, start: Point) -> Vec<Point> {
        let mut points = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start);
        visited.insert(start);

        while let Some(p) = queue.pop_front() {
            points.push(p);

            if game.board[p.x][p.y] == BoardState::Zero {
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }

                        let nx = p.x as isize + dx;
                        let ny = p.y as isize + dy;

                        if nx >= 0 && nx < game.cols as isize && ny >= 0 && ny < game.rows as isize
                        {
                            let neighbor = Point {
                                x: nx as usize,
                                y: ny as usize,
                            };

                            if visited.insert(neighbor) {
                                queue.push_back(neighbor);
                            }
                        }
                    }
                }
            }
        }

        points
    }
}
