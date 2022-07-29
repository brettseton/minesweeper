using System.Net.Http.Json;

namespace unittests
{
    public static class HttpClientExtensions
    {

        public static async Task<T> GetAsync<T>(this HttpClient client, string requestUri)
        {
            var response = await client.GetAsync(requestUri);
            return await response.Content.ReadFromJsonAsync<T>();
        }

        public static async Task<TResponse> PostAsJsonAsync<TValue, TResponse>(this HttpClient client, string requestUri, TValue value)
        {
            var response = await client.PostAsJsonAsync(requestUri, value);
            return await response.Content.ReadFromJsonAsync<TResponse>();
        }

    }
}