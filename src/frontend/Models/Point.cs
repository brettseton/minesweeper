using System.Runtime.Serialization;

namespace dotnet.Models
{
    [DataContract]
    public class Point
    {
        public int x {get; set;}
        public int y {get; set;}
    }
}
