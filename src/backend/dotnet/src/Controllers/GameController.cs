using System.Linq;
using backend.Extensions;
using backend.Model;
using backend.Repository;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Logging;

namespace backend.Controllers
{
    [Route("[controller]")]
    [ApiController]
    //[Authorize]
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
        public ActionResult<MinesweeperGameDto> Get(int? id)
        {
            if ((id ?? 0) == 0) return NewGame();
            _logger.LogInformation($"Getting game {id}");
            var game = _repository.GetGame(id ?? 0);
            if (game == null)
                return NotFound(id);

            return Ok(game.ToGameDto());
        }

        [HttpGet("new")]
        public ActionResult<MinesweeperGameDto> NewGame()
        {
            return New();
        }

        [HttpGet("new/{numberOfColumns}/{numberOfRows}/{numberOfMines}")]
        public ActionResult<MinesweeperGameDto> New([FromRoute] int numberOfColumns = 10, [FromRoute] int numberOfRows = 10, [FromRoute] int numberOfMines = 10)
        {
            _logger.LogInformation($"Creating new game");
            var game = new MinesweeperGame().GetNewGame(numberOfColumns, numberOfRows, numberOfMines);
            _repository.Save(game);
            return Ok(game.ToGameDto());
        }

        // POST api/values
        [HttpPost("{id}")]
        public ActionResult<MinesweeperGameDto> Post([FromRoute] int id, [FromBody] Point point)
        {
            _logger.LogInformation("Called with input:{0}", point);
            var game = _repository.GetGame(id);
            if (game.IsGameOver()) return Ok(game.ToGameDto());
            if (game.Moves != null && game.Moves.Contains(point)) return Ok(game.ToGameDto());
            if (game.Moves != null && game.FlagPoints.Contains(point)) return Ok(game.ToGameDto());

            // 0 square needs to reveal the area it is in
            if (game.Board[point.X][point.Y] == BoardState.ZERO)
                return Ok(_repository.AddMoves(id, game.GetZeroMoves(point).ToArray()).ToGameDto());

            // Reveal all mine locations
            if (game.Board[point.X][point.Y] == BoardState.MINE)
                return Ok(_repository.AddMoves(id, game.MinePoints.ToArray()).ToGameDto());

            return Ok(_repository.AddMoves(id, new Point[] { point }).ToGameDto());
        }

        // Toggle Flag
        [HttpPost("flag/{id}")]
        public ActionResult<MinesweeperGameDto> ToggleFlag([FromRoute] int id, [FromBody] Point point)
        {
            _logger.LogInformation("Called with input:{0}", point);
            var game = _repository.GetGame(id);
            if (game.IsGameOver())
                return Ok(game.ToGameDto());
            if (game.FlagPoints != null && game.FlagPoints.Contains(point))
                return Ok(_repository.RemoveFlag(id, point).ToGameDto());
            return Ok(_repository.AddFlag(id, point).ToGameDto());
        }
    }
}
