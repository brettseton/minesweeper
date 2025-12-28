using System.Collections.Generic;
using System.Linq;
using backend.Model;
using Microsoft.Extensions.Logging;

namespace backend.Repository
{
    public class InMemoryGameRepository : IGameRepository
    {
        private readonly ILogger logger;
        private readonly Dictionary<int, MinesweeperGame> _entities;

        public InMemoryGameRepository(ILoggerFactory loggingFactory)
        {
            logger = loggingFactory.CreateLogger<InMemoryGameRepository>();
            _entities = new Dictionary<int, MinesweeperGame>();
        }

        public MinesweeperGame AddFlag(int id, Point point)
        {
            logger.LogInformation($"AddFlag: {0}", point);
            var result = _entities[id].FlagPoints.Add(point);
            return GetGame(id);
        }

        public MinesweeperGame AddMoves(int id, Point[] points)
        {

            logger.LogInformation($"AddMove: {0}", points);
            _entities[id].Moves.UnionWith(points);
            return GetGame(id);
        }

        public MinesweeperGame GetGame(int id)
        {
            logger.LogInformation($"Get Game: {id}");
            logger.LogInformation($"Entities: {string.Join(", ", _entities.Keys)}");
            return _entities.GetValueOrDefault(id) ?? null;
        }

        public MinesweeperGame RemoveFlag(int id, Point point)
        {
            logger.LogInformation($"RemoveFlag: {0}", point);
            var result = _entities[id].FlagPoints.Remove(point);
            return GetGame(id);
        }

        public void Save(MinesweeperGame entry)
        {
            logger.LogInformation($"Save Game: {entry.Id}");
            _entities[entry.Id] = entry;
        }

        public IEnumerable<MinesweeperGame> GetGamesByIds(IEnumerable<int> ids)
        {
            return _entities.Values.Where(game => ids.Contains(game.Id));
        }
    }
}