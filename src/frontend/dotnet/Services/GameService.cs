using System;
using System.Collections.Generic;
using System.Net.Http;
using System.Text;
using System.Threading.Tasks;
using dotnet.Models;
using frontend;
using Microsoft.Extensions.Logging;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace dotnet.Services
{
    public class GameService : IGameService
    {
        private readonly HttpClient _httpClient;
        private readonly IEnvironmentConfiguration _envConfig;
        private readonly ILogger<GameService> _logger;
        private readonly JsonSerializerOptions _jsonOptions;

        public GameService(IHttpClientFactory httpClientFactory, IEnvironmentConfiguration envConfig, ILogger<GameService> logger)
        {
            _httpClient = httpClientFactory.CreateClient("BackendClient");
            _envConfig = envConfig;
            _logger = logger;
            _jsonOptions = new JsonSerializerOptions 
            { 
                PropertyNameCaseInsensitive = true,
                Converters = { new JsonStringEnumConverter() }
            };
        }

        public async Task<MinesweeperGame> GetGame(int id)
        {
            var response = await _httpClient.GetAsync($"{_envConfig.BackendAddress}/game/{id}");
            response.EnsureSuccessStatusCode();
            return await JsonSerializer.DeserializeAsync<MinesweeperGame>(await response.Content.ReadAsStreamAsync(), _jsonOptions);
        }

        public async Task<MinesweeperGame> NewGame()
        {
            var response = await _httpClient.GetAsync($"{_envConfig.BackendAddress}/game/new");
            response.EnsureSuccessStatusCode();
            return await JsonSerializer.DeserializeAsync<MinesweeperGame>(await response.Content.ReadAsStreamAsync(), _jsonOptions);
        }

        public async Task<MinesweeperGame> MakeMove(int gameId, int x, int y)
        {
            var content = new StringContent(JsonSerializer.Serialize(new { x, y }), Encoding.UTF8, "application/json");
            var response = await _httpClient.PostAsync($"{_envConfig.BackendAddress}/game/{gameId}", content);
            response.EnsureSuccessStatusCode();
            return await JsonSerializer.DeserializeAsync<MinesweeperGame>(await response.Content.ReadAsStreamAsync(), _jsonOptions);
        }

        public async Task<MinesweeperGame> ToggleFlag(int gameId, int x, int y)
        {
            var content = new StringContent(JsonSerializer.Serialize(new { x, y }), Encoding.UTF8, "application/json");
            var response = await _httpClient.PostAsync($"{_envConfig.BackendAddress}/game/flag/{gameId}", content);
            response.EnsureSuccessStatusCode();
            return await JsonSerializer.DeserializeAsync<MinesweeperGame>(await response.Content.ReadAsStreamAsync(), _jsonOptions);
        }

        public async Task<List<MinesweeperGame>> GetUserGames()
        {
            var response = await _httpClient.GetAsync($"{_envConfig.BackendAddress}/user/games");
            if (!response.IsSuccessStatusCode) return new List<MinesweeperGame>();
            return await JsonSerializer.DeserializeAsync<List<MinesweeperGame>>(await response.Content.ReadAsStreamAsync(), _jsonOptions);
        }

        public async Task<GameStats> GetUserStats()
        {
            var response = await _httpClient.GetAsync($"{_envConfig.BackendAddress}/user/stats");
            if (!response.IsSuccessStatusCode) return new GameStats();
            return await JsonSerializer.DeserializeAsync<GameStats>(await response.Content.ReadAsStreamAsync(), _jsonOptions);
        }

        public async Task<(bool IsAuthenticated, string Name)> GetUserStatus()
        {
            try 
            {
                var response = await _httpClient.GetAsync($"{_envConfig.BackendAddress}/account/status");
                if (response.IsSuccessStatusCode)
                {
                    var status = await JsonSerializer.DeserializeAsync<AuthStatus>(await response.Content.ReadAsStreamAsync(), _jsonOptions);
                    return (status.IsAuthenticated, status.Name);
                }
            }
            catch (Exception e)
            {
                _logger.LogWarning($"Failed to fetch auth status: {e.Message}");
            }
            return (false, null);
        }

        private class AuthStatus
        {
            public bool IsAuthenticated { get; set; }
            public string Name { get; set; }
        }
    }
}