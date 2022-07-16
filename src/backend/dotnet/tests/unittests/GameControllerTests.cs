using backend.Controllers;
using backend.Extensions;
using backend.Model;
using backend.Repository;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Logging;
using Moq;

namespace unittests
{
    public class GameControllerTests
    {
        private readonly InMemoryGameRepository _inMemoryRepository;
        private readonly GameController _gameController;

        public GameControllerTests()
        {
            _inMemoryRepository = new InMemoryGameRepository(new LoggerFactory());
            _gameController = new GameController(new LoggerFactory(), _inMemoryRepository);
        }

        [Fact]
        public void CreateNewGame_MatchesGetCall()
        {
            var newGame = GetGameFromActionResult(_gameController.New(10, 100, 10));
            var game = GetGameFromActionResult(_gameController.Get(newGame.Id));
            Assert.Equal(newGame.ToJson(), game.ToJson());
            Assert.True(newGame.Id > 0);
            Assert.NotNull(newGame.Board);
            Assert.Equal(10, newGame.Board.Length);
            for(var i = 0; i < newGame.Board.Length; i++)
                Assert.Equal(100, newGame.Board[i].Length);
            Assert.Equal(10, newGame.MineCount);
            Assert.Empty(newGame.FlagPoints);
        }

        [Fact]
        public void ToggleFlagOnAndOff_ReturnsCorrectBoardState()
        {
            var newGame = GetGameFromActionResult(_gameController.New(10, 10, 10));
            var game = GetGameFromActionResult(_gameController.ToggleFlag(newGame.Id, new Point(0, 0)));
            Assert.Equal(BoardState.FLAG, game.Board[0][0]);
            Assert.Single(newGame.FlagPoints);
            game = GetGameFromActionResult(_gameController.ToggleFlag(newGame.Id, new Point(0, 0)));
            for (var x = 0; x < newGame.Board.Length; x++)
                for (var y = 0; y < newGame.Board[0].Length; y++)
                Assert.Equal(BoardState.UNKNOWN, newGame.Board[x][y]);
        }

        [Fact]
        public void MoveOnEmptySpace_ReturnsCorrectBoardState()
        {
            var newGame = GetGameFromActionResult(_gameController.New(10, 10, 5));
            var repositoryGame = _inMemoryRepository.GetGame(newGame.Id);
            var firstNonMinePoint = repositoryGame.Board
                .Select((column, xIndex) => column.Select((cell, yIndex) => new { X = xIndex, Y = yIndex }).FirstOrDefault(cell => repositoryGame.Board[cell.X][cell.Y] != BoardState.MINE && repositoryGame.Board[cell.X][cell.Y] != BoardState.ZERO))
                .First(x => x != null);

            var game = GetGameFromActionResult(_gameController.Post(newGame.Id, new Point(firstNonMinePoint.X, firstNonMinePoint.Y)));
            Assert.InRange(game.Board[firstNonMinePoint.X][firstNonMinePoint.Y], BoardState.ONE, BoardState.EIGHT);
            Assert.NotEqual(BoardState.MINE, game.Board[firstNonMinePoint.X][firstNonMinePoint.Y]);
            Assert.NotEqual(BoardState.ZERO, game.Board[firstNonMinePoint.X][firstNonMinePoint.Y]);
            Assert.NotEqual(BoardState.UNKNOWN, game.Board[firstNonMinePoint.X][firstNonMinePoint.Y]);
            Assert.NotEqual(BoardState.FLAG, game.Board[firstNonMinePoint.X][firstNonMinePoint.Y]);
        }

        [Fact]
        public void MoveOnMineSpace_ReturnsCorrectBoardState()
        {
            var newGame = GetGameFromActionResult(_gameController.New(10, 10, 5));
            var repositoryGame = _inMemoryRepository.GetGame(newGame.Id);
            var firstMinePoint = repositoryGame.Board
                .Select((column, xIndex) => column.Select((cell, yIndex) => new { X = xIndex, Y = yIndex }).FirstOrDefault(cell => repositoryGame.Board[cell.X][cell.Y] == BoardState.MINE))
                .First(x => x != null);

            var game = GetGameFromActionResult(_gameController.Post(newGame.Id, new Point(firstMinePoint.X, firstMinePoint.Y)));
            
            Assert.NotEqual(BoardState.MINE, game.Board[firstMinePoint.X][firstMinePoint.Y]);
            foreach(var mine in repositoryGame.MinePoints)
            {
                Assert.NotEqual(BoardState.MINE, game.Board[mine.X][mine.Y]);
            }
        }

        private static MinesweeperGameDto GetGameFromActionResult(ActionResult<MinesweeperGameDto> actionResult)
        {
            Assert.NotNull(actionResult);
            Assert.IsType<OkObjectResult>(actionResult?.Result);
            var game = (actionResult?.Result as OkObjectResult)?.Value as MinesweeperGameDto;
            Assert.NotNull(game);
            return game;
        }
    }
}