using backend;
using backend.Controllers;
using backend.Repository;
using DotNet.Testcontainers.Builders;
using DotNet.Testcontainers.Configurations;
using DotNet.Testcontainers.Containers;
using Microsoft.AspNetCore.Authentication;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Mvc.Testing;
using Microsoft.AspNetCore.TestHost;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.DependencyInjection.Extensions;
using Microsoft.Extensions.Options;
using unittests.Authentication;

namespace unittests.Factories
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
                services.RemoveAll(typeof(IGameRepository));
                services.RemoveAll(typeof(IUserGameRepository));
                services.RemoveAll(typeof(MongoDB.Driver.IMongoDatabase));

                var client = new MongoDB.Driver.MongoClient(_dbContainer.ConnectionString);
                var database = client.GetDatabase("MinesweeperGame");
                
                services.AddSingleton<MongoDB.Driver.IMongoClient>(client);
                services.AddSingleton<MongoDB.Driver.IMongoDatabase>(database);

                services.AddSingleton<IGameRepository, GameRepository>();
                services.AddSingleton<IUserGameRepository, UserGameRepository>();
                services.AddAuthentication("Test")
                        .AddScheme<AuthenticationSchemeOptions, AuthenticationTestHandler>("Test", null);
            });
        }

        async Task IAsyncLifetime.DisposeAsync()
        {
            await _dbContainer.StopAsync();
        }
    }
}
