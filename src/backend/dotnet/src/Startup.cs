using System;
using System.Collections.Generic;
using backend.Middleware;
using backend.Repository;
using Microsoft.AspNetCore.Authentication.Cookies;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.HttpOverrides;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using OpenTelemetry.Resources;
using OpenTelemetry.Trace;
using OpenTelemetry.Metrics;
using Npgsql;
using MongoDB.Driver.Core.Extensions.DiagnosticSources;

namespace backend
{
    public class Startup
    {
        public Startup(IConfiguration configuration)
        {
            Configuration = configuration;
        }

        public IConfiguration Configuration { get; }

        public void ConfigureServices(IServiceCollection services)
        {
            var databaseAddr = Configuration["DB_ADDR"];

            services.AddCors();
            services.AddControllers();

            var otlpEndpoint = Configuration["OTEL_EXPORTER_OTLP_ENDPOINT"] ?? "http://signoz-otel-collector:4317";

            services.AddOpenTelemetry()
                .WithTracing(tracing => tracing
                    .SetResourceBuilder(
                        ResourceBuilder.CreateDefault()
                            .AddService(serviceName: "dotnet-backend")
                            .AddAttributes(new Dictionary<string, object>
                            {
                                ["deployment.environment"] = "development"
                            }))
                    .AddAspNetCoreInstrumentation()
                    .AddHttpClientInstrumentation()
                    .AddNpgsql()
                    .AddSource("MongoDB.Driver.Core.Extensions.DiagnosticSources")
                    .AddOtlpExporter(options =>
                    {
                        options.Endpoint = new Uri(otlpEndpoint);
                    }))
                .WithMetrics(metrics => metrics
                    .SetResourceBuilder(
                        ResourceBuilder.CreateDefault()
                            .AddService(serviceName: "dotnet-backend")
                            .AddAttributes(new Dictionary<string, object>
                            {
                                ["deployment.environment"] = "development"
                            }))
                    .AddMeter(MinesweeperMetrics.MeterName)
                    .AddAspNetCoreInstrumentation()
                    .AddHttpClientInstrumentation()
                    .AddRuntimeInstrumentation()
                    .AddProcessInstrumentation()
                    .AddPrometheusExporter()
                    .AddOtlpExporter(options =>
                    {
                        options.Endpoint = new Uri(otlpEndpoint);
                    }));

            if (!string.IsNullOrEmpty(Configuration["Authentication:Google:ClientId"]))
            {
                services.AddAuthentication(CookieAuthenticationDefaults.AuthenticationScheme)
                    .AddCookie(o =>
                    {
                        o.LoginPath = "/account/google-login";
                        o.Cookie.SameSite = SameSiteMode.Lax;
                        o.Cookie.SecurePolicy = CookieSecurePolicy.SameAsRequest;
                        o.Events.OnRedirectToLogin = context =>
                        {
                            if (context.Request.Path.StartsWithSegments("/game") || context.Request.Path.StartsWithSegments("/user"))
                            {
                                context.Response.StatusCode = StatusCodes.Status401Unauthorized;
                            }
                            else
                            {
                                context.Response.Redirect(context.RedirectUri);
                            }
                            return System.Threading.Tasks.Task.CompletedTask;
                        };
                    })
                    .AddGoogle(o =>
                    {
                        o.ClientId = Configuration["Authentication:Google:ClientId"] ?? string.Empty;
                        o.ClientSecret = Configuration["Authentication:Google:ClientSecret"] ?? string.Empty;
                        o.CorrelationCookie.SameSite = SameSiteMode.Lax;
                        o.CorrelationCookie.SecurePolicy = CookieSecurePolicy.SameAsRequest;
                    });
            }

            services.AddAuthorization();

            var postgresConnString = Configuration.GetConnectionString("PostgresConnection");
            if (string.IsNullOrEmpty(postgresConnString))
            {
                var postgresConfig = Configuration.GetSection("Postgres").Get<PostgresConfig>();
                if (!string.IsNullOrEmpty(postgresConfig?.Host))
                {
                    if (string.IsNullOrEmpty(postgresConfig.Password))
                    {
                        throw new InvalidOperationException("Postgres:Password is required when Postgres:Host is configured.");
                    }
                    postgresConnString = postgresConfig.ToConnectionString();
                }
            }

            if (!string.IsNullOrEmpty(postgresConnString))
            {
                services.AddScoped<IGameRepository>(s => new PostgresGameRepository(s.GetRequiredService<ILogger<PostgresGameRepository>>(), postgresConnString));
                services.AddScoped<IUserGameRepository>(s => new PostgresUserGameRepository(s.GetRequiredService<ILogger<PostgresUserGameRepository>>(), postgresConnString));
            }
            else if (string.IsNullOrEmpty(databaseAddr))
            {
                services.AddSingleton<IGameRepository, InMemoryGameRepository>();
                services.AddSingleton<IUserGameRepository, InMemoryUserGameRepository>();
            }
            else
            {
                var mongoAddr = $"mongodb://{databaseAddr}";
                services.Configure<MongoConfig>(c => c.DatabaseAddress = mongoAddr);

                services.AddSingleton<MongoDB.Driver.IMongoClient>(s =>
                {
                    var settings = MongoDB.Driver.MongoClientSettings.FromUrl(new MongoDB.Driver.MongoUrl(mongoAddr));
                    settings.ConnectTimeout = TimeSpan.FromSeconds(2);
                    settings.ServerSelectionTimeout = TimeSpan.FromSeconds(2);

                    // Enable tracing for MongoDB
                    settings.ClusterConfigurator = cb => cb.Subscribe(new DiagnosticsActivityEventSubscriber());

                    return new MongoDB.Driver.MongoClient(settings);
                });

                services.AddScoped(s => s.GetRequiredService<MongoDB.Driver.IMongoClient>().GetDatabase("MinesweeperGame"));
                services.AddScoped<IGameRepository, GameRepository>();
                services.AddScoped<IUserGameRepository, UserGameRepository>();
            }
        }

        public void Configure(IApplicationBuilder app, IWebHostEnvironment env)
        {
            if (env.IsDevelopment())
            {
                app.UseDeveloperExceptionPage();
            }
            else
            {
                // In production, we use our custom global exception handler to avoid leaking PII
                app.UseMiddleware<GlobalExceptionMiddleware>();
            }

            var forwardedHeaderOptions = new ForwardedHeadersOptions
            {
                ForwardedHeaders = ForwardedHeaders.XForwardedFor | ForwardedHeaders.XForwardedProto | ForwardedHeaders.XForwardedHost
            };
            forwardedHeaderOptions.KnownNetworks.Clear();
            forwardedHeaderOptions.KnownProxies.Clear();
            app.UseForwardedHeaders(forwardedHeaderOptions);

            app.UseCors(x => x.AllowAnyOrigin().AllowAnyMethod().AllowAnyHeader());
            app.UseOpenTelemetryPrometheusScrapingEndpoint();
            app.UseRouting();

            app.UseAuthentication();
            app.UseMiddleware<RequestLoggingMiddleware>();
            app.UseAuthorization();

            app.UseEndpoints(endpoints =>
            {
                endpoints.MapControllers();
            });
        }
    }
}
