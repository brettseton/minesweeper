using backend.Model;
using backend.Repository;
using FluentAssertions;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.DependencyInjection;

namespace unittests
{
    public class MongoGameControllerTests : ControllerTestsBase, IClassFixture<MongoDBWebApiFactory>
    {
        public MongoGameControllerTests(MongoDBWebApiFactory apiFactory) : base(apiFactory.CreateClient(), apiFactory.Services.GetRequiredService<IGameRepository>())
        {
        }
    }

    public class InMemoryGameControllerTests : ControllerTestsBase, IClassFixture<InMemoryWebApiFactory>
    {
        public InMemoryGameControllerTests(InMemoryWebApiFactory apiFactory) : base(apiFactory.CreateClient(), apiFactory.Services.GetRequiredService<IGameRepository>())
        {
        }
    }

    public abstract class ControllerTestsBase
    {
        private readonly HttpClient _client;
        private readonly IGameRepository _repository;

        public ControllerTestsBase(HttpClient client, IGameRepository repository)
        {
            _client = client;
            _repository = repository;
        }

        [Fact]
        public async Task CreateNewGame_MatchesGetCallAsync()
        {
            var newGame = await _client.GetAsync<MinesweeperGameDto>("game/new/10/100/10");
            var game = await _client.GetAsync<MinesweeperGameDto>($"game/{newGame.Id}");

            newGame.Should().BeEquivalentTo(game);

            Assert.Equal(10, newGame.Board.Length);
            for (var x = 0; x < newGame.Board.Length; x++)
            {
                Assert.Equal(100, newGame.Board[x].Length);
                for (var y = 0; y < newGame.Board[x].Length; y++)
                    Assert.Equal(BoardState.UNKNOWN, newGame.Board[x][y]);
            }
            Assert.Equal(10, newGame.MineCount);
            Assert.Empty(newGame.FlagPoints);
        }

        [Fact]
        public async Task ToggleFlagOnAndOff_ReturnsCorrectBoardStateAsync()
        {
            var newGame = await _client.GetAsync<MinesweeperGameDto>("game/new/10/10/10");

            var game = await _client.PostAsJsonAsync<Point, MinesweeperGameDto>($"game/flag/{newGame.Id}", new Point(0, 0));

            Assert.Equal(BoardState.FLAG, game.Board[0][0]);
            Assert.Single(game.FlagPoints);
            game = await _client.PostAsJsonAsync<Point, MinesweeperGameDto>($"game/flag/{newGame.Id}", new Point(0, 0));
            for (var x = 0; x < game.Board.Length; x++)
                for (var y = 0; y < game.Board[0].Length; y++)
                    Assert.Equal(BoardState.UNKNOWN, game.Board[x][y]);
        }

        [Fact]
        public async Task ClickOnFlag_ReturnsCorrectBoardStateAsync()
        {
            var newGame = await _client.GetAsync<MinesweeperGameDto>("game/new/10/10/5");

            var repositoryGame = _repository.GetGame(newGame.Id);
            var numberedPoint = GetNumberPoint(repositoryGame.Board);

            var gameFlagToggled = await _client.PostAsJsonAsync<Point, MinesweeperGameDto>($"game/flag/{newGame.Id}", numberedPoint);
            var game = await _client.PostAsJsonAsync<Point, MinesweeperGameDto>($"game/{newGame.Id}", numberedPoint);

            gameFlagToggled.Should().BeEquivalentTo(game);
        }

        [Fact]
        public async Task MoveOnNumberedSpace_ReturnsCorrectBoardStateAsync()
        {
            var newGame = await _client.GetAsync<MinesweeperGameDto>("game/new/10/10/5");
            var repositoryGame = _repository.GetGame(newGame.Id);
            var numberedPoint = GetNumberPoint(repositoryGame.Board);

            var game = await _client.PostAsJsonAsync<Point, MinesweeperGameDto>($"game/{newGame.Id}", numberedPoint);
            Assert.InRange(game.Board[numberedPoint.X][numberedPoint.Y], BoardState.ONE, BoardState.EIGHT);
        }

        [Fact]
        public async Task MoveOnMineSpace_ReturnsCorrectBoardStateAsync()
        {
            var newGame = await _client.GetAsync<MinesweeperGameDto>("game/new/10/10/5");
            var repositoryGame = _repository.GetGame(newGame.Id);
            var minePoint = GetMinePoint(repositoryGame.Board);

            var game = await _client.PostAsJsonAsync<Point, MinesweeperGameDto>($"game/{newGame.Id}", minePoint);

            Assert.Equal(BoardState.MINE, game.Board[minePoint.X][minePoint.Y]);
            foreach (var mine in repositoryGame.MinePoints)
            {
                Assert.Equal(BoardState.MINE, game.Board[mine.X][mine.Y]);
            }
        }

        [Fact]
        public async Task MoveAfterGameOver_DoesntChangeBoardStateAsync()
        {
            var newGame = await _client.GetAsync<MinesweeperGameDto>("game/new/10/10/5");
            var repositoryGame = _repository.GetGame(newGame.Id);
            var minePoint = GetMinePoint(repositoryGame.Board);

            var gameOver = await _client.PostAsJsonAsync<Point, MinesweeperGameDto>($"game/{newGame.Id}", minePoint);

            var numberedPoint = GetNumberPoint(repositoryGame.Board);

            var game = await _client.PostAsJsonAsync<Point, MinesweeperGameDto>($"game/{newGame.Id}", numberedPoint);

            gameOver.Should().BeEquivalentTo(game);

        }

        [Fact]
        public async Task MoveAfterVictory_DoesntChangeBoardStateAsync()
        {
            var newGame = await _client.GetAsync<MinesweeperGameDto>("game/new/10/10/5");
            var repositoryGame = _repository.GetGame(newGame.Id);
            var numberedPoints = GetAllNumberPoints(repositoryGame.Board);
            foreach (var numberedPoint in numberedPoints)
            {
                await _client.PostAsJsonAsync<Point, MinesweeperGameDto>($"game/{newGame.Id}", numberedPoint);
            }
            var victoryGame = await _client.GetAsync<MinesweeperGameDto>($"game/{newGame.Id}");

            var minePoint = GetMinePoint(repositoryGame.Board);
            var game = await _client.PostAsJsonAsync<Point, MinesweeperGameDto>($"game/{newGame.Id}", minePoint);

            victoryGame.Should().BeEquivalentTo(game);
        }

        [Fact]
        public async Task ToggleFlagAfterGameOver_DoesntChangeBoardStateAsync()
        {
            var newGame = await _client.GetAsync<MinesweeperGameDto>("game/new/10/10/5");
            var repositoryGame = _repository.GetGame(newGame.Id);
            var minePoint = GetMinePoint(repositoryGame.Board);
            var gameOver = await _client.PostAsJsonAsync<Point, MinesweeperGameDto>($"game/{newGame.Id}", minePoint);

            var numberedPoint = GetNumberPoint(repositoryGame.Board);
            var game = await _client.PostAsJsonAsync<Point, MinesweeperGameDto>($"game/{newGame.Id}", numberedPoint);

            gameOver.Should().BeEquivalentTo(game);
        }

        [Fact]
        public async Task ToggleFlagAfterVictory_DoesntChangeBoardStateAsync()
        {
            var newGame = await _client.GetAsync<MinesweeperGameDto>("game/new/10/10/5");
            var repositoryGame = _repository.GetGame(newGame.Id);
            var numberedPoints = GetAllNumberPoints(repositoryGame.Board);
            foreach (var numberedPoint in numberedPoints)
            {
                await _client.PostAsJsonAsync<Point, MinesweeperGameDto>($"game/{newGame.Id}", numberedPoint);
            }
            var victoryGame = await _client.GetAsync<MinesweeperGameDto>($"game/{newGame.Id}");

            var minePoint = GetMinePoint(repositoryGame.Board);
            var game = await _client.PostAsJsonAsync<Point, MinesweeperGameDto>($"game/flag/{newGame.Id}", minePoint);

            victoryGame.Should().BeEquivalentTo(game);
        }

        private static Point GetNumberPoint(BoardState[][] board)
        {
            return board.Select((column, xIndex) =>
                            column.Select((cell, yIndex) => new Point(xIndex, yIndex))
                                  .FirstOrDefault(cell => board[cell.X][cell.Y] != BoardState.MINE && board[cell.X][cell.Y] != BoardState.ZERO))
                        .First(x => x != null) ?? new Point(-1, -1);
        }

        private static IEnumerable<Point> GetAllNumberPoints(BoardState[][] board)
        {
            return board.SelectMany((column, xIndex) =>
                            column
                            .Select((cell, yIndex) => new Point(xIndex, yIndex))
                            .Where(point => board[point.X][point.Y] != BoardState.MINE));
        }

        private static Point GetMinePoint(BoardState[][] board)
        {
            return board.Select((column, xIndex) =>
                            column.Select((cell, yIndex) => new Point(xIndex, yIndex))
                                  .FirstOrDefault(cell => board[cell.X][cell.Y] == BoardState.MINE))
                        .First(x => x != null) ?? new Point(-1, -1);
        }

        private static MinesweeperGameDto GetGameFromActionResult(ActionResult<MinesweeperGameDto> actionResult)
        {
            Assert.NotNull(actionResult);
            Assert.IsType<OkObjectResult>(actionResult?.Result);
            var game = (actionResult?.Result as OkObjectResult)?.Value as MinesweeperGameDto;
            Assert.NotNull(game);
            return game ?? new MinesweeperGameDto();
        }
    }
}