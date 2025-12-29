using System.Threading.Tasks;
using dotnet.Models;
using dotnet.Services;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Logging;

namespace dotnet.Controllers
{
    [Route("stats")]
    public class StatsController : BaseController
    {
        public StatsController(IGameService gameService, ILogger<StatsController> logger)
            : base(gameService, logger)
        {
        }

        [HttpGet("")]
        public async Task<IActionResult> Index()
        {
            var stats = await _gameService.GetUserStats();
            return View(stats);
        }
    }
}
