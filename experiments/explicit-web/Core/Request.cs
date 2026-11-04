using System.Collections.ObjectModel;
using System.Text;
using System.Text.Json;

namespace ExplicitWeb;

public sealed class Request
{
    public Guid Id { get; } = Guid.NewGuid();
    public string Method { get; }
    public string Path { get; }
    public string Peer { get; }
    public string? ServerOrigin { get; }
    public CancellationToken Cancel { get; }
    private readonly IReadOnlyDictionary<string, string> headers;
    private readonly byte[] body;
    public Request(string method, string path, string peer, IDictionary<string, string> headers,
        byte[] body, CancellationToken cancel = default, string? serverOrigin = null)
    {
        Method = method; Path = path; Peer = peer; Cancel = cancel; ServerOrigin = serverOrigin;
        this.headers = new ReadOnlyDictionary<string, string>(new Dictionary<string, string>(headers, StringComparer.OrdinalIgnoreCase));
        this.body = body.ToArray();
    }
    public string? Header(string name) => headers.GetValueOrDefault(name);
    public string? Cookie(string name)
    {
        var matches = (Header("Cookie") ?? "").Split(';').Select(x => x.Trim().Split('=', 2))
            .Where(x => x.Length == 2 && x[0] == name).ToArray();
        return matches.Length == 1 ? matches[0][1] : null;
    }
    public Result<IReadOnlyDictionary<string, string>> Form()
    {
        if (!(Header("Content-Type") ?? "").Split(';')[0].Equals("application/x-www-form-urlencoded", StringComparison.OrdinalIgnoreCase))
            return Result<IReadOnlyDictionary<string, string>>.Fail(415, "Send an application/x-www-form-urlencoded form.");
        try
        {
            var fields = new Dictionary<string, string>(StringComparer.Ordinal);
            var text = new UTF8Encoding(false, true).GetString(body);
            foreach (var pair in text.Split('&', StringSplitOptions.RemoveEmptyEntries))
            {
                var pieces = pair.Split('=', 2);
                string Decode(string s) => Uri.UnescapeDataString(s.Replace('+', ' '));
                if (!fields.TryAdd(Decode(pieces[0]), Decode(pieces.Length == 2 ? pieces[1] : "")))
                    return Result<IReadOnlyDictionary<string, string>>.Fail(400, "Send each form field once.");
            }
            return Result<IReadOnlyDictionary<string, string>>.Ok(new ReadOnlyDictionary<string, string>(fields));
        }
        catch (Exception e) when (e is DecoderFallbackException or UriFormatException)
        { return Result<IReadOnlyDictionary<string, string>>.Fail(400, "Send a valid UTF-8 form."); }
    }
    public Result<T> Json<T>() where T : notnull
    {
        if (!(Header("Content-Type") ?? "").Split(';')[0].Equals("application/json", StringComparison.OrdinalIgnoreCase))
            return Result<T>.Fail(415, "Send application/json.");
        try
        {
            var value = JsonSerializer.Deserialize<T>(body, JsonDefaults.Options);
            return value is null ? Result<T>.Fail(400, "Send a JSON object.") : Result<T>.Ok(value);
        }
        catch (JsonException) { return Result<T>.Fail(400, "Send valid JSON matching the endpoint fields."); }
    }
}

public static class JsonDefaults
{
    public static JsonSerializerOptions Options { get; } = new(JsonSerializerDefaults.Web)
        { UnmappedMemberHandling = System.Text.Json.Serialization.JsonUnmappedMemberHandling.Disallow };
}
