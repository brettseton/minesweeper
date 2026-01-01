using System.Collections.Generic;
using System.Linq;
using backend.Model;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Options;
using MongoDB.Driver;

namespace backend.Repository
{
    public class GameRepository : IGameRepository
    {
        private readonly ILogger<GameRepository> _logger;
        private readonly IMongoCollection<MinesweeperGame> _entities;

        public GameRepository(
            ILogger<GameRepository> logger,
            IMongoDatabase database)
        {
            _logger = logger;
            _entities = database.GetCollection<MinesweeperGame>("Games");
        }

        public MinesweeperGame AddFlag(int id, Point point)
        {
            _logger.LogInformation("AddFlag: {Point}", point);
            return _entities.FindOneAndUpdate<MinesweeperGame>(
                game => game.Id == id,
                Builders<MinesweeperGame>.Update.AddToSet(x => x.FlagPoints, point),
                new FindOneAndUpdateOptions<MinesweeperGame> { ReturnDocument = ReturnDocument.After }
            )!;
        }

        public MinesweeperGame AddMoves(int id, Point[] points)
        {
            _logger.LogInformation("AddMove: {Points}", (object)points);
            return _entities.FindOneAndUpdate<MinesweeperGame>(
                game => game.Id == id,
                Builders<MinesweeperGame>.Update.AddToSetEach(x => x.Moves, points),
                new FindOneAndUpdateOptions<MinesweeperGame> { ReturnDocument = ReturnDocument.After }
            )!;
        }

        public MinesweeperGame? GetGame(int id)
        {
            return _entities.Find(game => game.Id == id).SingleOrDefault();
        }

        public MinesweeperGame RemoveFlag(int id, Point point)
        {
            _logger.LogInformation("RemoveFlag: {Point}", point);
            return _entities.FindOneAndUpdate<MinesweeperGame>(
                game => game.Id == id,
                Builders<MinesweeperGame>.Update.Pull(x => x.FlagPoints, point),
                new FindOneAndUpdateOptions<MinesweeperGame> { ReturnDocument = ReturnDocument.After }
            )!;
        }

        public void Save(MinesweeperGame entry)
        {
            _entities.InsertOne(entry);
        }

        public IEnumerable<MinesweeperGame> GetGamesByIds(IEnumerable<int> ids)
        {
            return _entities.Find(game => ids.Contains(game.Id)).ToList();
        }
    }
}
