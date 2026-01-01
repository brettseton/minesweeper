using backend.Extensions;
using backend.Model;
using backend.Repository;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.Filters;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

namespace backend.Filters
{
    public class AssociateGameWithUserAttribute : ActionFilterAttribute
    {
        public override void OnActionExecuted(ActionExecutedContext context)
        {
            // Only proceed if the action was successful and returned a Game DTO
            if (context.Result is OkObjectResult okResult && okResult.Value is MinesweeperGameDto gameDto)
            {
                var user = context.HttpContext.User;
                if (user.Identity?.IsAuthenticated == true)
                {
                    var userId = user.GetUserId();
                    if (userId != null)
                    {
                        var logger = context.HttpContext.RequestServices.GetRequiredService<ILogger<AssociateGameWithUserAttribute>>();
                        var userGameRepository = context.HttpContext.RequestServices.GetRequiredService<IUserGameRepository>();

                        logger.LogInformation($"Automatically associating game {gameDto.Id} with user {userId}");
                        userGameRepository.AddMapping(userId, gameDto.Id);
                    }
                }
            }

            base.OnActionExecuted(context);
        }
    }
}

