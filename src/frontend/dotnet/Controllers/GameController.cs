using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Microsoft.AspNetCore.Mvc;
using dotnet.Models;
using Microsoft.Extensions.Logging;
using System.Net.Http;
using frontend;
using System.Net.Http.Headers;
using System.Text;

namespace dotnet.Controllers
{
    [Route("Game")]
    [ApiController]
    public class GameController : Controller
    {
        private ILogger _logger;
        private IEnvironmentConfiguration _envConfig;
        private IHttpClientFactory _factory;

        public GameController(
            IHttpClientFactory httpFactory,
            ILoggerFactory loggerFactory,
            IEnvironmentConfiguration environmentConfiguration)
        {
            _factory = httpFactory;
            _logger = loggerFactory.CreateLogger<GameController>();
            _envConfig = environmentConfiguration;
        }

        [HttpGet("{id?}")]
        public async Task<IActionResult> Game(int? id)
        {
            _logger.LogInformation($"Getting game {id}");

            // Get the entries from the backend
            try
            {
                if ((id ?? 0) == 0) return Redirect($"~/game/new");

                var httpClient = _factory.CreateClient();
                httpClient.DefaultRequestHeaders.Accept.Add(new MediaTypeWithQualityHeaderValue("application/json"));
                _logger.LogInformation($"Making a request to {_envConfig.BackendGameAddress}");
                var response = await httpClient.GetAsync($"{_envConfig.BackendGameAddress}/{id}");
                _logger.LogInformation("Got response: {0}", response);
                response.EnsureSuccessStatusCode();
                var game = await response.Content.ReadAsAsync<MinesweeperGame>();
                if (game.Id == id)
                {
                    return View(game);
                }
                return Redirect($"~/game/{game.Id}");
            }
            catch (Exception e)
            {
                _logger.LogError(e.ToString());
                return View();
            }
        }

        [HttpGet("new")]
        public async Task<IActionResult> NewGame()
        {
            _logger.LogInformation($"Getting new game");

            // Get the entries from the backend
            try
            {
                var httpClient = _factory.CreateClient();
                httpClient.DefaultRequestHeaders.Accept.Add(new MediaTypeWithQualityHeaderValue("application/json"));

                _logger.LogInformation($"Making a request to {_envConfig.BackendGameAddress}/new");
                var response = await httpClient.GetAsync($"{_envConfig.BackendGameAddress}/new");
                _logger.LogInformation("Got response: {0}", response);
                response.EnsureSuccessStatusCode();
                var game = await response.Content.ReadAsAsync<MinesweeperGame>();
                return Redirect($"~/game/{game.Id}");
            }
            catch (Exception e)
            {
                _logger.LogError(e.ToString());
                return View();
            }
        }

        [HttpPost("{id}/{x}/{y}")]
        //[ValidateAntiForgeryToken]
        public async Task<IActionResult> Post([FromRoute] int id, [FromRoute] int x, [FromRoute] int y)
        {
            _logger.LogInformation($"Guess co-ordinates {x},{y}");

            try
            {
                var httpClient = _factory.CreateClient();
                httpClient.DefaultRequestHeaders.Accept.Add(new MediaTypeWithQualityHeaderValue("application/json"));
                var httpContent = new StringContent($"{{ \"x\": {x}, \"y\": {y} }}", Encoding.UTF8, "application/json");
                var response = await httpClient.PostAsync($"{_envConfig.BackendGameAddress}/{id}", httpContent);
                _logger.LogInformation("Got Post Response: {0}", response);
                return Redirect($"~/game/{id}");
            }
            catch (Exception e)
            {
                _logger.LogError(e.ToString());
                return View();
            }
        }

        [HttpPost("flag/{id}/{x}/{y}")]
        public async Task<IActionResult> ToggleFlag([FromRoute] int id, [FromRoute] int x, [FromRoute] int y)
        {
            _logger.LogInformation($"Flag co-ordinates {x},{y}");

            try
            {
                var httpClient = _factory.CreateClient();
                httpClient.DefaultRequestHeaders.Accept.Add(new MediaTypeWithQualityHeaderValue("application/json"));
                var httpContent = new StringContent($"{{ \"x\": {x}, \"y\": {y} }}", Encoding.UTF8, "application/json");
                var response = await httpClient.PostAsync($"{_envConfig.BackendGameAddress}/flag/{id}", httpContent);
                _logger.LogInformation("Got ToggleFlag Response: {0}", response);
                return Redirect($"~/game/{id}");
            }
            catch (Exception e)
            {
                _logger.LogError(e.ToString());
                return View();
            }
        }
    }
}
