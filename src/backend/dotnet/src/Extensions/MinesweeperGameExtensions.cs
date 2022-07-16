using System;
using System.Collections.Generic;
using System.Linq;
using backend.Model;

namespace backend.Extensions
{
    public static class MinesweeperGameExtensions
    {
        public static IEnumerable<Point> GetZeroMoves(this MinesweeperGame game, Point point)
        {
            var numberOfColumns = game.Board.Length - 1;
            var numberOfRows = game.Board[0].Length - 1;
            var points = new List<Point>((numberOfColumns + 1) * (numberOfRows + 1))
            {
                point
            };

            var visited = new bool[numberOfColumns + 1, numberOfRows + 1];
            visited[point.X, point.Y] = true;
            var i = 0;
            while (i < points.Count)
            {
                var X = points[i].X;
                var Y = points[i].Y;
                if (game.Board[X][Y] == 0)
                {
                    var my = Y > 0 ? Y - 1 : 0;
                    var py = Y < numberOfRows ? Y + 1 : numberOfRows;
                    if (X > 0)
                    {
                        var mx = X - 1;
                        if (!visited[mx, my]) { points.Add(new Point(mx, my)); visited[mx, my] = true; }
                        if (!visited[mx, Y]) { points.Add(new Point(mx, Y)); visited[mx, Y] = true; }
                        if (!visited[mx, py]) { points.Add(new Point(mx, py)); visited[mx, py] = true; }
                    }

                    if (!visited[X, my]) { points.Add(new Point(X, my)); visited[X, my] = true; }
                    if (!visited[X, py]) { points.Add(new Point(X, py)); visited[X, py] = true; }

                    if (X < numberOfColumns)
                    {
                        var px = X + 1;
                        if (!visited[px, my]) { points.Add(new Point(px, my)); visited[px, my] = true; }
                        if (!visited[px, Y]) { points.Add(new Point(px, Y)); visited[px, Y] = true; }
                        if (!visited[px, py]) { points.Add(new Point(px, py)); visited[px, py] = true; }
                    }
                }
                ++i;
            }
            return points;
        }

        public static MinesweeperGame GetNewGame(this MinesweeperGame game, int numberOfColumns = 10, int numberOfRows = 10, int numberOfMines = 10)
        {
            var random = new Random();
            var board = new BoardState[numberOfColumns][];
            var minePoints = new HashSet<Point>();
            for (var x = 0; x < numberOfColumns; ++x)
            {
                board[x] = new BoardState[numberOfRows];
            }
            while (minePoints.Count < numberOfMines)
            {
                var minePoint = new Point() { X = random.Next(numberOfColumns), Y = random.Next(numberOfRows) };
                if (!minePoints.Add(minePoint)) continue;
                // Add 1 to left column surrounding mine
                if (minePoint.X > 0)
                {
                    if (minePoint.Y > 0) board[minePoint.X - 1][minePoint.Y - 1]++;
                    board[minePoint.X - 1][minePoint.Y + 0]++;
                    if (minePoint.Y + 1 < numberOfRows) board[minePoint.X - 1][minePoint.Y + 1]++;
                }

                // Add 1 above and below mine
                if (minePoint.Y > 0) board[minePoint.X][minePoint.Y - 1]++;
                if (minePoint.Y + 1 < numberOfRows) board[minePoint.X][minePoint.Y + 1]++;

                // Add 1 to right column surrounding mine
                if (minePoint.X + 1 < numberOfColumns)
                {
                    if (minePoint.Y > 0) board[minePoint.X + 1][minePoint.Y - 1]++;
                    board[minePoint.X + 1][minePoint.Y + 0]++;
                    if (minePoint.Y + 1 < numberOfRows) board[minePoint.X + 1][minePoint.Y + 1]++;
                }

            }
            foreach (var minePoint in minePoints)
            {
                board[minePoint.X][minePoint.Y] = BoardState.MINE;
            }

            game.Id = random.Next();
            game.Board = board;
            game.Moves = new HashSet<Point>();
            game.MinePoints = minePoints;
            game.FlagPoints = new HashSet<Point>();

            return game;
        }

        public static MinesweeperGameDto ToGameDto(this MinesweeperGame game)
        {
            var numberOfColumns = game.Board.Length;
            var numberOfRows = game.Board[0].Length;

            var gameDto = new MinesweeperGameDto()
            {
                Id = game.Id,
                Board = new BoardState[game.Board.Length][],
                MineCount = game.MineCount,
                FlagPoints = game.FlagPoints
            };

            foreach (var x in Enumerable.Range(0, numberOfColumns))
            {
                gameDto.Board[x] = new BoardState[numberOfRows];
                foreach (var y in Enumerable.Range(0, numberOfRows))
                {
                    gameDto.Board[x][y] = BoardState.UNKNOWN;
                }
            }

            foreach (var flag in game.FlagPoints)
            {
                gameDto.Board[flag.X][flag.Y] = BoardState.FLAG;
            }

            foreach (var move in game.Moves)
            {
                gameDto.Board[move.X][move.Y] = (BoardState)game.Board[move.X][move.Y];
            }

            return gameDto;
        }
    }
}
