using System;
using System.Threading.Tasks;
using dotnet.Models;
using dotnet.Services;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Logging;

namespace dotnet.Controllers
{
    [Route("Game")]
    [ApiController]
    public class GameController : BaseController
    {
        public GameController(
            IGameService gameService,
            ILogger<GameController> logger)
            : base(gameService, logger)
        {
        }

        [HttpGet("{id?}")]
        public async Task<IActionResult> Game(int? id)
        {
            _logger.LogInformation($"Getting game {id}");

            try
            {
                if ((id ?? 0) == 0) return Redirect($"~/game/new");

                var game = await _gameService.GetGame(id!.Value);
                if (game.Id == id)
                {
                    return View(game);
                }
                return Redirect($"~/game/{game.Id}");
            }
            catch (Exception e)
            {
                _logger.LogError(e.ToString());
                return Redirect("~/");
            }
        }

        [HttpGet("new")]
        public async Task<IActionResult> NewGame()
        {
            _logger.LogInformation($"Getting new game");

            try
            {
                var game = await _gameService.NewGame();
                return Redirect($"~/game/{game.Id}");
            }
            catch (Exception e)
            {
                _logger.LogError(e.ToString());
                return Content($"Error starting new game: {e.Message}");
            }
        }

        [HttpPost("{id}/{x}/{y}")]
        [ValidateAntiForgeryToken]
        public async Task<IActionResult> Post([FromRoute] int id, [FromRoute] int x, [FromRoute] int y)
        {
            _logger.LogInformation($"Guess co-ordinates {x},{y}");

            try
            {
                await _gameService.MakeMove(id, x, y);
                return Redirect($"~/game/{id}");
            }
            catch (Exception e)
            {
                _logger.LogError(e.ToString());
                return Redirect("~/");
            }
        }

        [HttpPost("flag/{id}/{x}/{y}")]
        [ValidateAntiForgeryToken]
        public async Task<IActionResult> ToggleFlag([FromRoute] int id, [FromRoute] int x, [FromRoute] int y)
        {
            _logger.LogInformation($"Flag co-ordinates {x},{y}");

            try
            {
                await _gameService.ToggleFlag(id, x, y);
                return Redirect($"~/game/{id}");
            }
            catch (Exception e)
            {
                _logger.LogError(e.ToString());
                return Redirect("~/");
            }
        }
    }
}
