using System;
using System.Collections.Generic;
using System.Runtime.Serialization;
using System.Text.Json.Serialization;
using MongoDB.Bson.Serialization.Attributes;

namespace backend.Model
{
    [BsonIgnoreExtraElements]
    [DataContract]
    public class MinesweeperGame
    {
        [DataMember(Name = "Id")]
        [BsonElement("Id")]
        public int Id { get; set; }

        [DataMember(Name = "Board")]
        [BsonElement("Board")]
        public BoardState[][] Board { get; set; }

        [DataMember(Name = "Moves")]
        [BsonElement("Moves")]
        public HashSet<Point> Moves { get; set; }

        [DataMember(Name = "MineCount")]
        public int MineCount => MinePoints?.Count ?? 0;

        //[BsonElement("MinePoints")]
        [Newtonsoft.Json.JsonIgnore]
        [JsonIgnore]
        public HashSet<Point> MinePoints { get; set; }

        [DataMember(Name = "FlagPoints")]
        [BsonElement("FlagPoints")]
        public HashSet<Point> FlagPoints { get; set; }
    }

    [DataContract]
    public class MinesweeperGameDto
    {
        public int Id { get; set; }

        public BoardState[][] Board { get; set; }

        public int MineCount { get; set; }

        public HashSet<Point> FlagPoints { get; set; }
    }

    [BsonIgnoreExtraElements]
    [DataContract]
    public class Point
    {
        public Point()
        {
        }

        public Point(int x, int y)
        {
            X = x;
            Y = y;
        }

        [DataMember(Name = "x")]
        [BsonElement("x")]
        public int X { get; set; }

        [DataMember(Name = "y")]
        [BsonElement("y")]
        public int Y { get; set; }

        public override bool Equals(Object obj)
        {
            if (obj is not Point other)
                return false;
            else
                return X == other.X && Y == other.Y;
        }

        public override int GetHashCode()
        {
            return (X * 31) ^ (Y * 411);
        }
    }
}
