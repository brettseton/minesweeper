using System.Collections.Generic;
using System.Linq;
using backend.Model;
using Microsoft.Extensions.Logging;

namespace backend.Repository
{
    public class InMemoryGameRepository : IGameRepository
    {
        private readonly ILogger<InMemoryGameRepository> _logger;
        private readonly Dictionary<int, MinesweeperGame> _entities;

        public InMemoryGameRepository(ILogger<InMemoryGameRepository> logger)
        {
            _logger = logger;
            _entities = new Dictionary<int, MinesweeperGame>();
        }

        public MinesweeperGame AddFlag(int id, Point point)
        {
            _logger.LogInformation("AddFlag: {Point}", point);
            _entities[id].FlagPoints!.Add(point);
            return GetGame(id)!;
        }

        public MinesweeperGame AddMoves(int id, Point[] points)
        {
            _logger.LogInformation("AddMove: {Points}", (object)points);
            _entities[id].Moves!.UnionWith(points);
            return GetGame(id)!;
        }

        public MinesweeperGame? GetGame(int id)
        {
            _logger.LogInformation("Get Game: {Id}", id);
            _logger.LogInformation("Entities: {Keys}", string.Join(", ", _entities.Keys));
            return _entities.GetValueOrDefault(id);
        }

        public MinesweeperGame RemoveFlag(int id, Point point)
        {
            _logger.LogInformation("RemoveFlag: {Point}", point);
            _entities[id].FlagPoints!.Remove(point);
            return GetGame(id)!;
        }

        public void Save(MinesweeperGame entry)
        {
            _logger.LogInformation("Save Game: {Id}", entry.Id);
            _entities[entry.Id] = entry;
        }

        public IEnumerable<MinesweeperGame> GetGamesByIds(IEnumerable<int> ids)
        {
            return _entities.Values.Where(game => ids.Contains(game.Id));
        }
    }
}
