using backend;
using backend.Controllers;
using backend.Repository;
using DotNet.Testcontainers.Builders;
using DotNet.Testcontainers.Configurations;
using DotNet.Testcontainers.Containers;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Mvc.Testing;
using Microsoft.AspNetCore.TestHost;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.DependencyInjection.Extensions;
using Microsoft.Extensions.Options;

namespace unittests
{
    public class MongoDBWebApiFactory : WebApplicationFactory<GameController>, IAsyncLifetime
    {
        private readonly MongoDbTestcontainer _dbContainer =
            new TestcontainersBuilder<MongoDbTestcontainer>()
            .WithDatabase(new MongoDbTestcontainerConfiguration()
            {
                Database = "Minesweeper",
                Username = "test",
                Password = "test123"
            })
            .Build();

        public async Task InitializeAsync()
        {
            await _dbContainer.StartAsync();
        }

        protected override void ConfigureWebHost(IWebHostBuilder builder)
        {
            builder.ConfigureTestServices(services =>
            {
                services.RemoveAll(typeof(IOptionsMonitor<MongoConfig>));
                services.RemoveAll(typeof(IGameRepository));
                services.Configure<MongoConfig>(c => c.DatabaseAddress = _dbContainer.ConnectionString);
                services.AddSingleton<IGameRepository, GameRepository>();
            });
        }

        async Task IAsyncLifetime.DisposeAsync()
        {
            await _dbContainer.StopAsync();
        }
    }
}
