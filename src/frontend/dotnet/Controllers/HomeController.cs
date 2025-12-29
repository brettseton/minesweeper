using System;
using System.Threading.Tasks;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Logging;
using dotnet.Models;
using System.Linq;
using dotnet.Services;

namespace dotnet.Controllers
{
    [Route("")]
    public class HomeController : BaseController
    {
        public HomeController(ILoggerFactory loggerFactory, IGameService gameService)
            : base(gameService, loggerFactory.CreateLogger<HomeController>())
        {
        }

        [HttpGet("")]
        public async Task<IActionResult> Index()
        {
            _logger.LogInformation($"Getting Home Index");
            
            if (ViewBag.IsAuthenticated == true)
            {
                try
                {
                    var games = await _gameService.GetUserGames();
                    var latestActiveGame = games?
                        .Where(g => g.Status == GameStatus.InProgress)
                        .OrderByDescending(g => g.CreatedAt)
                        .FirstOrDefault();
                    
                    ViewBag.LatestGameId = latestActiveGame?.Id;
                }
                catch (Exception e)
                {
                    _logger.LogError($"Failed to fetch games: {e}");
                }
            }

            return View();
        }
    }
}
