using BenchmarkDotNet.Running;

namespace benchmarks
{
    public class Program
    {
        public static void Main(string[] args)
        {
            BenchmarkRunner.Run<GameControllerBenchmarks>();
        }
    }
}