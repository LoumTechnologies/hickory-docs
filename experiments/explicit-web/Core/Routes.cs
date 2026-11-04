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
