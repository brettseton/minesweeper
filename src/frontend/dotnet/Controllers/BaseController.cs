using System.Threading.Tasks;
using dotnet.Services;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.Filters;
using Microsoft.Extensions.Logging;

namespace dotnet.Controllers
{
    public abstract class BaseController : Controller
    {
        protected readonly IGameService _gameService;
        protected readonly ILogger _logger;

        public BaseController(IGameService gameService, ILogger logger)
        {
            _gameService = gameService;
            _logger = logger;
        }

        public override async Task OnActionExecutionAsync(ActionExecutingContext context, ActionExecutionDelegate next)
        {
            await PopulateUserStatus();
            await base.OnActionExecutionAsync(context, next);
        }

        protected async Task PopulateUserStatus()
        {
            var (isAuthenticated, name) = await _gameService.GetUserStatus();
            if (isAuthenticated)
            {
                ViewBag.IsAuthenticated = true;
                ViewBag.UserName = name;
            }
        }
    }
}
