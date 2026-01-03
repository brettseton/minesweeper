using System;
using System.Net.Http;
using dotnet.Infrastructure;
using dotnet.Services;
using frontend;
using Microsoft.AspNetCore.Authentication.Cookies;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using OpenTelemetry.Resources;
using OpenTelemetry.Trace;
using OpenTelemetry.Metrics;

namespace dotnet
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
            var envConfig = new EnvironmentConfiguration();

            // BACKEND_API_ADDR environment variable is provided in frontend.deployment.yaml.
            var backendAddr = Configuration["BACKEND_API_ADDR"];
            var port = Configuration["BACKEND_PORT"];
            bool.TryParse(Configuration["IS_HTTPS"], out bool isHttps);

            if (string.IsNullOrEmpty(backendAddr))
            {
                throw new ArgumentException("BACKEND_API_ADDR environment variable is not set");
            }

            if (string.IsNullOrEmpty(port))
            {
                throw new ArgumentException("BACKEND_PORT environment variable is not set");
            }

            // Set the address of the backend microservice
            envConfig.BackendAddress = $"http{(isHttps ? "s" : "")}://{backendAddr}:{port}";
            envConfig.BackendGameAddress = $"{envConfig.BackendAddress}/game";

            services.Configure<CookiePolicyOptions>(options =>
            {
                // This lambda determines whether user consent for non-essential cookies is needed for a given request.
                options.CheckConsentNeeded = context => true;
                options.MinimumSameSitePolicy = SameSiteMode.None;
            });

            services.AddHttpContextAccessor();
            services.AddTransient<CookieForwardingHandler>();

            services.AddHttpClient("BackendClient")
                .AddHttpMessageHandler<CookieForwardingHandler>();

            services.AddHttpClient("Proxy")
                .ConfigurePrimaryHttpMessageHandler(() => new HttpClientHandler
                {
                    AllowAutoRedirect = false
                });

            services.AddScoped<IGameService, GameService>();
            services.AddSingleton<IEnvironmentConfiguration>(envConfig);

            var otlpEndpoint = Configuration["OTEL_EXPORTER_OTLP_ENDPOINT"] ?? "http://signoz-otel-collector:4317";

            services.AddOpenTelemetry()
                .WithTracing(tracing => tracing
                    .SetResourceBuilder(
                        ResourceBuilder.CreateDefault()
                            .AddService(serviceName: "dotnet-frontend")
                            .AddAttributes(new Dictionary<string, object>
                            {
                                ["deployment.environment"] = "development"
                            }))
                    .AddAspNetCoreInstrumentation()
                    .AddHttpClientInstrumentation()
                    .AddOtlpExporter(options =>
                    {
                        options.Endpoint = new Uri(otlpEndpoint);
                    }))
                .WithMetrics(metrics => metrics
                    .SetResourceBuilder(
                        ResourceBuilder.CreateDefault()
                            .AddService(serviceName: "dotnet-frontend")
                            .AddAttributes(new Dictionary<string, object>
                            {
                                ["deployment.environment"] = "development"
                            }))
                    .AddAspNetCoreInstrumentation()
                    .AddHttpClientInstrumentation()
                    .AddRuntimeInstrumentation()
                    .AddProcessInstrumentation()
                    .AddOtlpExporter(options =>
                    {
                        options.Endpoint = new Uri(otlpEndpoint);
                    }));

            services.AddLogging();
            services.AddControllersWithViews();
        }

        // This method gets called by the runtime. Use this method to configure the HTTP request pipeline.
        public void Configure(IApplicationBuilder app, IWebHostEnvironment env, ILogger<Startup> logger)
        {
            var envConfig = app.ApplicationServices.GetRequiredService<IEnvironmentConfiguration>();
            logger.LogInformation($"Backend address is set to {envConfig.BackendAddress}");

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
