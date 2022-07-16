using backend.Extensions;
using backend.Model;
using BenchmarkDotNet.Attributes;

namespace benchmarks.Benchmarks
{
    [MemoryDiagnoser]
    public class MinesweeperGameBenchmarks
    {
        [Benchmark]
        public void GetNewGame()
        {
            var game = (new MinesweeperGame()).GetNewGame();
        }

        [Benchmark]
        public void GetNewGame_100x100()
        {
            var game = (new MinesweeperGame()).GetNewGame(100, 100, 1000);
        }

        [Benchmark]
        public void GetZeroMoves_10x10()
        {

            var game = oneMine10x10.GetZeroMoves(new Point() { X = 2, Y = 2 });
        }

        [Benchmark]
        public void GetZeroMoves_100x100()
        {

            var game = oneMine100x100.GetZeroMoves(new Point() { X = 2, Y = 2 });
        }

        [Benchmark]
        public void GetZeroMoves_1000x1000()
        {

            var game = oneMine1000x1000.GetZeroMoves(new Point() { X = 2, Y = 2 });
        }



        private readonly MinesweeperGame oneMine10x10 = new MinesweeperGame()
        {
            Id = 1,
            Board = Enumerable.Range(0, 10).Select(y =>
            {
                return Enumerable.Range(0, 10).Select(x => (y == 0 && x == 0) ? BoardState.MINE : (y <= 1 && x <= 1) ? BoardState.ONE : BoardState.ZERO).ToArray();

            }).ToArray(),
            Moves = new HashSet<Point>(),
            MinePoints = new HashSet<Point>() { new Point() { X = 0, Y = 0 } },
            FlagPoints = new HashSet<Point>()
        };

        private readonly MinesweeperGame oneMine100x100 = new MinesweeperGame()
        {
            Id = 1,
            Board = Enumerable.Range(0, 100).Select(y =>
            {
                return Enumerable.Range(0, 100).Select(x => (y == 0 && x == 0) ? BoardState.MINE : (y <= 1 && x <= 1) ? BoardState.ONE : BoardState.ZERO).ToArray();

            }).ToArray(),
            Moves = new HashSet<Point>(),
            MinePoints = new HashSet<Point>() { new Point() { X = 0, Y = 0 } },
            FlagPoints = new HashSet<Point>()
        };

        private readonly MinesweeperGame oneMine1000x1000 = new MinesweeperGame()
        {
            Id = 1,
            Board = Enumerable.Range(0, 1000).Select(y =>
            {
                return Enumerable.Range(0, 1000).Select(x => (y == 0 && x == 0) ? BoardState.MINE : (y <= 1 && x <= 1) ? BoardState.ONE : BoardState.ZERO).ToArray();

            }).ToArray(),
            Moves = new HashSet<Point>(),
            MinePoints = new HashSet<Point>() { new Point() { X = 0, Y = 0 } },
            FlagPoints = new HashSet<Point>()
        };
    }
}
