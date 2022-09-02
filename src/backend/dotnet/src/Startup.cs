using System;
using backend.Repository;
using Microsoft.AspNetCore.Authentication.Cookies;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.HttpOverrides;
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

            // PORT environment variable is provided in backend.deployment.yaml.
            var port = Environment.GetEnvironmentVariable("PORT");

            services.AddControllers();
            if (Configuration["Authentication:Google:ClientId"] != null)
            {
                services.AddAuthentication(CookieAuthenticationDefaults.AuthenticationScheme)
                    .AddCookie(o =>
                    {
                        o.LoginPath = "/account/google-login";
                    })
                    .AddGoogle(o =>
                    {
                        o.ClientId = Configuration["Authentication:Google:ClientId"];
                        o.ClientSecret = Configuration["Authentication:Google:ClientSecret"];
                    });
            }

            services.AddAuthorization();

            if (string.IsNullOrEmpty(port) || string.IsNullOrEmpty(databaseAddr))
            {
                services.AddSingleton<IGameRepository, InMemoryGameRepository>();
            }
            else
            {
                // Pass the configuration for connecting to MongoDB to Dependency Injection container
                services.Configure<MongoConfig>(c => c.DatabaseAddress = $"mongodb://{databaseAddr}");
                services.AddScoped<IGameRepository, GameRepository>();
            }
        }

        // This method gets called by the runtime. Use this method to configure the HTTP request pipeline.
        public void Configure(IApplicationBuilder app, IWebHostEnvironment env)
        {
            if (env.IsDevelopment())
            {
                app.UseDeveloperExceptionPage();
            }
            app.UseForwardedHeaders(new ForwardedHeadersOptions
            {
                ForwardedHeaders = ForwardedHeaders.XForwardedProto
            });
            app.UseRouting();
            app.UseAuthentication();
            app.UseAuthorization();
            app.UseEndpoints(endpoints =>
            {
                endpoints.MapControllers();
            });
        }
    }
}
