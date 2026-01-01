using System;
using System.Collections.Generic;
using System.Data;
using System.Linq;
using backend.Model;
using Dapper;
using Microsoft.Extensions.Logging;
using Npgsql;

namespace backend.Repository
{
    public class PostgresGameRepository : IGameRepository
    {
        private readonly ILogger<PostgresGameRepository> _logger;
        private readonly string _connectionString;

        public PostgresGameRepository(ILogger<PostgresGameRepository> logger, string connectionString)
        {
            _logger = logger;
            _connectionString = connectionString;
            InitializeDatabase();
        }

        private void InitializeDatabase()
        {
            using var connection = new NpgsqlConnection(_connectionString);
            connection.Open();
            connection.Execute(@"
                CREATE TABLE IF NOT EXISTS Games (
                    Id INTEGER PRIMARY KEY,
                    Width INTEGER NOT NULL,
                    Height INTEGER NOT NULL,
                    Board BYTEA NOT NULL,
                    MinePoints BYTEA NOT NULL,
                    Moves BYTEA NOT NULL,
                    Flags BYTEA NOT NULL,
                    CreatedAt TIMESTAMP WITH TIME ZONE NOT NULL
                );
            ");
        }

        public MinesweeperGame? GetGame(int id)
        {
            using var connection = new NpgsqlConnection(_connectionString);
            var row = connection.QuerySingleOrDefault(@"
                SELECT Id, Width, Height, Board, MinePoints, Moves, Flags, CreatedAt 
                FROM Games WHERE Id = @Id", new { Id = id });

            if (row == null) return null;

            return UnpackGame((int)row.id, (int)row.width, (int)row.height, (byte[])row.board, (byte[])row.minepoints, (byte[])row.moves, (byte[])row.flags, (DateTime)row.createdat);
        }

        public void Save(MinesweeperGame entry)
        {
            if (entry.Board == null) throw new ArgumentException("Board cannot be null");
            int width = entry.Board.Length;
            int height = entry.Board[0].Length;

            using var connection = new NpgsqlConnection(_connectionString);
            connection.Execute(@"
                INSERT INTO Games (Id, Width, Height, Board, MinePoints, Moves, Flags, CreatedAt)
                VALUES (@Id, @Width, @Height, @Board, @MinePoints, @Moves, @Flags, @CreatedAt)
                ON CONFLICT (Id) DO UPDATE SET
                    Moves = EXCLUDED.Moves,
                    Flags = EXCLUDED.Flags;",
                new
                {
                    Id = entry.Id,
                    Width = width,
                    Height = height,
                    Board = PackBoard(entry.Board),
                    MinePoints = PackPoints(entry.MinePoints),
                    Moves = PackBitmask(entry.Moves, width, height),
                    Flags = PackBitmask(entry.FlagPoints, width, height),
                    CreatedAt = entry.CreatedAt.ToUniversalTime()
                });
        }

        public MinesweeperGame AddMoves(int id, Point[] points)
        {
            using var connection = new NpgsqlConnection(_connectionString);
            var row = connection.QuerySingle("SELECT Width, Height, Moves FROM Games WHERE Id = @Id", new { Id = id });
            int width = row.width;
            int height = row.height;
            byte[] moves = row.moves;

            foreach (var p in points)
            {
                int bitIndex = p.X * height + p.Y;
                moves[bitIndex / 8] |= (byte)(1 << (7 - (bitIndex % 8)));
            }

            connection.Execute("UPDATE Games SET Moves = @Moves WHERE Id = @Id", new { Id = id, Moves = moves });
            return GetGame(id)!;
        }

        public MinesweeperGame AddFlag(int id, Point point)
        {
            using var connection = new NpgsqlConnection(_connectionString);
            var row = connection.QuerySingle("SELECT Width, Height, Flags FROM Games WHERE Id = @Id", new { Id = id });
            int width = row.width;
            int height = row.height;
            byte[] flags = row.flags;

            int bitIndex = point.X * height + point.Y;
            flags[bitIndex / 8] |= (byte)(1 << (7 - (bitIndex % 8)));

            connection.Execute("UPDATE Games SET Flags = @Flags WHERE Id = @Id", new { Id = id, Flags = flags });
            return GetGame(id)!;
        }

        public MinesweeperGame RemoveFlag(int id, Point point)
        {
            using var connection = new NpgsqlConnection(_connectionString);
            var row = connection.QuerySingle("SELECT Width, Height, Flags FROM Games WHERE Id = @Id", new { Id = id });
            int width = row.width;
            int height = row.height;
            byte[] flags = row.flags;

            int bitIndex = point.X * height + point.Y;
            flags[bitIndex / 8] &= (byte)~(1 << (7 - (bitIndex % 8)));

            connection.Execute("UPDATE Games SET Flags = @Flags WHERE Id = @Id", new { Id = id, Flags = flags });
            return GetGame(id)!;
        }

        public IEnumerable<MinesweeperGame> GetGamesByIds(IEnumerable<int> ids)
        {
            using var connection = new NpgsqlConnection(_connectionString);
            var rows = connection.Query(@"
                SELECT Id, Width, Height, Board, MinePoints, Moves, Flags, CreatedAt 
                FROM Games WHERE Id = ANY(@Ids)", new { Ids = ids.ToArray() });

            return rows.Select(r => UnpackGame((int)r.id, (int)r.width, (int)r.height, (byte[])r.board, (byte[])r.minepoints, (byte[])r.moves, (byte[])r.flags, (DateTime)r.createdat)).ToList();
        }

        #region Packing Helpers

        private byte[] PackBoard(BoardState[][] board)
        {
            int w = board.Length;
            int h = board[0].Length;
            byte[] bytes = new byte[w * h];
            for (int x = 0; x < w; x++)
                for (int y = 0; y < h; y++)
                    bytes[x * h + y] = (byte)(sbyte)board[x][y];
            return bytes;
        }

        private byte[] PackPoints(HashSet<Point>? points)
        {
            if (points == null) return Array.Empty<byte>();
            byte[] bytes = new byte[points.Count * 8];
            int i = 0;
            foreach (var p in points)
            {
                BitConverter.GetBytes(p.X).CopyTo(bytes, i);
                BitConverter.GetBytes(p.Y).CopyTo(bytes, i + 4);
                i += 8;
            }
            return bytes;
        }

        private byte[] PackBitmask(HashSet<Point>? points, int width, int height)
        {
            byte[] bytes = new byte[(width * height + 7) / 8];
            if (points == null) return bytes;
            foreach (var p in points)
            {
                int bitIndex = p.X * height + p.Y;
                bytes[bitIndex / 8] |= (byte)(1 << (7 - (bitIndex % 8)));
            }
            return bytes;
        }

        private MinesweeperGame UnpackGame(int id, int width, int height, byte[] boardBytes, byte[] mineBytes, byte[] moveBytes, byte[] flagBytes, DateTime createdAt)
        {
            var game = new MinesweeperGame
            {
                Id = id,
                CreatedAt = createdAt.ToUniversalTime(),
                Board = new BoardState[width][],
                MinePoints = new HashSet<Point>(),
                Moves = new HashSet<Point>(),
                FlagPoints = new HashSet<Point>()
            };

            for (int x = 0; x < width; x++)
            {
                game.Board[x] = new BoardState[height];
                for (int y = 0; y < height; y++)
                    game.Board[x][y] = (BoardState)(sbyte)boardBytes[x * height + y];
            }

            for (int i = 0; i < mineBytes.Length; i += 8)
            {
                game.MinePoints.Add(new Point(BitConverter.ToInt32(mineBytes, i), BitConverter.ToInt32(mineBytes, i + 4)));
            }

            for (int i = 0; i < width * height; i++)
            {
                if (i / 8 < moveBytes.Length && (moveBytes[i / 8] & (1 << (7 - (i % 8)))) != 0)
                    game.Moves.Add(new Point(i / height, i % height));
                
                if (i / 8 < flagBytes.Length && (flagBytes[i / 8] & (1 << (7 - (i % 8)))) != 0)
                    game.FlagPoints.Add(new Point(i / height, i % height));
            }

            return game;
        }

        #endregion
    }
}