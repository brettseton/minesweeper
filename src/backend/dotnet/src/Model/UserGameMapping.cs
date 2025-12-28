using System.Runtime.Serialization;
using MongoDB.Bson.Serialization.Attributes;

namespace backend.Model
{
    [BsonIgnoreExtraElements]
    [DataContract]
    public class UserGameMapping
    {
        [BsonId]
        [BsonElement("_id")]
        public string Id { get; set; }

        [BsonElement("UserId")]
        public string UserId { get; set; }

        [BsonElement("GameId")]
        public int GameId { get; set; }
    }
}

