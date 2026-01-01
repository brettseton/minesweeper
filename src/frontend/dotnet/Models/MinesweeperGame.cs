using System;
using System.Collections.Generic;
using System.Runtime.Serialization;

namespace dotnet.Models
{
    public enum GameStatus
    {
        InProgress,
        Won,
        Lost
    }

    [DataContract]
    public class MinesweeperGame
    {
        [DataMember(Name = "Id")]
        public int Id { get; set; }
        [DataMember(Name = "Board")]
        public int[][] Board { get; set; } = null!;
        [DataMember(Name = "MineCount")]
        public int MineCount { get; set; }
        [DataMember(Name = "FlagPoints")]
        public HashSet<Point> FlagPoints { get; set; } = null!;
        [DataMember(Name = "Status")]
        public GameStatus Status { get; set; }
        [DataMember(Name = "CreatedAt")]
        public DateTime CreatedAt { get; set; }
    }
}
