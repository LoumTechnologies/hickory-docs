using ExplicitWeb;
using ExplicitWeb.Data;
using ExplicitWeb.Security;
using TaskApp;

if (args.Contains("--serve-errors"))
{
    var settings = TaskApp.Settings.Load();
    using var stop = new CancellationTokenSource();
    using var signal = OperatingSystem.IsWindows() ? null : System.Runtime.InteropServices.PosixSignalRegistration.Create(System.Runtime.InteropServices.PosixSignal.SIGTERM, c => { c.Cancel = true; stop.Cancel(); });
    await ExplicitWeb.Hosting.Server.Run(settings.Server, async request =>
    {
        if (request.Path == "/throw") throw new InvalidOperationException("PRIVATE_EXCEPTION_MARKER");
        if (request.Path == "/timeout") await Task.Delay(Timeout.Infinite, request.Cancel);
        return new Cors([], [], []).Inspect(request).Apply(Response.Text("ok"));
    }, uri => File.WriteAllText(settings.ReadyFile, uri.GetLeftPart(UriPartial.Authority)), stop.Token);
    return;
}
void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
T Value<T>(Result<T> result) where T : notnull => result.Match(x => x, p => throw new Exception(p.Message));
var directory = Path.Combine(Path.GetTempPath(), "explicit-check-" + Guid.NewGuid()); Directory.CreateDirectory(directory);
try
{
    var database = new Database(Path.Combine(directory, "checks.sqlite")); Schema.Initialize(database); Schema.Initialize(database);
    database.Transaction(sql => { sql.Execute($"CREATE TABLE probes(text TEXT, bytes BLOB)"); return true; });
    var text = "Unicode ☃ ' ; DROP TABLE users; --" + (char)0 + "tail"; var bytes = new byte[] { 0, 127, 255 };
    database.Transaction(sql => { sql.Execute($"INSERT INTO probes VALUES({text},{bytes})"); return true; });
    Check(database.Transaction(sql => sql.Query($"SELECT {1.25}", r => r.GetDouble(0)).Single()) == 1.25, "Double round trip failed");
    Check(database.Transaction(sql => sql.Query($"SELECT {Array.Empty<byte>()}", r => !r.IsDBNull(0) && ((byte[])r[0]).Length == 0).Single()), "Empty BLOB became NULL");
    var row = database.Transaction(sql => sql.Query($"SELECT text,bytes FROM probes", r => r).Single());
    Check(row.GetString(0) == text && ((byte[])row[1]).SequenceEqual(bytes), "Parameter or row lifetime failure");
    try { database.Transaction<bool>(sql => { sql.Execute($"DELETE FROM probes"); throw new InvalidOperationException("rollback"); }); }
    catch (InvalidOperationException) { }
    Check(database.Transaction(sql => sql.Scalar($"SELECT COUNT(*) FROM probes")) == 1, "Transaction did not roll back");
    var sessions = new Sessions(database, TimeProvider.System, TimeSpan.FromHours(1), true);
    Request Guest() => new("GET", "/", "local", new Dictionary<string,string>(), []);
    var a = sessions.Load(Guest()); var b = sessions.Load(Guest());
    var write = new Request("POST", "/tasks", "local", new Dictionary<string,string>(), []);
    Check(a.Verify(write, b.Csrf).Match(_ => false, p => p.Status == 403), "Cross-session CSRF accepted");
    Check(a.Verify(Guest(), a.Csrf).Match(_ => false, p => p.Status == 405), "Safe request got mutation proof");
    var accounts = new Accounts(database, sessions);
    var signed = Value(accounts.Register(Value(a.Verify(write, a.Csrf)), Value(Credentials.Parse("runtime", "correct horse battery"))));
    Check(signed.NewCookie!.Contains("; Secure"), "Secure cookie flag missing");
    var principal = Value(signed.RequireUser());
    Check(principal.Authorize(Value(b.Verify(write, b.Csrf))).Match(_ => false, p => p.Status == 403), "Cross-session proof joined");
    var route = Routes.Id("GET", "/tasks/{id}"); var id = Guid.NewGuid();
    Check(Value(route.Match(new("GET", route.Link(id).Value, "local", new Dictionary<string,string>(), []))).Value == id, "Typed route round trip failed");
    foreach (var path in new[]{"//evil.example", "/%2fevil", "/bad\\path", "https://evil.example"})
        Check(LocalUrl.Parse(path).Match(_ => false, p => p.Status == 400), "Unsafe redirect accepted");
    Check(Html.Template($"<p>{"<script>"}</p>").Encoded == "<p>&lt;script&gt;</p>", "HTML interpolation was not encoded");
    Console.WriteLine("PASS: transaction rollback, parameter binding, row lifetime, sessions, capability binding, routes, redirects and HTML encoding");
}
finally { Directory.Delete(directory, true); }
