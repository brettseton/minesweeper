using backend.Controllers;
using backend.Repository;
using BenchmarkDotNet.Attributes;
using Microsoft.Extensions.Logging;

namespace benchmarks
{
    [MemoryDiagnoser]
    public class GameControllerBenchmarks
    {
        private static readonly GameController _gameController = new(new LoggerFactory(), new InMemoryGameRepository(new LoggerFactory()));

        [Benchmark]
        public void CreateNewGame()
        {
            var result = _gameController.New();
        }
    }
}