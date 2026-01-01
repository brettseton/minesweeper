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
    [AllowAnonymous]
    public class GameController : ControllerBase
    {
        private readonly ILogger<GameController> _logger;
        private readonly IGameRepository _repository;

        public GameController(
            ILogger<GameController> logger,
            IGameRepository repository)
        {
            _logger = logger;
            _repository = repository;
        }

        // GET api/values
        [HttpGet("{id?}")]
        public ActionResult<MinesweeperGameDto> Get(int? id)
        {
            if ((id ?? 0) == 0) return NewGame();
            _logger.LogInformation("Getting game {Id}", id);
            var game = _repository.GetGame(id ?? 0);
            if (game == null)
                return NotFound(id);

            return Ok(game.ToGameDto());
        }

        [HttpGet("new")]
        [backend.Filters.AssociateGameWithUser]
        public ActionResult<MinesweeperGameDto> NewGame()
        {
            return New();
        }

        [HttpGet("new/{numberOfColumns}/{numberOfRows}/{numberOfMines}")]
        [backend.Filters.AssociateGameWithUser]
        public ActionResult<MinesweeperGameDto> New([FromRoute] int numberOfColumns = 10, [FromRoute] int numberOfRows = 10, [FromRoute] int numberOfMines = 10)
        {
            _logger.LogInformation("Creating new game");
            var game = new MinesweeperGame().GetNewGame(numberOfColumns, numberOfRows, numberOfMines);
            _repository.Save(game);
            return Ok(game.ToGameDto());
        }

        // POST api/values
        [HttpPost("{id?}")]
        public ActionResult<MinesweeperGameDto> Post([FromRoute] int? id, [FromBody] Point point)
        {
            var gameId = id ?? point.GameId ?? 0;
            if (gameId == 0) return BadRequest("Game ID is required");

            _logger.LogInformation("Called with input: {Point} for game: {GameId}", point, gameId);
            var game = _repository.GetGame(gameId);
            if (game == null) return NotFound(gameId);

            if (game.Board == null) return StatusCode(500, "Game board is not initialized");

            if (point.X < 0 || point.X >= game.Board.Length || point.Y < 0 || point.Y >= game.Board[0].Length)
                return BadRequest("Coordinates out of bounds");

            if (game.IsGameOver()) return Ok(game.ToGameDto());
            if (game.Moves != null && game.Moves.Contains(point)) return Ok(game.ToGameDto());
            if (game.FlagPoints != null && game.FlagPoints.Contains(point)) return Ok(game.ToGameDto());

            // 0 square needs to reveal the area it is in
            if (game.Board[point.X][point.Y] == BoardState.ZERO)
                return Ok(_repository.AddMoves(gameId, game.GetZeroMoves(point).ToArray()).ToGameDto());

            // Reveal all mine locations
            if (game.Board[point.X][point.Y] == BoardState.MINE)
                return Ok(_repository.AddMoves(gameId, game.MinePoints?.ToArray() ?? Array.Empty<Point>()).ToGameDto());

            return Ok(_repository.AddMoves(gameId, new Point[] { point }).ToGameDto());
        }

        // Toggle Flag
        [HttpPost("flag/{id?}")]
        public ActionResult<MinesweeperGameDto> ToggleFlag([FromRoute] int? id, [FromBody] Point point)
        {
            var gameId = id ?? point.GameId ?? 0;
            if (gameId == 0) return BadRequest("Game ID is required");

            _logger.LogInformation("Called with input: {Point} for game: {GameId}", point, gameId);
            var game = _repository.GetGame(gameId);
            if (game == null) return NotFound(gameId);

            if (game.Board == null) return StatusCode(500, "Game board is not initialized");

            if (point.X < 0 || point.X >= game.Board.Length || point.Y < 0 || point.Y >= game.Board[0].Length)
                return BadRequest("Coordinates out of bounds");

            if (game.IsGameOver())
                return Ok(game.ToGameDto());

            // Cannot toggle flag on a space that has already been revealed
            if (game.Moves != null && game.Moves.Contains(point))
                return Ok(game.ToGameDto());

            if (game.FlagPoints != null && game.FlagPoints.Contains(point))
                return Ok(_repository.RemoveFlag(gameId, point).ToGameDto());
            return Ok(_repository.AddFlag(gameId, point).ToGameDto());
        }
    }
}
