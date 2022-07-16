using System;
using backend.Repository;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;

namespace backend
{
    public class Startup
    {
        public Startup(IConfiguration configuration)
        {
            Configuration = configuration;
        }

        public IConfiguration Configuration { get; }

        // This method gets called by the runtime. Use this method to add services to the container.
        public void ConfigureServices(IServiceCollection services)
        {
            // DB_ADDR environment variable is provided in backend.deployment.yaml.
            var databaseAddr = Environment.GetEnvironmentVariable("DB_ADDR");
            if (string.IsNullOrEmpty(databaseAddr))
            {
                throw new ArgumentException("DB_ADDR environment variable is not set");
            }

            // PORT environment variable is provided in backend.deployment.yaml.
            var port = Environment.GetEnvironmentVariable("PORT");
            if (string.IsNullOrEmpty(port))
            {
                throw new ArgumentException("PORT environment variable is not set");
            }

            services.AddControllers();

            // Pass the configuration for connecting to MongoDB to Dependency Injection container
            services.Configure<MongoConfig>(c => c.DatabaseAddress = $"mongodb://{databaseAddr}");

            //services.AddScoped<IGameRepository, GameRepository>();
            services.AddSingleton<IGameRepository, InMemoryGameRepository>();
        }

        // This method gets called by the runtime. Use this method to configure the HTTP request pipeline.
        public void Configure(IApplicationBuilder app, IWebHostEnvironment env)
        {
            if (env.IsDevelopment())
            {
                app.UseDeveloperExceptionPage();
            }

            app.UseRouting();

            app.UseEndpoints(endpoints =>
            {
                endpoints.MapControllers();
            });
        }
    }
}
