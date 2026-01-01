using System.Collections.Generic;
using System.Threading.Tasks;
using backend.Extensions;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.Logging;

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
            var userId = (context.User.Identity?.IsAuthenticated == true
                ? context.User.GetUserId()
                : "Anonymous") ?? "Unknown";

            using (_logger.BeginScope(new Dictionary<string, object> { ["UserId"] = userId }))
            {
                await _next(context);
            }
        }
    }
}
