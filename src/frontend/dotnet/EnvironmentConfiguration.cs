namespace frontend
{
    public interface IEnvironmentConfiguration
    {
        string BackendAddress { get; set; }
        string BackendGameAddress { get; set; }
    }

    public class EnvironmentConfiguration : IEnvironmentConfiguration
    {
        public string BackendAddress { get; set; }
        public string BackendGameAddress { get; set; }
    }
}
