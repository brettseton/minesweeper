using backend.Model;

namespace backend.Repository
{
    public interface IGameRepository
    {
        MinesweeperGame? GetGame(int id);
        void Save(MinesweeperGame entry);
        MinesweeperGame AddMoves(int id, Point[] points);
        MinesweeperGame AddFlag(int id, Point point);
        MinesweeperGame RemoveFlag(int id, Point point);
        IEnumerable<MinesweeperGame> GetGamesByIds(IEnumerable<int> ids);
    }
}
