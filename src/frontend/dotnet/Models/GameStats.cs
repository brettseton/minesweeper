using System.Runtime.Serialization;

namespace dotnet.Models
{
    [DataContract]
    public class GameStats
    {
        [DataMember(Name = "Won")]
        public int Won { get; set; }

        [DataMember(Name = "Lost")]
        public int Lost { get; set; }

        [DataMember(Name = "InProgress")]
        public int InProgress { get; set; }
    }
}
