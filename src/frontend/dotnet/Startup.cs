using System;
using frontend;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;

namespace dotnet
{
    public class Startup
    {
        private EnvironmentConfiguration envConfig;

        public Startup(IConfiguration configuration)
        {
            Configuration = configuration;
            envConfig = new EnvironmentConfiguration();
        }

        public IConfiguration Configuration { get; }

        // This method gets called by the runtime. Use this method to add services to the container.
        public void ConfigureServices(IServiceCollection services)
        {
            services.Configure<CookiePolicyOptions>(options =>
            {
                // This lambda determines whether user consent for non-essential cookies is needed for a given request.
                options.CheckConsentNeeded = context => true;
                options.MinimumSameSitePolicy = SameSiteMode.None;
            });

            services.AddHttpClient();
            services.AddSingleton<IEnvironmentConfiguration>(envConfig);

            services.AddLogging();

            services.AddControllersWithViews();
        }

        // This method gets called by the runtime. Use this method to configure the HTTP request pipeline.
        public void Configure(IApplicationBuilder app,  IWebHostEnvironment env, ILogger<Startup> logger)
        {
            // API_ADDR environment variable is provided in frontend.deployment.yaml.
            var backendAddr = Environment.GetEnvironmentVariable("BACKEND_API_ADDR");
            logger.LogInformation($"Backend address is set to {backendAddr}");
            if (string.IsNullOrEmpty(backendAddr))
            {
                throw new ArgumentException("API_ADDR environment variable is not set");
            }

            // PORT environment variable is provided in frontend.deployment.yaml.
            var port = Environment.GetEnvironmentVariable("BACKEND_PORT");
            logger.LogInformation($"Port is set to {port}");
            if (string.IsNullOrEmpty(port))
            {
                throw new ArgumentException("PORT environment variable is not set");
            }

            bool.TryParse(Environment.GetEnvironmentVariable("IS_HTTPS"), out bool isHttps);
            logger.LogInformation($"Is Https is set to {isHttps}");

            // Set the address of the backend microservice
            envConfig.BackendGameAddress = $"http{(isHttps ? "s" : "")}://{backendAddr}:{port}/game";

            if (env.IsDevelopment())
            {
                app.UseDeveloperExceptionPage();
            }
            else
            {
                app.UseExceptionHandler("/Home/Error");
            }

            app.UseStaticFiles();
            app.UseCookiePolicy();

            app.UseRouting();

            app.UseEndpoints(endpoints =>
            {
                endpoints.MapControllerRoute(
                    name: "default",
                    pattern: "{controller=Home}/{action=Index}");
                
                endpoints.MapControllerRoute(
                    name: "game",
                    pattern: "{controller=Game}/{action=Game}/{id?}");

            });

        }
    }
}
