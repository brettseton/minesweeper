using backend.Controllers;
using backend.Repository;
using Microsoft.AspNetCore.Authentication;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Mvc.Testing;
using Microsoft.AspNetCore.TestHost;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.DependencyInjection.Extensions;
using unittests.Authentication;

namespace unittests.Factories
{
    public class InMemoryWebApiFactory : WebApplicationFactory<GameController>
    {
        protected override void ConfigureWebHost(IWebHostBuilder builder)
        {
            builder.ConfigureTestServices(services =>
            {
                services.RemoveAll(typeof(IGameRepository));
                services.AddSingleton<IGameRepository, InMemoryGameRepository>();
                services.AddAuthentication("Test")
                        .AddScheme<AuthenticationSchemeOptions, AuthenticationTestHandler>("Test", null);
            });
        }
    }
}
