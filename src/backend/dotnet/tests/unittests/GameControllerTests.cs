using backend.Controllers;
using backend.Model;
using backend.Repository;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Logging;

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
            AssertGameDtosAreEqual(newGame, game);

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
        public void ToggleFlagOnAndOff_ReturnsCorrectBoardState()
        {
            var newGame = GetGameFromActionResult(_gameController.New(10, 10, 10));
            var game = GetGameFromActionResult(_gameController.ToggleFlag(newGame.Id, new Point(0, 0)));
            Assert.Equal(BoardState.FLAG, game.Board[0][0]);
            Assert.Single(game.FlagPoints);
            game = GetGameFromActionResult(_gameController.ToggleFlag(game.Id, new Point(0, 0)));
            for (var x = 0; x < game.Board.Length; x++)
                for (var y = 0; y < game.Board[0].Length; y++)
                    Assert.Equal(BoardState.UNKNOWN, game.Board[x][y]);
        }

        [Fact]
        public void ClickOnFlag_ReturnsCorrectBoardState()
        {
            var newGame = GetGameFromActionResult(_gameController.New(10, 10, 5));
            var repositoryGame = _inMemoryRepository.GetGame(newGame.Id);
            var numberedPoint = GetNumberPoint(repositoryGame.Board);

            var gameFlagToggled = GetGameFromActionResult(_gameController.ToggleFlag(newGame.Id, numberedPoint));
            var game = GetGameFromActionResult(_gameController.Post(newGame.Id, numberedPoint));

            AssertGameDtosAreEqual(gameFlagToggled, game);
        }

        [Fact]
        public void MoveOnNumberedSpace_ReturnsCorrectBoardState()
        {
            var newGame = GetGameFromActionResult(_gameController.New(10, 10, 5));
            var repositoryGame = _inMemoryRepository.GetGame(newGame.Id);
            var numberedPoint = GetNumberPoint(repositoryGame.Board);

            var game = GetGameFromActionResult(_gameController.Post(newGame.Id, numberedPoint));
            Assert.InRange(game.Board[numberedPoint.X][numberedPoint.Y], BoardState.ONE, BoardState.EIGHT);
        }

        [Fact]
        public void MoveOnMineSpace_ReturnsCorrectBoardState()
        {
            var newGame = GetGameFromActionResult(_gameController.New(10, 10, 5));
            var repositoryGame = _inMemoryRepository.GetGame(newGame.Id);
            var minePoint = GetMinePoint(repositoryGame.Board);

            var game = GetGameFromActionResult(_gameController.Post(newGame.Id, new Point(minePoint.X, minePoint.Y)));

            Assert.Equal(BoardState.MINE, game.Board[minePoint.X][minePoint.Y]);
            foreach (var mine in repositoryGame.MinePoints)
            {
                Assert.Equal(BoardState.MINE, game.Board[mine.X][mine.Y]);
            }
        }

        [Fact]
        public void MoveAfterGameOver_DoesntChangeBoardState()
        {
            var newGame = GetGameFromActionResult(_gameController.New(10, 10, 5));
            var repositoryGame = _inMemoryRepository.GetGame(newGame.Id);
            var minePoint = GetMinePoint(repositoryGame.Board);

            var gameOver = GetGameFromActionResult(_gameController.Post(newGame.Id, minePoint));

            var numberedPoint = GetNumberPoint(repositoryGame.Board);

            var game = GetGameFromActionResult(_gameController.Post(newGame.Id, numberedPoint));

            AssertGameDtosAreEqual(gameOver, game);

        }

        [Fact]
        public void MoveAfterVictory_DoesntChangeBoardState()
        {
            var newGame = GetGameFromActionResult(_gameController.New(10, 10, 5));
            var repositoryGame = _inMemoryRepository.GetGame(newGame.Id);
            var numberedPoints = GetAllNumberPoints(repositoryGame.Board);
            foreach(var numberPoint in numberedPoints)
            {
                _gameController.Post(newGame.Id, numberPoint);
            }
            var victoryGame = GetGameFromActionResult(_gameController.Get(newGame.Id));

            var minePoint = GetMinePoint(repositoryGame.Board);
            var game = GetGameFromActionResult(_gameController.Post(newGame.Id, minePoint));

            AssertGameDtosAreEqual(victoryGame, game);
        }

        [Fact]
        public void ToggleFlagAfterGameOver_DoesntChangeBoardState()
        {
            var newGame = GetGameFromActionResult(_gameController.New(10, 10, 5));
            var repositoryGame = _inMemoryRepository.GetGame(newGame.Id);
            var minePoint = GetMinePoint(repositoryGame.Board);
            var gameOver = GetGameFromActionResult(_gameController.Post(newGame.Id, minePoint));
            
            var numberedPoint = GetNumberPoint(repositoryGame.Board);
            var game = GetGameFromActionResult(_gameController.ToggleFlag(newGame.Id, numberedPoint));

            AssertGameDtosAreEqual(gameOver, game);
        }

        [Fact]
        public void ToggleFlagAfterVictory_DoesntChangeBoardState()
        {
            var newGame = GetGameFromActionResult(_gameController.New(10, 10, 5));
            var repositoryGame = _inMemoryRepository.GetGame(newGame.Id);
            var numberedPoints = GetAllNumberPoints(repositoryGame.Board);
            foreach (var numberPoint in numberedPoints)
            {
                _gameController.Post(newGame.Id, numberPoint);
            }
            var victoryGame = GetGameFromActionResult(_gameController.Get(newGame.Id));

            var minePoint = GetMinePoint(repositoryGame.Board);
            var game = GetGameFromActionResult(_gameController.ToggleFlag(newGame.Id, minePoint));

            AssertGameDtosAreEqual(victoryGame, game);
        }

        private static void AssertGameDtosAreEqual(MinesweeperGameDto expected, MinesweeperGameDto actual)
        {
            Assert.Equal(expected.Id, actual.Id);
            Assert.Equal(expected.MineCount, actual.MineCount);
            Assert.Equal(expected.FlagPoints, actual.FlagPoints);
            Assert.NotNull(expected.Board);
            Assert.NotNull(actual.Board);
            Assert.Equal(expected.Board.Length, actual.Board.Length);
            Assert.Equal(expected.FlagPoints, actual.FlagPoints);
            for (var x = 0; x < expected.Board.Length; x++)
            {
                Assert.Equal(expected.Board[x].Length, actual.Board[x].Length);
                for (var y = 0; y < expected.Board[0].Length; y++)
                    Assert.Equal(expected.Board[x][y], actual.Board[x][y]);
            }
        }

        private static Point GetNumberPoint(BoardState[][] board)
        {
            return board.Select((column, xIndex) =>
                            column.Select((cell, yIndex) => new Point(xIndex, yIndex))
                                  .FirstOrDefault(cell => board[cell.X][cell.Y] != BoardState.MINE && board[cell.X][cell.Y] != BoardState.ZERO))
                        .First(x => x != null);
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
                        .First(x => x != null);
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