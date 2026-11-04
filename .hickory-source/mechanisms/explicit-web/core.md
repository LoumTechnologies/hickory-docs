<web:doc xmlns:web="http://www.hickorydocs.com/1.0">
# Construct checked HTTP values

For application authors following untrusted input into a response. No application policy is registered with a host pipeline. A request is a bounded value; validation failures are values too. C# supplies construction constraints, not linear types or a proof against deliberate reflection, null suppression or bypassing these libraries.

## Core core csproj

Keep assembly boundaries real: internal constructors cannot be called by consumers. Nullable warnings are errors; deterministic builds and dependency locks make evaluation reproducible.

<web:copy id="ew-Core-Core-csproj">
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup><PackageId>ExplicitWeb.Core</PackageId>
    <TargetFramework>net10.0</TargetFramework><Nullable>enable</Nullable><ImplicitUsings>enable</ImplicitUsings>
    <TreatWarningsAsErrors>true</TreatWarningsAsErrors><UseAppHost>false</UseAppHost>
    <RestorePackagesWithLockFile>true</RestorePackagesWithLockFile><Deterministic>true</Deterministic>
  </PropertyGroup>
  <ItemGroup></ItemGroup>
</Project>
</web:copy>

## Result

Require a success and failure branch at the boundary. The error contains a public message; the hosting adapter logs unexpected exceptions separately.

<web:copy id="ew-result">
namespace ExplicitWeb;

public sealed record Problem(int Status, string Message);
public sealed class Result<T> where T : notnull
{
    private readonly T? value;
    private readonly Problem? problem;
    private Result(T value) => this.value = value;
    private Result(Problem problem) => this.problem = problem;
    public static Result<T> Ok(T value) => new(value ?? throw new ArgumentNullException(nameof(value)));
    public static Result<T> Fail(int status, string message) => new(new Problem(status, message));
    public R Match<R>(Func<T, R> success, Func<Problem, R> failure)
        => problem is null ? success(value!) : failure(problem);
    public Result<R> Bind<R>(Func<T, Result<R>> next) where R : notnull
        => Match(next, p => Result<R>.Fail(p.Status, p.Message));
}
</web:copy>

## Request

Read the body once in the host and pass immutable snapshots. Form parsing rejects duplicate keys instead of silently choosing one. JSON parsing reports invalid input without exposing a parser stack trace. Bound bodies prevent parsing an unbounded upload; multipart is outside the v1 contract.

<web:copy id="ew-request">
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
</web:copy>

## Html

Encode every interpolated value, including attribute values. Only an already checked Html value composes without encoding. Templates use quoted attributes and never place input in script, CSS, tag names or attribute names. URL validation is separate because HTML encoding alone does not make a javascript URL safe.

<web:copy id="ew-html">
using System.Globalization;
using System.Net;
using System.Text;

namespace ExplicitWeb;

public sealed class Html
{
    public string Encoded { get; }
    private Html(string encoded) => Encoded = encoded;
    public static Html Text(string value) => new(WebUtility.HtmlEncode(value));
    public static Html Template(FormattableString template)
    {
        var arguments = template.GetArguments().Select(x => x is Html html ? html.Encoded :
            WebUtility.HtmlEncode(Convert.ToString(x, CultureInfo.InvariantCulture) ?? "")).Cast<object>().ToArray();
        return new Html(string.Format(CultureInfo.InvariantCulture, template.Format, arguments));
    }
    public static Html Join(IEnumerable<Html> fragments) => new(string.Concat(fragments.Select(x => x.Encoded)));
}

public sealed class LocalUrl
{
    public string Value { get; }
    private LocalUrl(string value) => Value = value;
    public static Result<LocalUrl> Parse(string? value)
    {
        if (string.IsNullOrEmpty(value) || value[0] != '/' || value.StartsWith("//", StringComparison.Ordinal) ||
            value.Any(c => char.IsControl(c) || c == '\\' || c == '%'))
            return Result<LocalUrl>.Fail(400, "Use an absolute local path without encoded characters.");
        return Result<LocalUrl>.Ok(new LocalUrl(value));
    }
}
</web:copy>

## Response

There is no public raw HTML response constructor. Redirects require a checked local URL. Arbitrary header strings are validated before the host sees them; cookies enter through the security library. Responses remain buffered so exceptions cannot leave half a successful page on the wire.

<web:copy id="ew-response">
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
</web:copy>

## Routes

Routes are typed values that both match requests and build links. A route parameter is decoded and parsed once; handlers receive the parsed type. The application inspects routes explicitly; there is no registration table or implicit ordering. A GUID route cannot be linked with a string.

<web:copy id="ew-routes">
namespace ExplicitWeb;

public sealed record Unit;
public sealed record RouteMatch<T>(T Value) where T : notnull;
public sealed class Route<T> where T : notnull
{
    public string Method { get; }
    public string Pattern { get; }
    private readonly Func<string, Result<T>> parse;
    private readonly Func<T, string> print;
    public Route(string method, string pattern, Func<string, Result<T>> parse, Func<T, string> print)
    {
        if (!pattern.StartsWith('/') || pattern.Count(c => c == '{') != pattern.Count(c => c == '}') || pattern.Count(c => c == '{') > 1 ||
            (pattern.Contains('{') && !pattern.EndsWith("/{id}", StringComparison.Ordinal)))
            throw new ArgumentException("Use a local route with at most one final /{id} parameter.");
        Method = method; Pattern = pattern; this.parse = parse; this.print = print;
    }
    public bool PathMatches(string path) => Pattern.EndsWith("/{id}", StringComparison.Ordinal)
        ? path.StartsWith(Pattern[..^4], StringComparison.Ordinal) && path.Length > Pattern.Length - 4 && !path[(Pattern.Length - 4)..].Contains('/')
        : path == Pattern;
    public Result<RouteMatch<T>> Match(Request request)
    {
        if (!PathMatches(request.Path)) return Result<RouteMatch<T>>.Fail(404, "Route not found. Follow a link from the home page.");
        if (request.Method != Method && !(request.Method == "HEAD" && Method == "GET"))
            return Result<RouteMatch<T>>.Fail(405, $"Use {Method} for this route.");
        var raw = Pattern.EndsWith("/{id}", StringComparison.Ordinal) ? request.Path[(Pattern.Length - 4)..] : "";
        return parse(raw).Match(x => Result<RouteMatch<T>>.Ok(new(x)), p => Result<RouteMatch<T>>.Fail(p.Status, p.Message));
    }
    public LocalUrl Link(T value)
    {
        var path = Pattern.EndsWith("/{id}", StringComparison.Ordinal) ? Pattern[..^4] + Uri.EscapeDataString(print(value)) : Pattern;
        return LocalUrl.Parse(path).Match(x => x, p => throw new InvalidOperationException(p.Message));
    }
}
public static class Routes
{
    public static Route<Unit> At(string method, string path) => new(method, path, _ => Result<Unit>.Ok(new()), _ => "");
    public static Route<Guid> Id(string method, string path) => new(method, path,
        s => Guid.TryParse(s, out var id) ? Result<Guid>.Ok(id) : Result<Guid>.Fail(400, "Use a valid item ID."), id => id.ToString());
}
</web:copy>

## Cors

A host handler must return Prepared, so an unchecked Response does not satisfy the transport delegate. Apply carries the origin decision onto successes and errors alike. Denied decisions override attempted success. Credentialed CORS accepts the configured exact origin and the bound socket origin supplied by the host, never an untrusted Host header; preflight accepts only declared methods and headers. This server deliberately refuses disallowed origins before application work.

<web:copy id="ew-cors">
namespace ExplicitWeb;

public sealed class Cors
{
    private readonly HashSet<string> origins;
    private readonly HashSet<string> methods;
    private readonly HashSet<string> headers;
    public Cors(IEnumerable<Uri> origins, IEnumerable<string> methods, IEnumerable<string> headers)
    {
        this.origins = new(StringComparer.Ordinal);
        foreach (var origin in origins)
        {
            if (!origin.IsAbsoluteUri || origin.Scheme is not ("http" or "https") || origin.AbsolutePath != "/" || origin.Query != "" || origin.Fragment != "" || origin.UserInfo != "")
                throw new ArgumentException("Configure HTTP(S) origins without paths, queries or credentials.");
            this.origins.Add(origin.GetLeftPart(UriPartial.Authority));
        }
        this.methods = new(methods, StringComparer.Ordinal);
        this.headers = new(headers, StringComparer.OrdinalIgnoreCase);
    }
    public Decision Inspect(Request request)
    {
        var origin = request.Header("Origin");
        bool denied = origin is not null && origin != request.ServerOrigin && !origins.Contains(origin);
        bool preflight = origin is not null && request.Method == "OPTIONS" && request.Header("Access-Control-Request-Method") is not null;
        if (preflight)
        {
            denied |= !methods.Contains(request.Header("Access-Control-Request-Method")!);
            denied |= (request.Header("Access-Control-Request-Headers") ?? "").Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries).Any(x => !headers.Contains(x));
        }
        return new Decision(denied, preflight, origin, string.Join(", ", methods), string.Join(", ", headers));
    }
    public sealed class Decision
    {
        private readonly bool denied, preflight;
        private readonly string? origin;
        private readonly string methods, headers;
        internal Decision(bool denied, bool preflight, string? origin, string methods, string headers)
            => (this.denied, this.preflight, this.origin, this.methods, this.headers) = (denied, preflight, origin, methods, headers);
        public Prepared? Stop() => denied ? Apply(Response.Text("Origin or preflight is not allowed. Check the configured origin, method and request headers.", 403)) :
            preflight ? Apply(Response.Empty()) : null;
        public Prepared Apply(Response response)
        {
            if (denied) response = Response.Text("Origin or preflight is not allowed. Check the configured origin, method and request headers.", 403);
            else if (preflight) response = Response.Empty();
            response = response.WithHeader("Vary", "Origin, Access-Control-Request-Method, Access-Control-Request-Headers")
                .WithHeader("X-Content-Type-Options", "nosniff")
                .WithHeader("Content-Security-Policy", "default-src 'self'; script-src 'none'; style-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'")
                .WithHeader("Referrer-Policy", "same-origin");
            if (!response.Headers.ContainsKey("Cache-Control")) response = response.WithHeader("Cache-Control", "no-store");
            if (!denied && origin is not null)
            {
                response = response.WithHeader("Access-Control-Allow-Origin", origin).WithHeader("Access-Control-Allow-Credentials", "true");
                if (preflight) response = response.WithHeader("Access-Control-Allow-Methods", methods).WithHeader("Access-Control-Allow-Headers", headers);
            }
            return new Prepared(response);
        }
    }
    public sealed class Prepared
    {
        public Response Response { get; }
        internal Prepared(Response response) => Response = response;
    }
}
</web:copy>

## Assets

Serve only explicitly registered assets. There is no directory traversal mapping, MIME guessing, or user-controlled filesystem path. Exact ETags produce 304; HEAD body suppression belongs to the host. This initial asset library serves CSS text, which is sufficient for the reference app and keeps executable uploads out of the static origin.

<web:copy id="ew-assets">
using System.Security.Cryptography;
using System.Text;

namespace ExplicitWeb;

public sealed class Assets
{
    private readonly Dictionary<string, (string Css, string Tag)> styles = new(StringComparer.Ordinal);
    public void AddCss(LocalUrl path, string css)
    {
        var tag = "\"" + Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(css))) + "\"";
        styles.Add(path.Value, (css, tag));
    }
    public Response? Find(Request request)
    {
        if (!styles.TryGetValue(request.Path, out var asset)) return null;
        if (request.Method is not ("GET" or "HEAD")) return Response.Text("Use GET to retrieve this stylesheet.", 405).WithHeader("Allow", "GET, HEAD");
        var response = request.Header("If-None-Match") == asset.Tag ? Response.Empty(304) : Response.Css(asset.Css);
        return response.WithHeader("ETag", asset.Tag).WithHeader("Cache-Control", "public, max-age=300");
    }
}
</web:copy>

## Assemble these outputs

<web:file path="experiments/explicit-web/Core/Core.csproj"><web:paste select="#ew-Core-Core-csproj"/></web:file>
<web:file path="experiments/explicit-web/Core/Result.cs"><web:paste select="#ew-result"/></web:file>
<web:file path="experiments/explicit-web/Core/Request.cs"><web:paste select="#ew-request"/></web:file>
<web:file path="experiments/explicit-web/Core/Html.cs"><web:paste select="#ew-html"/></web:file>
<web:file path="experiments/explicit-web/Core/Response.cs"><web:paste select="#ew-response"/></web:file>
<web:file path="experiments/explicit-web/Core/Routes.cs"><web:paste select="#ew-routes"/></web:file>
<web:file path="experiments/explicit-web/Core/Cors.cs"><web:paste select="#ew-cors"/></web:file>
<web:file path="experiments/explicit-web/Core/Assets.cs"><web:paste select="#ew-assets"/></web:file>

</web:doc>
