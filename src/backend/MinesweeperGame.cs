﻿using System;
using System.Runtime.Serialization;
using System.Collections.Generic;
using MongoDB.Bson.Serialization.Attributes;

namespace backend
{
    [BsonIgnoreExtraElements]
    [DataContract]
    public class MinesweeperGame
    {
        [DataMember(Name = "Id")]
        [BsonElement("Id")]
        public int Id {get; set;}
        
        [DataMember(Name = "Board")]
        [BsonElement("Board")]
        public int[][] Board { get; set; }

        [DataMember(Name = "Moves")]
        [BsonElement("Moves")]
        public HashSet<Point> Moves { get; set; }

        [DataMember(Name = "MineCount")]
        public int MineCount => MinePoints?.Count ?? 0;

        [DataMember(Name = "MinePoints")]
        [BsonElement("MinePoints")]
        public HashSet<Point> MinePoints { get; set; }

        [DataMember(Name = "FlagPoints")]
        [BsonElement("FlagPoints")]
        public HashSet<Point> FlagPoints { get; set; }
    }

    [BsonIgnoreExtraElements]
    [DataContract]
    public class Point {
        [DataMember(Name = "x")]
        [BsonElement("x")]
        public int x {get; set;}

        [DataMember(Name = "y")]
        [BsonElement("y")]
        public int y {get; set;}
    }
}
