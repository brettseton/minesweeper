using System.Collections.Generic;
using System.Linq;
using backend.Model;
using Dapper;
using Microsoft.Extensions.Logging;
using Npgsql;

namespace backend.Repository
{
    public class PostgresUserGameRepository : IUserGameRepository
    {
        private readonly ILogger<PostgresUserGameRepository> _logger;
        private readonly string _connectionString;

        public PostgresUserGameRepository(ILogger<PostgresUserGameRepository> logger, string connectionString)
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
                CREATE TABLE IF NOT EXISTS UserGameMappings (
                    Id TEXT PRIMARY KEY,
                    UserId TEXT,
                    GameId INTEGER
                );
                CREATE INDEX IF NOT EXISTS idx_usergamemappings_userid ON UserGameMappings(UserId);
            ");
        }

        public void AddMapping(string userId, int gameId)
        {
            _logger.LogInformation($"Adding mapping for user {userId} to game {gameId}");
            var id = $"{userId}_{gameId}";
            using var connection = new NpgsqlConnection(_connectionString);
            var sql = @"
                INSERT INTO UserGameMappings (Id, UserId, GameId)
                VALUES (@Id, @UserId, @GameId)
                ON CONFLICT (Id) DO NOTHING;";
            connection.Execute(sql, new { Id = id, UserId = userId, GameId = gameId });
        }

        public IEnumerable<int> GetGameIdsByUserId(string userId)
        {
            using var connection = new NpgsqlConnection(_connectionString);
            var sql = "SELECT GameId FROM UserGameMappings WHERE UserId = @UserId";
            return connection.Query<int>(sql, new { UserId = userId });
        }
    }
}
