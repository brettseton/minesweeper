using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;

namespace backend.Comparers
{
    public class PointComparer : IEqualityComparer<Point>
    {
        public bool Equals(Point x, Point y)
        {
            return x?.X == y?.X && x?.Y == y?.Y;
        }

        public int GetHashCode([DisallowNull] Point p)
        {
            return 357 ^ p.X + 411 ^ p.Y;
        }
    }
}
