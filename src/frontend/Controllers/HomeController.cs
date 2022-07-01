using System.Threading.Tasks;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Logging;

namespace dotnet.Controllers
{
    [Route("")]
    [ApiController]
    public class HomeController : Controller
    {
        private ILogger _logger;

        public HomeController(ILoggerFactory loggerFactory)
        {
            _logger = loggerFactory.CreateLogger<HomeController>();
        }

        [HttpGet("")]
        public async Task<IActionResult> Index()
        {
            _logger.LogInformation($"Getting Home Index");
            return View();
        }
    }
}
