using backend.Model;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Options;
using MongoDB.Driver;

namespace backend.Repository
{
    public class GameRepository : IGameRepository
    {
        private readonly ILogger logger;
        private readonly IMongoCollection<MinesweeperGame> entities;

        public GameRepository(
            ILoggerFactory loggingFactory,
            IOptionsMonitor<MongoConfig> config)
        {
            logger = loggingFactory.CreateLogger<GameRepository>();

            logger.LogInformation($"Trying to connect to {config.CurrentValue.DatabaseAddress}");

            var client = new MongoClient(config.CurrentValue.DatabaseAddress);
            var database = client.GetDatabase("MinesweeperGame");
            entities = database.GetCollection<MinesweeperGame>("Games");
        }

        public MinesweeperGame AddFlag(int id, Point point)
        {
            logger.LogInformation($"AddFlag: {0}", point);
            var result = entities.UpdateOne(game => game.Id == id, Builders<MinesweeperGame>.Update.AddToSet(x => x.FlagPoints, point));
            return GetGame(id);
        }

        public MinesweeperGame AddMoves(int id, Point[] points)
        {

            logger.LogInformation($"AddMove: {0}", points);
            var result = entities.UpdateOne(game => game.Id == id, Builders<MinesweeperGame>.Update.AddToSetEach(x => x.Moves, points));
            return GetGame(id);
        }

        public MinesweeperGame GetGame(int id)
        {
            return entities.Find(game => game.Id == id).SingleOrDefault();
        }

        public MinesweeperGame RemoveFlag(int id, Point point)
        {
            logger.LogInformation($"RemoveFlag: {0}", point);
            var result = entities.UpdateOne(game => game.Id == id, Builders<MinesweeperGame>.Update.Pull(x => x.FlagPoints, point));
            return GetGame(id);
        }

        public void Save(MinesweeperGame entry)
        {
            entities.InsertOne(entry);
        }
    }
}