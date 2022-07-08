
using System;
using System.Collections.Generic;
using System.Runtime.Serialization;

namespace dotnet.Models
{
    [DataContract]
    public class MinesweeperGame
    {
        [DataMember(Name = "Id")]
        public int Id { get; set; }
        [DataMember(Name = "Board")]
        public int[][] Board { get; set; }
        [DataMember(Name = "MineCount")]
        public int MineCount { get; set; }
        [DataMember(Name = "FlagPoints")]
        public HashSet<Point> FlagPoints { get; set; }
    }
}
