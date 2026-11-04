using ExplicitWeb;
using ExplicitWeb.Security;

namespace TaskApp;

public sealed partial class Application
{
    private readonly Cors cors;
    private readonly Sessions sessions;
    private readonly Accounts accounts;
    private readonly Tasks tasks;
    private readonly RateLimit loginLimit;
    private readonly Assets assets = new();
    private static readonly Route<Guid> ApiItem = Routes.Id("GET", "/api/tasks/{id}");
    public Application(Cors cors, Sessions sessions, Accounts accounts, Tasks tasks, RateLimit loginLimit)
    {
        (this.cors, this.sessions, this.accounts, this.tasks, this.loginLimit) = (cors, sessions, accounts, tasks, loginLimit);
        assets.AddCss(LocalUrl.Parse("/site.css").Match(x => x, _ => throw new InvalidOperationException()), Pages.Css);
    }
    public Cors.Prepared Handle(Request request)
    {
        var origin = cors.Inspect(request);
        if (origin.Stop() is { } stop) return stop;
        if (assets.Find(request) is { } asset) return origin.Apply(asset);
        if (request.Path == "/health") return origin.Apply(Response.Json(new { status = "ok" }));
        var session = sessions.Load(request);
        var response = Dispatch(request, session);
        if (session.NewCookie is { } cookie && !response.Headers.ContainsKey("Set-Cookie")) response = response.WithHeader("Set-Cookie", cookie);
        return origin.Apply(response);
    }
    private static bool Reading(Request request) => request.Method is "GET" or "HEAD";
    private static Response Error(Request request, Problem problem) => request.Path.StartsWith("/api/", StringComparison.Ordinal) ? Response.Error(problem) : Pages.Error(problem);
    private static Response Home() => Response.Redirect(Pages.Home.Link(new Unit()));
    private Response Dispatch(Request request, Session session)
    {
        if (request.Path == "/") return Reading(request)
            ? session.User is { } user ? Pages.List(session, user, tasks.List(user)) : Pages.Welcome()
            : Method("GET, HEAD");
        if (request.Path is "/login" or "/register")
        {
            if (Reading(request)) return Pages.Account(session, request.Path == "/register");
            if (request.Method != "POST") return Method("GET, HEAD, POST");
            return Account(request, session);
        }
        if (request.Path == "/logout")
        {
            if (request.Method != "POST") return Method("POST");
            return Verified(request, session).Match(input => sessions.Revoke(input).Match(_ => Home().WithHeader("Set-Cookie", sessions.ExpiredCookie), p => Error(request, p)), p => Error(request, p));
        }
        if (request.Path == "/api/session") return Reading(request)
            ? Response.Json(new { csrf = session.Csrf, user = session.User?.Name }) : Method("GET, HEAD");
        if (request.Path == "/api/tasks") return ApiCollection(request, session);
        if (request.Path.StartsWith("/api/tasks/", StringComparison.Ordinal)) return ApiTask(request, session);
        if (request.Path == "/tasks") return request.Method == "POST" ? FormCreate(request, session) : Method("POST");
        if (request.Path.StartsWith("/tasks/", StringComparison.Ordinal)) return FormTask(request, session);
        return Error(request, new Problem(404, "Page not found. Follow a link from the home page."));
    }
    private static Response Method(string allow) => Response.Text($"Method not allowed. Use {allow}.", 405).WithHeader("Allow", allow);
    private static Result<IReadOnlyDictionary<string, string>> Fields(Request request) => request.Form();
    private static string? Field(IReadOnlyDictionary<string, string> fields, string key) => fields.GetValueOrDefault(key);
    private static Result<VerifiedInput> Verified(Request request, Session session) => request.Path.StartsWith("/api/", StringComparison.Ordinal)
        ? session.Verify(request, request.Header("X-CSRF-Token"))
        : Fields(request).Bind(fields => session.Verify(request, Field(fields, "csrf")));
    private static Result<WritePermit> Permit(Request request, Session session) => session.RequireUser()
        .Bind(user => Verified(request, session).Bind(user.Authorize));
    private Response Account(Request request, Session session)
    {
        if (!loginLimit.Allow(request.Peer)) return Error(request, new Problem(429, "Too many sign-in attempts. Wait a minute before trying again.")).WithHeader("Retry-After", "60");
        return Fields(request).Bind(fields => session.Verify(request, Field(fields, "csrf")).Bind(input =>
            Credentials.Parse(Field(fields, "name"), Field(fields, "password")).Bind(credentials =>
                request.Path == "/register" ? accounts.Register(input, credentials) : accounts.Login(input, credentials))))
            .Match(next => Home().WithHeader("Set-Cookie", next.NewCookie!), p => Error(request, p));
    }
}
