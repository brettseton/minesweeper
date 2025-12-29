using System.Collections.Generic;
using System.Threading.Tasks;
using dotnet.Models;

namespace dotnet.Services
{
    public interface IGameService
    {
        Task<MinesweeperGame> GetGame(int id);
        Task<MinesweeperGame> NewGame();
        Task<MinesweeperGame> MakeMove(int gameId, int x, int y);
        Task<MinesweeperGame> ToggleFlag(int gameId, int x, int y);
        Task<List<MinesweeperGame>> GetUserGames();
        Task<GameStats> GetUserStats();
        Task<(bool IsAuthenticated, string Name)> GetUserStatus();
    }
}
