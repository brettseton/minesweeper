using System.Collections.Generic;
using backend.Model;

namespace backend.Repository
{
    public interface IUserGameRepository
    {
        void AddMapping(string userId, int gameId);
        IEnumerable<int> GetGameIdsByUserId(string userId);
    }
}

