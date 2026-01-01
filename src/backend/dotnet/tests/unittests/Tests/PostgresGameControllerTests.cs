using backend.Repository;
using unittests.Factories;
using Microsoft.Extensions.DependencyInjection;
using Xunit;

namespace unittests.Tests
{
    public class PostgresGameControllerTests : ControllerTestsBase, IClassFixture<PostgresWebApiFactory>
    {
        public PostgresGameControllerTests(PostgresWebApiFactory apiFactory) : base(apiFactory.CreateClient(), apiFactory.Services.GetRequiredService<IGameRepository>())
        {
        }
    }
}
