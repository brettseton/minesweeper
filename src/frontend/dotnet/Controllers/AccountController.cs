using System;
using System.Linq;
using System.Net.Http;
using System.Threading.Tasks;
using frontend;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Logging;

namespace dotnet.Controllers
{
    public class AccountController : Controller
    {
        private readonly IHttpClientFactory _factory;
        private readonly IEnvironmentConfiguration _envConfig;
        private readonly ILogger<AccountController> _logger;

        public AccountController(IHttpClientFactory factory, IEnvironmentConfiguration envConfig, ILogger<AccountController> logger)
        {
            _factory = factory;
            _envConfig = envConfig;
            _logger = logger;
        }

        [Route("account/google-login")]
        public async Task GoogleLogin()
        {
            await ProxyToBackend("account/google-login");
        }

        [Route("account/status")]
        public async Task Status()
        {
            await ProxyToBackend("account/status");
        }

        [Route("signin-google")]
        public async Task SignInGoogle()
        {
            await ProxyToBackend("signin-google");
        }

        [HttpPost, Route("account/google-logout")]
        public async Task<IActionResult> GoogleLogout()
        {
            try
            {
                var httpClient = _factory.CreateClient("Proxy");
                var requestMessage = new HttpRequestMessage(HttpMethod.Post, $"{_envConfig.BackendAddress}/account/google-logout");

                // Forward headers (cookies are crucial)
                foreach (var header in Request.Headers)
                {
                    if (header.Key.Equals("Host", StringComparison.OrdinalIgnoreCase) ||
                        header.Key.Equals("Transfer-Encoding", StringComparison.OrdinalIgnoreCase))
                    {
                        continue;
                    }
                    if (!requestMessage.Headers.TryAddWithoutValidation(header.Key, header.Value.ToArray()))
                    {
                        requestMessage.Content?.Headers.TryAddWithoutValidation(header.Key, header.Value.ToArray());
                    }
                }

                var response = await httpClient.SendAsync(requestMessage);

                // Forward Set-Cookie headers to clear the session
                if (response.Headers.TryGetValues("Set-Cookie", out var cookies))
                {
                    foreach (var cookie in cookies)
                    {
                        Response.Headers.Append("Set-Cookie", cookie);
                    }
                }

                return Redirect("~/");
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Error during logout");
                return Redirect("~/");
            }
        }

        private async Task ProxyToBackend(string path, HttpMethod method = null)
        {
            try 
            {
                method ??= new HttpMethod(Request.Method);
                var httpClient = _factory.CreateClient("Proxy");

                var requestMessage = new HttpRequestMessage(method, $"{_envConfig.BackendAddress}/{path}{Request.QueryString}");

                // Forward headers
                foreach (var header in Request.Headers)
                {
                    // Skip Host header (let HttpClient set it) and Transfer-Encoding (hop-by-hop)
                    if (header.Key.Equals("Host", StringComparison.OrdinalIgnoreCase) ||
                        header.Key.Equals("Transfer-Encoding", StringComparison.OrdinalIgnoreCase))
                    {
                        continue;
                    }

                    if (!requestMessage.Headers.TryAddWithoutValidation(header.Key, header.Value.ToArray()))
                    {
                        requestMessage.Content?.Headers.TryAddWithoutValidation(header.Key, header.Value.ToArray());
                    }
                }

                // Set Forwarded headers so backend knows the original host
                requestMessage.Headers.Add("X-Forwarded-Host", Request.Host.Value);
                requestMessage.Headers.Add("X-Forwarded-Proto", Request.Scheme);
                requestMessage.Headers.Add("X-Forwarded-For", Request.HttpContext.Connection.RemoteIpAddress?.ToString());

                if (Request.ContentLength > 0 && (method == HttpMethod.Post || method == HttpMethod.Put))
                {
                    requestMessage.Content = new StreamContent(Request.Body);
                }

                var response = await httpClient.SendAsync(requestMessage);

                Response.StatusCode = (int)response.StatusCode;
                foreach (var header in response.Headers)
                {
                    // Skip Transfer-Encoding on response as well
                    if (header.Key.Equals("Transfer-Encoding", StringComparison.OrdinalIgnoreCase)) continue;
                    Response.Headers[header.Key] = header.Value.ToArray();
                }
                foreach (var header in response.Content.Headers)
                {
                    Response.Headers[header.Key] = header.Value.ToArray();
                }

                if (response.Content != null)
                {
                    await response.Content.CopyToAsync(Response.Body);
                }
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, $"Proxy error for path {path}");
                Response.StatusCode = 500;
                await Response.WriteAsync($"Proxy error: {ex.Message}");
            }
        }
    }
}
