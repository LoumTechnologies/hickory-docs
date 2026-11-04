using System.Collections.ObjectModel;
using System.Text;
using System.Text.Json;

namespace ExplicitWeb;

public sealed class Response
{
    public int Status { get; }
    public string ContentType { get; }
    private readonly byte[] bytes;
    public ReadOnlyMemory<byte> Body => bytes;
    public IReadOnlyDictionary<string, string> Headers { get; }
    private Response(int status, string contentType, byte[] bytes, IDictionary<string, string>? headers = null)
    {
        Status = status; ContentType = contentType; this.bytes = bytes;
        Headers = new ReadOnlyDictionary<string, string>(new Dictionary<string, string>(headers ?? new Dictionary<string, string>(), StringComparer.OrdinalIgnoreCase));
    }
    public static Response Page(Html html, int status = 200) => new(status, "text/html; charset=utf-8", Encoding.UTF8.GetBytes(html.Encoded));
    public static Response Text(string text, int status = 200) => new(status, "text/plain; charset=utf-8", Encoding.UTF8.GetBytes(text));
    public static Response Json<T>(T value, int status = 200) => new(status, "application/json; charset=utf-8", JsonSerializer.SerializeToUtf8Bytes(value, JsonDefaults.Options));
    public static Response Error(Problem problem) => Json(new { error = problem.Message }, problem.Status);
    public static Response Css(string css) => new(200, "text/css; charset=utf-8", Encoding.UTF8.GetBytes(css));
    public static Response Empty(int status = 204) => new(status, "text/plain", []);
    public static Response Redirect(LocalUrl url) => Empty(303).WithHeader("Location", url.Value);
    public Response WithStatus(int status) => new(status, ContentType, bytes, Headers.ToDictionary(x => x.Key, x => x.Value));
    public Response WithHeader(string name, string value)
    {
        if (name.Length == 0 || name.Any(c => !char.IsAsciiLetterOrDigit(c) && c != '-') ||
            value.Any(c => c == '\r' || c == '\n' || c == '\0'))
            throw new ArgumentException("Use a valid HTTP header name and a value without line breaks.");
        var headers = Headers.ToDictionary(x => x.Key, x => x.Value, StringComparer.OrdinalIgnoreCase);
        headers[name] = value;
        return new Response(Status, ContentType, bytes, headers);
    }
}
