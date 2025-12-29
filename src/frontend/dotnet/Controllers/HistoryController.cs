using System.Threading.Tasks;
using dotnet.Services;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Logging;
using System.Linq;

namespace dotnet.Controllers
{
    [Route("history")]
    public class HistoryController : BaseController
    {
        public HistoryController(IGameService gameService, ILogger<HistoryController> logger)
            : base(gameService, logger)
        {
        }

        [HttpGet("")]
        public async Task<IActionResult> Index()
        {
            var games = await _gameService.GetUserGames();
            return View(games.OrderByDescending(g => g.CreatedAt).ToList());
        }
    }
}
