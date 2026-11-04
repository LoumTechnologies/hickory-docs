<web:doc xmlns:web="http://www.hickorydocs.com/1.0">
# Build a complete private task application

For framework users building a small CRUD application. Start the app, register, create a task, edit it, complete it, and delete it. The same repository serves forms and JSON. Authentication is required before reading tasks; writes additionally require CSRF verification and a validated title/version. The SQL predicate always includes the authenticated owner. This application is the acceptance vehicle, not the framework itself.

## Server server csproj

Keep assembly boundaries real: internal constructors cannot be called by consumers. Nullable warnings are errors; deterministic builds and dependency locks make evaluation reproducible.

<web:copy id="ew-Server-Server-csproj">
<Project Sdk="Microsoft.NET.Sdk.Web">
  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework><Nullable>enable</Nullable><ImplicitUsings>enable</ImplicitUsings>
    <TreatWarningsAsErrors>true</TreatWarningsAsErrors><UseAppHost>false</UseAppHost>
    <RestorePackagesWithLockFile>true</RestorePackagesWithLockFile><Deterministic>true</Deterministic>
  </PropertyGroup>
  <ItemGroup><ProjectReference Include="../Core/Core.csproj" /><ProjectReference Include="../Data/Data.csproj" /><ProjectReference Include="../Security/Security.csproj" /><ProjectReference Include="../Hosting/Hosting.csproj" /></ItemGroup>
</Project>
</web:copy>

## Task values

Validate a task title before the repository can accept it. Optimistic versions prevent a stale form from silently overwriting a newer edit. An absent or foreign-owned ID returns the same 404, keeping another user’s data out of the response.

<web:copy id="ew-task-values">
using ExplicitWeb;
using ExplicitWeb.Data;
using ExplicitWeb.Security;

namespace TaskApp;

public sealed class Title
{
    public string Value { get; }
    private Title(string value) => Value = value;
    public static Result<Title> Parse(string? value)
    {
        value = value?.Trim();
        return string.IsNullOrEmpty(value) || value.Length > 200 || value.Any(char.IsControl)
            ? Result<Title>.Fail(422, "Enter a task title of 1–200 characters without control characters.")
            : Result<Title>.Ok(new(value));
    }
}
public sealed record TaskItem(Guid Id, string Title, bool Done, long Version);
public sealed record TaskInput(string? Title, bool Done = false, long Version = 0);
public sealed record LoginInput(string? Name, string? Password);
public sealed class Tasks(Database database)
{
    private static TaskItem Read(ExplicitWeb.Data.Row r) => new(Guid.Parse(r.GetString(0)), r.GetString(1), r.GetBoolean(2), r.GetInt64(3));
    public List<TaskItem> List(Principal user) => database.Transaction(sql => sql.Query($"SELECT id,title,done,version FROM tasks WHERE owner={user.UserId} ORDER BY rowid DESC", Read));
    public Result<TaskItem> Find(Principal user, Guid id) => database.Transaction(sql => sql.Query($"SELECT id,title,done,version FROM tasks WHERE owner={user.UserId} AND id={id.ToString()}", Read))
        .SingleOrDefault() is { } task ? Result<TaskItem>.Ok(task) : Result<TaskItem>.Fail(404, "Task not found. Return to your task list.");
    public TaskItem Create(WritePermit permit, Title title) => database.Transaction(sql =>
    {
        var task = new TaskItem(Guid.NewGuid(), title.Value, false, 1);
        sql.Execute($"INSERT INTO tasks(id,owner,title,done,version) VALUES({task.Id.ToString()},{permit.UserId},{task.Title},{false},{1})");
        return task;
    });
    public Result<TaskItem> Update(WritePermit permit, Guid id, Title title, bool done, long version) => database.Transaction(sql =>
    {
        var count = sql.Execute($"UPDATE tasks SET title={title.Value},done={done},version=version+1 WHERE id={id.ToString()} AND owner={permit.UserId} AND version={version}");
        if (count == 1) return Result<TaskItem>.Ok(new(id, title.Value, done, version + 1));
        return Failure<TaskItem>(sql, permit, id);
    });
    public Result<bool> Delete(WritePermit permit, Guid id, long version) => database.Transaction(sql =>
    {
        var count = sql.Execute($"DELETE FROM tasks WHERE id={id.ToString()} AND owner={permit.UserId} AND version={version}");
        return count == 1 ? Result<bool>.Ok(true) : Failure<bool>(sql, permit, id);
    });
    private static Result<T> Failure<T>(Sql sql, WritePermit permit, Guid id) where T : notnull =>
        sql.Scalar($"SELECT COUNT(*) FROM tasks WHERE id={id.ToString()} AND owner={permit.UserId}") == 0
            ? Result<T>.Fail(404, "Task not found. Return to your task list.")
            : Result<T>.Fail(409, "This task changed since you loaded it. Reload before editing again.");
}
</web:copy>

## App schema

Use one versioned migration sequence for accounts, sessions and tasks. Foreign keys link owners and sessions to accounts. The server constructs the schema before it starts listening. Reopening the same database is idempotent; a new version must append a migration rather than rewrite a deployed one.

<web:copy id="ew-app-schema">
using ExplicitWeb.Data;
using ExplicitWeb.Security;

namespace TaskApp;

public static class Schema
{
    public static void Initialize(Database database) => database.Migrate(IdentitySchema.Migration, """
        CREATE TABLE tasks(id TEXT PRIMARY KEY, owner TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE, title TEXT NOT NULL, done INTEGER NOT NULL, version INTEGER NOT NULL);
        CREATE INDEX tasks_owner ON tasks(owner);
        """);
}
</web:copy>

## Pages layout

Use the checked HTML template throughout. All dynamic titles, usernames, task text, IDs and CSRF values enter interpolation slots. The CSS is a registered static asset; no inline scripts or inline event handlers are necessary. This works with the default CSP and with JavaScript disabled.

<web:copy id="ew-pages-layout">
using ExplicitWeb;
using ExplicitWeb.Security;

namespace TaskApp;

public static class Pages
{
    public static readonly Route<Guid> Item = Routes.Id("GET", "/tasks/{id}");
    public static readonly Route<Unit> Home = Routes.At("GET", "/");
    public static Response Layout(string title, Html body) => Response.Page(Html.Template($"""
        <!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
        <title>{title} · Explicit Tasks</title><link rel="stylesheet" href="/site.css"></head>
        <body><main><header><a href="/">Explicit Tasks</a><span>Ordinary functions. Checked boundaries.</span></header>{body}</main></body></html>
        """));
    public static Response Account(Session session, bool register) => Layout(register ? "Create account" : "Sign in", Html.Template($"""
        <h1>{(register ? "Create your account" : "Welcome back")}</h1>
        <form method="post" action="{(register ? "/register" : "/login")}">
        <input type="hidden" name="csrf" value="{session.Csrf}">
        <label>Username <input name="name" required minlength="3" maxlength="40" autocomplete="username"></label>
        <label>Password <input name="password" type="password" required minlength="12" maxlength="128" autocomplete="{(register ? "new-password" : "current-password")}"></label>
        <button>{(register ? "Create account" : "Sign in")}</button></form><p><a href="{(register ? "/login" : "/register")}">{(register ? "Already registered? Sign in" : "Need an account? Register")}</a></p>
        """));
    public static Response Welcome() => Layout("Welcome", Html.Template($"""
        <h1>A small place to get things done.</h1><p>Create a private task list. Your work stays with your account.</p>
        <p><a class="button" href="/register">Create account</a> <a href="/login">Sign in</a></p>
        """));
</web:copy>

## Task pages

Render each task through checked interpolation and link it through the GUID route. Edit forms carry the CSRF token and optimistic version; the repository still verifies both relevant boundaries on submission.

<web:copy id="ew-task-pages">
    public static Response List(Session session, Principal user, IEnumerable<TaskItem> tasks) => Layout("Your tasks", Html.Template($"""
        <div class="heading"><h1>{user.Name}’s tasks</h1><form method="post" action="/logout"><input type="hidden" name="csrf" value="{session.Csrf}"><button class="secondary">Sign out</button></form></div>
        <form method="post" action="/tasks" class="new-task"><input type="hidden" name="csrf" value="{session.Csrf}"><label>New task <input name="title" required maxlength="200" placeholder="What needs doing?"></label><button>Add task</button></form>
        <ul class="tasks">{Html.Join(tasks.Select(t => Html.Template($"""
            <li><a href="{Item.Link(t.Id).Value}">{t.Title}</a><span>{(t.Done ? "Complete" : "Open")}</span></li>
            """)))}</ul><p class="hint">Select a task to edit, complete or delete it.</p>
        """));
    public static Response Edit(Session session, TaskItem task) => Layout("Edit task", Html.Template($"""
        <p><a href="/">← All tasks</a></p><h1>Edit task</h1><form method="post" action="{Item.Link(task.Id).Value}">
        <input type="hidden" name="csrf" value="{session.Csrf}"><input type="hidden" name="version" value="{task.Version}">
        <label>Title <input name="title" value="{task.Title}" required maxlength="200"></label>
        <label>Status <select name="done"><option value="{(task.Done ? "true" : "false")}">{(task.Done ? "Complete" : "Open")}</option><option value="{(task.Done ? "false" : "true")}">{(task.Done ? "Open" : "Complete")}</option></select></label>
        <div class="actions"><button name="action" value="save">Save changes</button><button class="danger" name="action" value="delete">Delete task</button></div></form>
        """));
</web:copy>

## Error pages

Preserve a failure status while presenting a useful recovery link. The stylesheet is trusted application source registered by path, not untrusted uploaded content.

<web:copy id="ew-error-pages">
    public static Response Error(Problem problem) => Layout("Please try again", Html.Template($"""
        <h1>We couldn’t complete that request</h1><p>{problem.Message}</p><p><a href="/">Return home</a> or use your browser’s Back button to correct the form.</p>
        """)).WithStatus(problem.Status);
    public const string Css = """
        :root{font-family:system-ui,sans-serif;color:#18332c;background:#f4f5ee}*{box-sizing:border-box}body{margin:0}main{max-width:760px;margin:auto;padding:32px 24px}header{display:flex;justify-content:space-between;gap:20px;padding-bottom:28px;border-bottom:1px solid #cdd7cb;margin-bottom:48px}header a{font-weight:750;text-decoration:none}header span,.hint{font-size:.85rem;color:#567063}h1{font-size:2.3rem;letter-spacing:-.04em}a{color:#245c46}form{display:grid;gap:20px}label{display:grid;gap:8px}input,select{font:inherit;padding:12px;border:1px solid #b8c9ba;border-radius:6px;background:#fff}button,.button{font:inherit;border:0;border-radius:6px;background:#245c46;color:white;padding:12px 20px;cursor:pointer;text-decoration:none;display:inline-block}button.secondary{background:#e2e9df;color:#245c46}.danger{background:#873d35}.heading,.actions{display:flex;align-items:center;justify-content:space-between;gap:12px}.tasks{list-style:none;padding:0;margin:32px 0}.tasks li{padding:18px 0;border-bottom:1px solid #cdd7cb;display:flex;justify-content:space-between;gap:20px}.tasks span{font-size:.85rem;color:#567063}.new-task{background:#e8eee1;padding:24px;border-radius:8px}@media(max-width:540px){header span{display:none}.heading{display:block}h1{font-size:2rem}}
        """;
}
</web:copy>

## App request path

This is the complete application-level request path: CORS decision, static asset lookup, session resolution, route dispatch, cookie propagation, CORS response preparation. Follow any endpoint below to see its identity, validation and mutation checks in execution order. The host never wraps this function in a configurable stack.

<web:copy id="ew-app-request-path">
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
</web:copy>

## Api endpoints

Both reads and writes resolve a principal. Collection creation requires Title; updates and deletes require a current version. Invalid JSON, media type, title and route ID stay explicit error results. JSON returns semantic HTTP statuses and a Location for creation; the API uses the same cookie and CSRF mechanism as forms.

<web:copy id="ew-api-endpoints">
using ExplicitWeb;
using ExplicitWeb.Security;

namespace TaskApp;

public sealed partial class Application
{
    private Response ApiCollection(Request request, Session session)
    {
        if (Reading(request)) return session.RequireUser().Match(user => Response.Json(tasks.List(user)), p => Error(request, p));
        if (request.Method != "POST") return Method("GET, HEAD, POST");
        return Permit(request, session).Bind(permit => request.Json<TaskInput>().Bind(input => Title.Parse(input.Title)
            .Match(title => Result<TaskItem>.Ok(tasks.Create(permit, title)), p => Result<TaskItem>.Fail(p.Status, p.Message))))
            .Match(task => Response.Json(task, 201).WithHeader("Location", ApiItem.Link(task.Id).Value), p => Error(request, p));
    }
    private Response ApiTask(Request request, Session session)
    {
        var route = Routes.Id(request.Method, "/api/tasks/{id}");
        if (Reading(request)) return session.RequireUser().Bind(user => route.Match(request).Bind(match => tasks.Find(user, match.Value)))
            .Match(task => Response.Json(task), p => Error(request, p));
        if (request.Method is not ("PUT" or "DELETE")) return Method("GET, HEAD, PUT, DELETE");
        return Permit(request, session).Match(permit => route.Match(request).Match(match =>
            request.Json<TaskInput>().Match(input =>
            {
                if (input.Version < 1) return Error(request, new Problem(422, "Supply the current positive task version."));
                if (request.Method == "DELETE") return tasks.Delete(permit, match.Value, input.Version).Match(_ => Response.Empty(), p => Error(request, p));
                return Title.Parse(input.Title).Bind(title => tasks.Update(permit, match.Value, title, input.Done, input.Version))
                    .Match(task => Response.Json(task), p => Error(request, p));
            }, p => Error(request, p)), p => Error(request, p)), p => Error(request, p));
    }
}
</web:copy>

## Form endpoints

Forms share the protected repository rather than reimplementing persistence. Successful writes use POST/redirect/GET. Failed writes preserve their error status and explain recovery. Editing carries the row version as a hidden field; the database enforces it atomically.

<web:copy id="ew-form-endpoints">
using ExplicitWeb;
using ExplicitWeb.Security;

namespace TaskApp;

public sealed partial class Application
{
    private Response FormCreate(Request request, Session session) => Permit(request, session).Match(permit =>
        Fields(request).Bind(fields => Title.Parse(Field(fields, "title"))).Match(title =>
        { tasks.Create(permit, title); return Home(); }, p => Error(request, p)), p => Error(request, p));
    private Response FormTask(Request request, Session session)
    {
        var route = Routes.Id(request.Method, "/tasks/{id}");
        if (Reading(request)) return session.RequireUser().Bind(user => route.Match(request).Bind(match => tasks.Find(user, match.Value)))
            .Match(task => Pages.Edit(session, task), p => Error(request, p));
        if (request.Method != "POST") return Method("GET, HEAD, POST");
        return Permit(request, session).Match(permit => route.Match(request).Match(match => Fields(request).Match(fields =>
        {
            if (!long.TryParse(Field(fields, "version"), out var version) || version < 1)
                return Error(request, new Problem(422, "Reload the task to get its current version."));
            if (Field(fields, "action") == "delete") return tasks.Delete(permit, match.Value, version).Match(_ => Home(), p => Error(request, p));
            if (Field(fields, "action") != "save") return Error(request, new Problem(422, "Choose Save changes or Delete task."));
            return Title.Parse(Field(fields, "title")).Bind(title => tasks.Update(permit, match.Value, title, Field(fields, "done") == "true", version))
                .Match(_ => Home(), p => Error(request, p));
        }, p => Error(request, p)), p => Error(request, p)), p => Error(request, p));
    }
}
</web:copy>

## Settings

Read the complete environment schema once. Refuse insecure cookies on a non-loopback listener. Production TLS is terminated by a reverse proxy; forwarded identity headers are not trusted. Data and ready-file paths are explicit configuration. No environment branches occur in request handling.

<web:copy id="ew-settings">
using System.Net;
using ExplicitWeb.Hosting;

namespace TaskApp;

public sealed record Settings(ServerSettings Server, Uri Origin, string Database, string ReadyFile, bool SecureCookie, TimeSpan SessionLifetime, int LoginAttempts)
{
    public static Settings Load()
    {
        string Read(string name) => Environment.GetEnvironmentVariable(name) ?? throw new ArgumentException($"Set {name} before starting. See docs/developers/quickstart.md.");
        int Number(string name, int min, int max)
        {
            if (!int.TryParse(Read(name), out var n) || n < min || n > max) throw new ArgumentException($"Set {name} to an integer from {min} to {max}.");
            return n;
        }
        string FilePath(string name)
        {
            var value = Read(name); if (!Path.IsPathFullyQualified(value)) throw new ArgumentException($"Set {name} to an absolute file path."); return value;
        }
        if (!Uri.TryCreate(Read("EXPLICIT_LISTEN_URL"), UriKind.Absolute, out var listen) || listen.Scheme != "http" ||
            !IPAddress.TryParse(listen.Host, out var address) || listen.AbsolutePath != "/" || listen.Query != "" || listen.Fragment != "" || listen.UserInfo != "")
            throw new ArgumentException("Set EXPLICIT_LISTEN_URL to an HTTP URL with an IP address and port, without a path or credentials.");
        if (!Uri.TryCreate(Read("EXPLICIT_ALLOWED_ORIGIN"), UriKind.Absolute, out var origin)) throw new ArgumentException("Set EXPLICIT_ALLOWED_ORIGIN to the browser HTTP(S) origin.");
        if (!bool.TryParse(Read("EXPLICIT_SECURE_COOKIE"), out var secure)) throw new ArgumentException("Set EXPLICIT_SECURE_COOKIE to true or false.");
        if (!secure && !IPAddress.IsLoopback(address)) throw new ArgumentException("Enable EXPLICIT_SECURE_COOKIE for non-loopback deployments and terminate TLS at a proxy.");
        return new(new(address, listen.Port, Number("EXPLICIT_MAX_BODY_BYTES", 1, 16_777_216), TimeSpan.FromSeconds(Number("EXPLICIT_REQUEST_TIMEOUT_SECONDS", 1, 300))),
            origin, FilePath("EXPLICIT_DATABASE"), FilePath("EXPLICIT_READY_FILE"), secure,
            TimeSpan.FromSeconds(Number("EXPLICIT_SESSION_SECONDS", 1, 2_592_000)), Number("EXPLICIT_LOGIN_ATTEMPTS", 1, 1000));
    }
}
</web:copy>

## Startup

Compose concrete libraries at startup, initialize the schema, then connect the app function directly to the transport. A ready file is a test/lifecycle interface, not a guessed port. SIGINT and SIGTERM cancel the server; finally removes the ready file. Credentials and sessions never enter request logs.

<web:copy id="ew-startup">
using TaskApp;
await Bootstrap.Run(false);
</web:copy>

## Compose host

The comparison arm calls this same composition with the conventional host selected. Functional behavior and dependency versions stay identical. This switch exists only at the executable boundary, not inside request handling.

<web:copy id="ew-compose-host">
using System.Runtime.InteropServices;
using ExplicitWeb;
using ExplicitWeb.Data;
using ExplicitWeb.Hosting;
using ExplicitWeb.Security;
namespace TaskApp;

public static class Bootstrap
{
public static async Task Run(bool pipeline)
{
var settings = Settings.Load();
var database = new Database(settings.Database);
Schema.Initialize(database);
var clock = TimeProvider.System;
var sessions = new Sessions(database, clock, settings.SessionLifetime, settings.SecureCookie);
var accounts = new Accounts(database, sessions);
var cors = new Cors([settings.Origin], ["GET", "POST", "PUT", "DELETE"], ["content-type", "x-csrf-token"]);
var application = new TaskApp.Application(cors, sessions, accounts, new Tasks(database), new RateLimit(clock, settings.LoginAttempts, TimeSpan.FromMinutes(1)));
using var stop = new CancellationTokenSource();
Console.CancelKeyPress += (_, e) => { e.Cancel = true; stop.Cancel(); };
using var terminate = OperatingSystem.IsWindows() ? null : PosixSignalRegistration.Create(PosixSignal.SIGTERM, context => { context.Cancel = true; stop.Cancel(); });
try
{
    Func<ExplicitWeb.Request, Task<Cors.Prepared>> handle = request => Task.FromResult(application.Handle(request));
    Action<Uri> ready = uri => File.WriteAllText(settings.ReadyFile, uri.GetLeftPart(UriPartial.Authority));
    if (pipeline) await PipelineServer.Run(settings.Server, handle, ready, stop.Token);
    else await Server.Run(settings.Server, handle, ready, stop.Token);
}
finally { File.Delete(settings.ReadyFile); }
}
}
</web:copy>

## Assemble these outputs

<web:file path="experiments/explicit-web/Server/Server.csproj"><web:paste select="#ew-Server-Server-csproj"/></web:file>
<web:file path="experiments/explicit-web/Server/Tasks.cs"><web:paste select="#ew-task-values"/></web:file>
<web:file path="experiments/explicit-web/Server/Schema.cs"><web:paste select="#ew-app-schema"/></web:file>
<web:file path="experiments/explicit-web/Server/Pages.cs"><web:paste select="#ew-pages-layout"/><web:paste select="#ew-task-pages"/><web:paste select="#ew-error-pages"/></web:file>
<web:file path="experiments/explicit-web/Server/Application.cs"><web:paste select="#ew-app-request-path"/></web:file>
<web:file path="experiments/explicit-web/Server/Api.cs"><web:paste select="#ew-api-endpoints"/></web:file>
<web:file path="experiments/explicit-web/Server/Forms.cs"><web:paste select="#ew-form-endpoints"/></web:file>
<web:file path="experiments/explicit-web/Server/Settings.cs"><web:paste select="#ew-settings"/></web:file>
<web:file path="experiments/explicit-web/Server/Program.cs"><web:paste select="#ew-startup"/></web:file>
<web:file path="experiments/explicit-web/Server/Bootstrap.cs"><web:paste select="#ew-compose-host"/></web:file>

</web:doc>
