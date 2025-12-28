using System.Collections.Generic;
using System.Linq;
using backend.Model;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Options;
using MongoDB.Driver;

namespace backend.Repository
{
    public class UserGameRepository : IUserGameRepository
    {
        private readonly ILogger _logger;
        private readonly IMongoCollection<UserGameMapping> _mappings;

        public UserGameRepository(
            ILoggerFactory loggingFactory,
            IMongoDatabase database)
        {
            _logger = loggingFactory.CreateLogger<UserGameRepository>();
            _mappings = database.GetCollection<UserGameMapping>("UserGameMappings");
        }

        public void AddMapping(string userId, int gameId)
        {
            _logger.LogInformation($"Adding mapping for user {userId} to game {gameId}");
            var mapping = new UserGameMapping
            {
                Id = $"{userId}_{gameId}",
                UserId = userId,
                GameId = gameId
            };
            _mappings.ReplaceOne(x => x.Id == mapping.Id, mapping, new ReplaceOptions { IsUpsert = true });
        }

        public IEnumerable<int> GetGameIdsByUserId(string userId)
        {
            return _mappings.Find(x => x.UserId == userId)
                            .ToEnumerable()
                            .Select(x => x.GameId);
        }
    }
}

