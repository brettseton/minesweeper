using backend.Comparers;
using backend.Extensions;
using backend.Repository;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Logging;
using System;
using System.Collections.Generic;
using System.Linq;

namespace backend.Controllers
{
    [Route("[controller]")]
    [ApiController]
    public class GameController : ControllerBase
    {
        private readonly ILogger _logger;
        private readonly IGameRepository _repository;

        public GameController(
            ILoggerFactory factory,
            IGameRepository repository)
        {
            _logger = factory.CreateLogger<GameController>();
            _repository = repository;
        }

        // GET api/values
        [HttpGet("{id?}")]
        public ActionResult<MinesweeperGame> Get(int? id)
        {
            if ((id ?? 0) == 0) return RedirectToAction("New");
            _logger.LogInformation($"Getting game {id}");
            var game = _repository.GetGame(id ?? 0);
            if (game == null)
                return NotFound(id);
            var numberOfColumns = game.Board.Length;
            var numberOfRows = game.Board[0].Length;
            var gameState = new MinesweeperGame()
            {
                Id = game.Id,
                Board = new int[game.Board.Length][],
                MinePoints = game.MinePoints,
                FlagPoints = game.FlagPoints
            };

            foreach (var x in Enumerable.Range(0, numberOfColumns))
            {
                gameState.Board[x] = new int[numberOfRows];
                foreach (var y in Enumerable.Range(0, numberOfRows))
                {
                    gameState.Board[x][y] = -1;
                }
            }

            foreach (var flag in game.FlagPoints)
            {
                gameState.Board[flag.X][flag.Y] = -3;
            }

            foreach (var move in game.Moves)
            {
                gameState.Board[move.X][move.Y] = game.Board[move.X][move.Y];
            }

            return Ok(gameState);
        }

        [HttpGet("new")]
        public ActionResult<MinesweeperGame> New()
        {
            _logger.LogInformation($"Creating new game");
            var game = GetNewGame();
            _repository.Save(game);
            return Ok(game);
        }

        // POST api/values
        [HttpPost("{id}")]
        public ActionResult<MinesweeperGame> Post([FromRoute] int id, [FromBody] Point point)
        {
            _logger.LogInformation("Called with input:{0}", point);
            var game = _repository.GetGame(id);
            if (game.Moves != null && game.Moves.Contains(point, new PointComparer())) return game;
            if (game.Moves != null && game.MinePoints.Intersect(game.Moves, new PointComparer()).Any()) return game;
            if (game.Moves != null && game.FlagPoints.Contains(point, new PointComparer())) return game;

            // 0 square needs to reveal the area it is in
            if (game.Board[point.X][point.Y] == 0)
                return Ok(_repository.AddMoves(id, GetZeroMoves(game, point).ToArray()));

            // Reveal all bomb locations
            if (game.Board[point.X][point.Y] == -2)
                return Ok(_repository.AddMoves(id, game.MinePoints.ToArray()));

            return Ok(_repository.AddMoves(id, new Point[] { point }));
        }

        // POST api/values
        [HttpPost("flag/{id}")]
        public ActionResult<MinesweeperGame> ToggleFlag([FromRoute] int id, [FromBody] Point point)
        {
            _logger.LogInformation("Called with input:{0}", point);
            var game = _repository.GetGame(id);
            if (game.FlagPoints != null && game.FlagPoints.Contains(point, new PointComparer()))
                return Ok(_repository.RemoveFlag(id, point));
            return Ok(_repository.AddFlag(id, point));
        }

        private static HashSet<Point> GetZeroMoves(MinesweeperGame game, Point point)
        {
            var points = new HashSet<Point>(new PointComparer())
            {
                point
            };

            var numberOfColumns = game.Board.Length;
            var numberOfRows = game.Board[0].Length;
            var i = 0;
            while (i < points.Count)
            {
                var currentPoint = points.ElementAt(i);
                if (game.Board[currentPoint.X][currentPoint.Y] == 0)
                {
                    if (currentPoint.X - 1 >= 0)
                    {
                        if (currentPoint.Y - 1 >= 0) points.Add(new Point() { X = currentPoint.X - 1, Y = currentPoint.Y - 1 });
                        points.Add(new Point() { X = currentPoint.X - 1, Y = currentPoint.Y });
                        if (currentPoint.Y + 1 < numberOfRows) points.Add(new Point() { X = currentPoint.X - 1, Y = currentPoint.Y + 1 });
                    }
                    if (currentPoint.Y - 1 >= 0) points.Add(new Point() { X = currentPoint.X, Y = currentPoint.Y - 1 });
                    if (currentPoint.Y + 1 < numberOfRows) points.Add(new Point() { X = currentPoint.X, Y = currentPoint.Y + 1 });

                    if (currentPoint.X + 1 < numberOfColumns)
                    {
                        if (currentPoint.Y - 1 >= 0) points.Add(new Point() { X = currentPoint.X + 1, Y = currentPoint.Y - 1 });
                        points.Add(new Point() { X = currentPoint.X + 1, Y = currentPoint.Y });
                        if (currentPoint.Y + 1 < numberOfRows) points.Add(new Point() { X = currentPoint.X + 1, Y = currentPoint.Y + 1 });
                    }
                }
                ++i;
            }
            return points;
        }

        private static MinesweeperGame GetNewGame()
        {
            var random = new Random();
            var numberOfColumns = random.Next(5, 10);
            var numberOfRows = random.Next(5, 10);
            var numberOfMines = numberOfColumns * numberOfRows * 1 / 10;
            var board = new int[numberOfColumns][];
            var mines = new int[numberOfColumns * numberOfRows];
            var minePoints = new HashSet<Point>();

            for (var i = 0; i < numberOfMines; ++i)
            {
                mines[i] = 1;
            }

            mines.Shuffle();

            for (var x = 0; x < numberOfColumns; ++x)
            {
                board[x] = new int[numberOfRows];
                for (var y = 0; y < numberOfRows; ++y)
                {
                    if (mines[y * numberOfColumns + x] == 1)
                    {
                        board[x][y] = -2;
                        minePoints.Add(new Point() { X = x, Y = y });
                    }
                    else
                    {
                        // Top row
                        if (y - 1 >= 0)
                        {
                            board[x][y] += (x - 1) >= 0 ? mines[(y - 1) * numberOfColumns + x - 1] : 0;
                            board[x][y] += mines[(y - 1) * numberOfColumns + x];
                            board[x][y] += (x + 1) < numberOfColumns ? mines[(y - 1) * numberOfColumns + x + 1] : 0;
                        }

                        // Either side
                        board[x][y] += (x - 1) >= 0 ? mines[y * numberOfColumns + x - 1] : 0;
                        board[x][y] += (x + 1) < numberOfColumns ? mines[y * numberOfColumns + x + 1] : 0;

                        //Bottom Row
                        if (y + 1 < numberOfRows)
                        {
                            board[x][y] += (x - 1) >= 0 ? mines[(y + 1) * numberOfColumns + x - 1] : 0;
                            board[x][y] += mines[(y + 1) * numberOfColumns + x];
                            board[x][y] += (x + 1) < numberOfColumns ? mines[(y + 1) * numberOfColumns + x + 1] : 0;
                        }
                    }
                }
            }
            return new MinesweeperGame() { Id = random.Next(), Board = board, Moves = new HashSet<Point>(), MinePoints = minePoints, FlagPoints = new HashSet<Point>() };
        }
    }
}
