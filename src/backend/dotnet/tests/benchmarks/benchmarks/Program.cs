using BenchmarkDotNet.Running;
using benchmarks.Benchmarks;

namespace benchmarks
{
    public class Program
    {
        public static void Main(string[] args)
        {
            //BenchmarkRunner.Run<GameControllerBenchmarks>();
            BenchmarkRunner.Run<MinesweeperGameBenchmarks>();
        }
    }
}