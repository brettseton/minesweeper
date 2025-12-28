using System.Collections.Generic;
using System.Linq;
using backend.Extensions;
using backend.Model;
using backend.Repository;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Logging;

namespace backend.Controllers
{
    [Route("user")]
    [ApiController]
    [Authorize]
    public class UserController : ControllerBase
    {
        private readonly ILogger _logger;
        private readonly IGameRepository _repository;
        private readonly IUserGameRepository _userGameRepository;

        public UserController(
            ILoggerFactory factory,
            IGameRepository repository,
            IUserGameRepository userGameRepository)
        {
            _logger = factory.CreateLogger<UserController>();
            _repository = repository;
            _userGameRepository = userGameRepository;
        }

        [HttpGet("games")]
        public ActionResult<IEnumerable<MinesweeperGameDto>> Games()
        {
            var userId = User.GetUserId();

            var gameIds = _userGameRepository.GetGameIdsByUserId(userId).ToList();
            if (!gameIds.Any()) return Ok(Enumerable.Empty<MinesweeperGameDto>());

            var games = _repository.GetGamesByIds(gameIds);

            return Ok(games.Select(g => g.ToGameDto()));
        }

        [HttpGet("stats")]
        public ActionResult Stats()
        {
            var userId = User.GetUserId();
            var gameIds = _userGameRepository.GetGameIdsByUserId(userId).ToList();
            if (!gameIds.Any()) return Ok(new { Won = 0, Lost = 0, InProgress = 0 });

            var games = _repository.GetGamesByIds(gameIds);

            var won = games.Count(g => g.IsGameWon());
            var lost = games.Count(g => g.IsGameLost());
            var inProgress = games.Count(g => !g.IsGameWon() && !g.IsGameLost());

            return Ok(new { Won = won, Lost = lost, InProgress = inProgress });
        }
    }
}

