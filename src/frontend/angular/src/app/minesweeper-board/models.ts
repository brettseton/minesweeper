export interface MinesweeperGame {
  id: number;
  board: number[][];
  mineCount: number;
  flagPoints: Point[];
}

export interface Point {
  x: number;
  y: number;
  gameId?: number;
}

export enum GameState {
  Running,
  MineHit,
  Victory
}

export enum BoardCell {
  UNKNOWN = -1,
  MINE = -2,
  FLAG = -3,
  ZERO = 0
}
