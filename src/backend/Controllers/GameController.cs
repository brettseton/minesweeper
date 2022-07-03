using System;
using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.Linq;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Logging;

namespace backend.Controllers
{
    [Route("[controller]")]
    [ApiController]
    public class GameController : ControllerBase
    {
        private ILogger _logger;
        private IGameRepository _repository;

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

            foreach(var x in Enumerable.Range(0, numberOfColumns)) {
                gameState.Board[x] = new int[numberOfRows];
                foreach(var y in Enumerable.Range(0, numberOfRows)){
                   gameState.Board[x][y] = -1;
                }
            }

            foreach(var flag in game.FlagPoints) {
                gameState.Board[flag.x][flag.y] = -3;
            }

            foreach(var move in game.Moves) {
                gameState.Board[move.x][move.y] = game.Board[move.x][move.y];
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
            _logger.LogInformation("called with input:{0}", point);
            var game = _repository.GetGame(id);
            if (game.Moves != null && game.Moves.Contains(point, new PointComparer())) return game;
            if (game.Moves != null && game.MinePoints.Intersect(game.Moves, new PointComparer()).Any()) return game;
            if (game.Moves != null && game.FlagPoints.Contains(point, new PointComparer())) return game;

            // 0 square needs to reveal the area it is in
            if (game.Board[point.x][point.y] == 0)
                return Ok(_repository.AddMoves(id, GetZeroMoves(game, point).ToArray()));

            // Reveal all bomb locations
            if (game.Board[point.x][point.y] == -2)
                return Ok(_repository.AddMoves(id, game.MinePoints.ToArray()));

            return Ok(_repository.AddMoves(id, new Point[] {point}));
        }

        // POST api/values
        [HttpPost("flag/{id}")]
        public ActionResult<MinesweeperGame> ToggleFlag([FromRoute] int id, [FromBody] Point point)
        {
            _logger.LogInformation("called with input:{0}", point);
            var game = _repository.GetGame(id);
            if (game.FlagPoints != null && game.FlagPoints.Contains(point, new PointComparer()))
              return Ok(_repository.RemoveFlag(id, point));
            return Ok(_repository.AddFlag(id, point));
        }

        private HashSet<Point> GetZeroMoves(MinesweeperGame game, Point point)
        {
            var points = new HashSet<Point>(new PointComparer());
            points.Add(point);

            var numberOfColumns = game.Board.Length;
            var numberOfRows = game.Board[0].Length;
            var i = 0;
            while (i < points.Count)
            {
                var currentPoint = points.ElementAt(i);
                if (game.Board[currentPoint.x][currentPoint.y] == 0)
                {
                    if (currentPoint.x - 1 >= 0)
                    {
                        if (currentPoint.y - 1 >= 0) points.Add(new Point() { x = currentPoint.x - 1, y = currentPoint.y - 1 });
                        points.Add(new Point() { x = currentPoint.x - 1, y = currentPoint.y });
                        if (currentPoint.y + 1 < numberOfRows) points.Add(new Point() { x = currentPoint.x - 1, y = currentPoint.y + 1 });
                    }
                    if (currentPoint.y - 1 >= 0) points.Add(new Point() { x = currentPoint.x, y = currentPoint.y - 1 });
                    if (currentPoint.y + 1 < numberOfRows) points.Add(new Point() { x = currentPoint.x, y = currentPoint.y + 1 });

                    if (currentPoint.x + 1 < numberOfColumns)
                    {
                        if (currentPoint.y - 1 >= 0) points.Add(new Point() { x = currentPoint.x + 1, y = currentPoint.y - 1 });
                        points.Add(new Point() { x = currentPoint.x + 1, y = currentPoint.y });
                        if (currentPoint.y + 1 < numberOfRows) points.Add(new Point() { x = currentPoint.x + 1, y = currentPoint.y + 1 });
                    }
                }
                ++i;
            }
            return points;
        }

        private MinesweeperGame GetNewGame()
        {
            var random = new Random();
            var numberOfColumns = random.Next(5,10);
            var numberOfRows = random.Next(5,10);
            var numberOfMines = numberOfColumns*numberOfRows*1/10;
            var board = new int[numberOfColumns][];
            var mines = new int[numberOfColumns*numberOfRows];
            var minePoints = new HashSet<Point>();

            for(var i = 0; i < numberOfMines; ++i) {
                mines[i] = 1;
            }

            mines.Shuffle();

            for(var x = 0; x < numberOfColumns; ++x) {
                board[x] = new int[numberOfRows];
                for(var y = 0; y < numberOfRows; ++y){
                    if (mines[y*numberOfColumns+x] == 1) {
                        board[x][y] = -2;
                        minePoints.Add(new Point(){x = x, y = y});
                    } else {
                        // Top row
                        if(y-1 >= 0) {
                            board[x][y] += (x-1) >= 0 ? mines[(y-1)*numberOfColumns+x-1] : 0;
                            board[x][y] += mines[(y-1)*numberOfColumns+x];
                            board[x][y] += (x+1) < numberOfColumns ? mines[(y-1)*numberOfColumns+x+1] : 0;
                        }

                        // Either side
                        board[x][y] += (x-1) >= 0 ? mines[y*numberOfColumns+x-1] : 0;
                        board[x][y] += (x+1) < numberOfColumns ? mines[y*numberOfColumns+x+1] : 0;
                        
                        //Bottom Row
                        if (y+1 < numberOfRows)
                        {
                            board[x][y] += (x-1) >= 0 ? mines[(y+1)*numberOfColumns+x-1] : 0;
                            board[x][y] += mines[(y+1)*numberOfColumns+x];
                            board[x][y] += (x+1) < numberOfColumns ? mines[(y+1)*numberOfColumns+x+1] : 0;
                        }
                    }
                }
            }
            return new MinesweeperGame(){Id = random.Next(), Board = board, Moves = new HashSet<Point>(), MinePoints = minePoints, FlagPoints = new HashSet<Point>()};
        }
    }

    public class PointComparer : IEqualityComparer<Point>
    {
        public bool Equals(Point? x, Point? y)
        {
            return x?.x == y?.x && x?.y == y?.y;
        }

        public int GetHashCode([DisallowNull] Point p)
        {
            return 357 ^ p.x + 411 ^ p.y; 
        }
    }
}

public static class IEnumerableExtensions{
    private static Random random = new Random();
    public static void Shuffle<T>(this IList<T> list)
    {  
        int n = list.Count;
        while (n > 1) {
            n--;
            int k = random.Next(n + 1);
            T value = list[k];
            list[k] = list[n];
            list[n] = value;
        }
    }
}
