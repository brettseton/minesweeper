using backend.Controllers;
using backend.Repository;
using Microsoft.AspNetCore.Authentication;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Mvc.Testing;
using Microsoft.AspNetCore.TestHost;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.DependencyInjection.Extensions;
using Microsoft.Extensions.Logging;
using Testcontainers.PostgreSql;
using unittests.Authentication;
using Xunit;

namespace unittests.Factories
{
    public class PostgresWebApiFactory : WebApplicationFactory<GameController>, IAsyncLifetime
    {
        private readonly PostgreSqlContainer _dbContainer =
            new PostgreSqlBuilder()
                .WithImage("postgres:15")
                .Build();

        protected override void ConfigureWebHost(IWebHostBuilder builder)
        {
            builder.ConfigureTestServices(services =>
            {
                services.RemoveAll(typeof(IGameRepository));
                services.RemoveAll(typeof(IUserGameRepository));

                var connString = _dbContainer.GetConnectionString();
                services.AddSingleton<IGameRepository>(s => new PostgresGameRepository(s.GetRequiredService<ILogger<PostgresGameRepository>>(), connString));
                services.AddSingleton<IUserGameRepository>(s => new PostgresUserGameRepository(s.GetRequiredService<ILogger<PostgresUserGameRepository>>(), connString));

                services.AddAuthentication("Test")
                        .AddScheme<AuthenticationSchemeOptions, AuthenticationTestHandler>("Test", null);
            });
        }

        public async Task InitializeAsync()
        {
            await _dbContainer.StartAsync();
        }

        public new async Task DisposeAsync()
        {
            await _dbContainer.StopAsync();
        }
    }
}
