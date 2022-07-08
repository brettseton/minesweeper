using System;
using System.Collections.Generic;

namespace backend.Extensions
{
    public static class IEnumerableExtensions
    {
        private static readonly Random random = new();
        public static void Shuffle<T>(this IList<T> list)
        {
            int n = list.Count;
            while (n > 1)
            {
                n--;
                int k = random.Next(n + 1);
                (list[n], list[k]) = (list[k], list[n]);
            }
        }
    }
}