using System.ComponentModel.DataAnnotations;

namespace backend
{
    public class PostgresConfig
    {
        public string? Host { get; set; }
        
        public string Database { get; set; } = "minesweeper";
        
        public string Username { get; set; } = "postgres";
        
        [Required(AllowEmptyStrings = false)]
        public string? Password { get; set; }

        public string ToConnectionString()
        {
            var builder = new Npgsql.NpgsqlConnectionStringBuilder
            {
                Host = Host,
                Database = Database,
                Username = Username,
                Password = Password,
                Pooling = true
            };
            return builder.ToString();
        }
    }
}
