using backend.Controllers;
using backend.Model;
using backend.Repository;
using BenchmarkDotNet.Attributes;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Logging;
using MongoDB.Driver;
using Testcontainers.MongoDb;
using Testcontainers.PostgreSql;

namespace benchmarks.Benchmarks
{
    [MemoryDiagnoser]
    public class GameControllerBenchmarks
    {
        private ILoggerFactory _loggerFactory = null!;
        private GameController _gameController = null!;
        private int _testGameId;
        private string? _databaseName;
        private MongoDbContainer? _mongoDbContainer;
        private PostgreSqlContainer? _postgresContainer;

        [Params("InMemory", "Mongo", "Postgres")]
        public string RepositoryType { get; set; } = null!;

        [GlobalSetup]
        public void Setup()
        {
            _loggerFactory = new LoggerFactory();
            IGameRepository repository;

            if (RepositoryType == "Mongo")
            {
                _mongoDbContainer = new MongoDbBuilder()
                    .WithImage("mongo:4")
                    .Build();

                _mongoDbContainer.StartAsync().Wait();

                _databaseName = "MinesweeperBenchmarks_" + Guid.NewGuid().ToString("N");
                var client = new MongoClient(_mongoDbContainer.GetConnectionString());
                var database = client.GetDatabase(_databaseName);
                repository = new GameRepository(_loggerFactory.CreateLogger<GameRepository>(), database);
            }
            else if (RepositoryType == "Postgres")
            {
                _postgresContainer = new PostgreSqlBuilder()
                    .WithImage("postgres:15")
                    .Build();

                _postgresContainer.StartAsync().Wait();

                repository = new PostgresGameRepository(
                    _loggerFactory.CreateLogger<PostgresGameRepository>(),
                    _postgresContainer.GetConnectionString());
            }
            else
            {
                repository = new InMemoryGameRepository(_loggerFactory.CreateLogger<InMemoryGameRepository>());
            }

            _gameController = new GameController(
                _loggerFactory.CreateLogger<GameController>(),
                repository);

            // Seed a game for Get/Post/ToggleFlag benchmarks
            var actionResult = _gameController.New(10, 10, 10);
            if (actionResult.Result is OkObjectResult okResult && okResult.Value is MinesweeperGameDto gameDto)
            {
                _testGameId = gameDto.Id;
            }
        }

        [GlobalCleanup]
        public void Cleanup()
        {
            if (RepositoryType == "Mongo" && _mongoDbContainer != null)
            {
                _mongoDbContainer.StopAsync().Wait();
                _mongoDbContainer.DisposeAsync().AsTask().Wait();
            }
            else if (RepositoryType == "Postgres" && _postgresContainer != null)
            {
                _postgresContainer.StopAsync().Wait();
                _postgresContainer.DisposeAsync().AsTask().Wait();
            }
            _loggerFactory?.Dispose();
        }

        [Benchmark]
        public void CreateNewGame()
        {
            _gameController.New();
        }

        [Benchmark]
        public void GetExistingGame()
        {
            _gameController.Get(_testGameId);
        }

        [Benchmark]
        public void MakeMove()
        {
            _gameController.Post(_testGameId, new Point(1, 1));
        }

        [Benchmark]
        public void ToggleFlag()
        {
            _gameController.ToggleFlag(_testGameId, new Point(2, 2));
        }
    }
}