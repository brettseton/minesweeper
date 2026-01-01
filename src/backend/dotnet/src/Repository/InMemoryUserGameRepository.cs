using System.Collections.Generic;
using System.Linq;
using backend.Model;
using Microsoft.Extensions.Logging;

namespace backend.Repository
{
    public class InMemoryUserGameRepository : IUserGameRepository
    {
        private readonly ILogger<InMemoryUserGameRepository> _logger;
        private readonly Dictionary<string, UserGameMapping> _mappings = new Dictionary<string, UserGameMapping>();

        public InMemoryUserGameRepository(ILogger<InMemoryUserGameRepository> logger)
        {
            _logger = logger;
        }

        public void AddMapping(string userId, int gameId)
        {
            var id = $"{userId}_{gameId}";
            _logger.LogInformation("Adding in-memory mapping for user {UserId} to game {GameId}", userId, gameId);

            _mappings[id] = new UserGameMapping
            {
                Id = id,
                UserId = userId,
                GameId = gameId
            };
        }

        public IEnumerable<int> GetGameIdsByUserId(string userId)
        {
            return _mappings.Values
                            .Where(m => m.UserId == userId)
                            .Select(m => m.GameId);
        }
    }
}

