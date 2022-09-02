using System.Net.Http.Json;
using unittests.Extensions;

namespace unittests.Extensions
{
    public static class HttpClientExtensions
    {

        public static async Task<T> GetAsync<T>(this HttpClient client, string requestUri)
        {
            var response = await client.GetAsync(requestUri);
            response.EnsureSuccessStatusCode();
            return await response.Content.ReadFromJsonAsync<T>() ?? throw new Exception("Response Content is null");
        }

        public static async Task<TResponse> PostAsJsonAsync<TValue, TResponse>(this HttpClient client, string requestUri, TValue value)
        {
            var response = await client.PostAsJsonAsync(requestUri, value);
            response.EnsureSuccessStatusCode();
            return await response.Content.ReadFromJsonAsync<TResponse>() ?? throw new Exception("Response Content is null");
        }

    }
}