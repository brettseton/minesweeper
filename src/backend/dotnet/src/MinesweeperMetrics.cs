using System.Diagnostics.Metrics;

namespace backend
{
    public static class MinesweeperMetrics
    {
        public const string MeterName = "Minesweeper.Backend";
        private static readonly Meter Meter = new(MeterName, "1.0.0");

        public static readonly Counter<long> GamesStarted = Meter.CreateCounter<long>(
            "minesweeper.games.started",
            unit: "{game}",
            description: "Number of games started");

        public static readonly Counter<long> GamesWon = Meter.CreateCounter<long>(
            "minesweeper.games.won",
            unit: "{game}",
            description: "Number of games won");

        public static readonly Counter<long> GamesLost = Meter.CreateCounter<long>(
            "minesweeper.games.lost",
            unit: "{game}",
            description: "Number of games lost");

        public static readonly Counter<long> MovesMade = Meter.CreateCounter<long>(
            "minesweeper.moves.made",
            unit: "{move}",
            description: "Number of moves made");
    }
}
