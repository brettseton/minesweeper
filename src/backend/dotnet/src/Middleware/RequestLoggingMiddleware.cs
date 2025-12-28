using System.Threading.Tasks;
using System.Collections.Generic;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.Logging;
using backend.Extensions;

namespace backend.Middleware
{
    public class RequestLoggingMiddleware
    {
        private readonly RequestDelegate _next;
        private readonly ILogger<RequestLoggingMiddleware> _logger;

        public RequestLoggingMiddleware(RequestDelegate next, ILogger<RequestLoggingMiddleware> logger)
        {
            _next = next;
            _logger = logger;
        }

        public async Task InvokeAsync(HttpContext context)
        {
            var userId = context.User.Identity?.IsAuthenticated == true
                ? context.User.GetUserId()
                : "Anonymous";

            using (_logger.BeginScope(new Dictionary<string, object> { ["UserId"] = userId }))
            {
                await _next(context);
            }
        }
    }
}
