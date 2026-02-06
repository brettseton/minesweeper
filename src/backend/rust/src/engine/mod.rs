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
    fn is_diagonal_step_blocked(
        &self,
        game: &MinesweeperGame,
        from: Point,
        dx: isize,
        dy: isize,
    ) -> bool {
        if dx == 0 || dy == 0 {
            return false;
        }

        let side_a = Point {
            x: (from.x as isize + dx) as usize,
            y: from.y,
        };
        let side_b = Point {
            x: from.x,
            y: (from.y as isize + dy) as usize,
        };

        game.is_point_flagged(&side_a) && game.is_point_flagged(&side_b)
    }

    fn get_zero_moves(&self, game: &MinesweeperGame, start: Point) -> Vec<Point> {
        let mut points = Vec::new();
        let mut visited = vec![false; game.cols * game.rows];
        let mut queue = VecDeque::new();

        queue.push_back(start);
        visited[start.x * game.rows + start.y] = true;

        while let Some(p) = queue.pop_front() {
            if game.is_point_flagged(&p) {
                continue;
            }
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
                            let nx = nx as usize;
                            let ny = ny as usize;
                            let idx = nx * game.rows + ny;

                            if !visited[idx] {
                                visited[idx] = true;
                                let next = Point { x: nx, y: ny };
                                if game.is_point_flagged(&next) {
                                    continue;
                                }
                                if self.is_diagonal_step_blocked(game, p, dx, dy) {
                                    continue;
                                }
                                queue.push_back(next);
                            }
                        }
                    }
                }
            }
        }

        points
    }
}

#[cfg(test)]
mod tests {
    use super::{BoardEngine, MinesweeperEngine};
    use crate::model::{BoardState, MinesweeperGame, Point};
    use chrono::Utc;
    use std::collections::HashSet;

    #[test]
    fn zero_wave_does_not_expand_through_flag_barrier() {
        let engine = MinesweeperEngine;
        let mut flag_points = HashSet::new();
        flag_points.insert(Point { x: 0, y: 1 }); // north of center
        flag_points.insert(Point { x: 1, y: 2 }); // east of center
        flag_points.insert(Point { x: 2, y: 1 }); // south of center
        flag_points.insert(Point { x: 1, y: 0 }); // west of center

        let game = MinesweeperGame {
            id: 1,
            board: vec![vec![BoardState::Zero; 3]; 3],
            moves: HashSet::new(),
            mine_points: HashSet::new(),
            flag_points,
            created_at: Utc::now(),
            mines_generated: true,
            cols: 3,
            rows: 3,
            mine_count_target: 0,
        };

        let revealed = engine.get_reveal_points(&game, Point { x: 0, y: 0 });
        assert_eq!(
            revealed,
            vec![Point { x: 0, y: 0 }],
            "zero-wave should stop when flags form a barrier around the center"
        );
    }
}
