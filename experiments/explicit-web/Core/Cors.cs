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
