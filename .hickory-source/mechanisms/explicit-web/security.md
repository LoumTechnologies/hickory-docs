<web:doc xmlns:web="http://www.hickorydocs.com/1.0">
# Carry identity and mutation authority as values

For authors adding protected behavior. A session is loaded explicitly in the request function. Authenticating produces Principal; verifying a submitted token produces VerifiedInput; joining those from the same session produces WritePermit. The repository cannot accept a naked user ID as authority. These types prevent accidental omission in the supported API; application authors can still bypass the repository or misuse a retained capability. C# has no affine ownership.

## Security security csproj

Keep assembly boundaries real: internal constructors cannot be called by consumers. Nullable warnings are errors; deterministic builds and dependency locks make evaluation reproducible.

<web:copy id="ew-Security-Security-csproj">
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup><PackageId>ExplicitWeb.Security</PackageId>
    <TargetFramework>net10.0</TargetFramework><Nullable>enable</Nullable><ImplicitUsings>enable</ImplicitUsings>
    <TreatWarningsAsErrors>true</TreatWarningsAsErrors><UseAppHost>false</UseAppHost>
    <RestorePackagesWithLockFile>true</RestorePackagesWithLockFile><Deterministic>true</Deterministic>
  </PropertyGroup>
  <ItemGroup><ProjectReference Include="../Core/Core.csproj" /><ProjectReference Include="../Data/Data.csproj" /></ItemGroup>
</Project>
</web:copy>

## Identity values

Separate raw credentials, a recognized principal, and a permitted mutation. Restrict usernames to a normalized portable alphabet; passwords have a bounded length. CSRF input is tied to both the session and a non-safe request. Account transitions require the same verified input type as other state changes.

<web:copy id="ew-identity-values">
namespace ExplicitWeb.Security;

public sealed class Credentials
{
    public string Name { get; }
    internal string Password { get; }
    private Credentials(string name, string password) => (Name, Password) = (name, password);
    public static Result<Credentials> Parse(string? name, string? password)
    {
        name = name?.Trim().ToLowerInvariant();
        if (name is null || name.Length < 3 || name.Length > 40 || name.Any(c => !char.IsAsciiLetterOrDigit(c) && c != '_'))
            return Result<Credentials>.Fail(422, "Use a username of 3–40 letters, digits or underscores.");
        if (password is null || password.Length < 12 || password.Length > 128)
            return Result<Credentials>.Fail(422, "Use a password of 12–128 characters.");
        return Result<Credentials>.Ok(new(name, password));
    }
}
public sealed class Principal
{
    public string UserId { get; }
    public string Name { get; }
    private readonly string sessionId;
    internal Principal(string userId, string name, string sessionId) => (UserId, Name, this.sessionId) = (userId, name, sessionId);
    public Result<WritePermit> Authorize(VerifiedInput input) => input.SessionId == sessionId
        ? Result<WritePermit>.Ok(new(UserId, input.RequestId))
        : Result<WritePermit>.Fail(403, "The form belongs to another session. Reload the page and try again.");
}
public sealed class VerifiedInput
{
    internal string SessionId { get; }
    internal Guid RequestId { get; }
    internal VerifiedInput(string sessionId, Guid requestId) => (SessionId, RequestId) = (sessionId, requestId);
}
public sealed class WritePermit
{
    public string UserId { get; }
    public Guid RequestId { get; }
    internal WritePermit(string userId, Guid requestId) => (UserId, RequestId) = (userId, requestId);
}
</web:copy>

## Session checks

Store only a hash of the random session cookie. A session expires on the server, survives a restart, and rotates when an account is entered. Token comparison uses fixed time after bounding the supplied length. Revoke invalidates the old token; no signed cookie can resurrect it. Session lookup and CSRF checks are deliberately visible calls.

<web:copy id="ew-session-checks">
using System.Security.Cryptography;
using System.Text;
using ExplicitWeb.Data;

namespace ExplicitWeb.Security;

public sealed class Session
{
    internal string Hash { get; }
    public string Csrf { get; }
    public Principal? User { get; }
    public string? NewCookie { get; }
    internal Session(string hash, string csrf, Principal? user, string? newCookie)
        => (Hash, Csrf, User, NewCookie) = (hash, csrf, user, newCookie);
    public Result<Principal> RequireUser() => User is { } user ? Result<Principal>.Ok(user) : Result<Principal>.Fail(401, "Sign in to continue.");
    public Result<VerifiedInput> Verify(Request request, string? token)
    {
        if (request.Method is "GET" or "HEAD" or "OPTIONS")
            return Result<VerifiedInput>.Fail(405, "Use a state-changing method to submit this form.");
        if (token is null || token.Length != Csrf.Length || !CryptographicOperations.FixedTimeEquals(Encoding.UTF8.GetBytes(token), Encoding.UTF8.GetBytes(Csrf)))
            return Result<VerifiedInput>.Fail(403, "The security token is missing or expired. Reload the page and submit again.");
        return Result<VerifiedInput>.Ok(new(Hash, request.Id));
    }
}
public sealed class Sessions
{
    private readonly Database database;
    private readonly TimeProvider clock;
    private readonly TimeSpan lifetime;
    private readonly bool secure;
    public const string CookieName = "explicit_session";
    public Sessions(Database database, TimeProvider clock, TimeSpan lifetime, bool secure)
    {
        if (lifetime <= TimeSpan.Zero || lifetime > TimeSpan.FromDays(30)) throw new ArgumentException("Use a session lifetime between zero and 30 days.");
        (this.database, this.clock, this.lifetime, this.secure) = (database, clock, lifetime, secure);
    }
    private static string Random() => Convert.ToHexString(RandomNumberGenerator.GetBytes(32));
    private static string Hash(string token) => Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(token)));
    private string Cookie(string token) => $"{CookieName}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={(int)lifetime.TotalSeconds}" + (secure ? "; Secure" : "");
    public Session Load(Request request)
    {
        var token = request.Cookie(CookieName);
        if (token is not null && token.Length == 64)
        {
            var hash = Hash(token); var now = clock.GetUtcNow().ToUnixTimeSeconds();
            var found = database.Transaction(sql => sql.Query($"SELECT s.csrf,u.id,u.name FROM sessions s LEFT JOIN users u ON u.id=s.user_id WHERE s.hash={hash} AND s.expires>{now}",
                row => new Session(hash, row.GetString(0), row.IsDBNull(1) ? null : new Principal(row.GetString(1), row.GetString(2), hash), null))).SingleOrDefault();
            if (found is not null) return found;
        }
        return Create(null, null);
    }
    internal Session Create(Sql? transaction, string? userId)
    {
        var token = Random(); var hash = Hash(token); var csrf = Random();
        var now = clock.GetUtcNow().ToUnixTimeSeconds(); var expires = now + (long)lifetime.TotalSeconds;
        Session Insert(Sql sql)
        {
            sql.Execute($"DELETE FROM sessions WHERE expires<={now}");
            sql.Execute($"INSERT INTO sessions(hash,user_id,csrf,expires) VALUES({hash},{userId},{csrf},{expires})");
            var name = userId is null ? null : sql.Query($"SELECT name FROM users WHERE id={userId}", r => r.GetString(0)).Single();
            return new Session(hash, csrf, userId is null ? null : new Principal(userId, name!, hash), Cookie(token));
        }
        return transaction is null ? database.Transaction(Insert) : Insert(transaction);
    }
    public Result<bool> Revoke(VerifiedInput input) => database.Transaction(sql =>
    {
        sql.Execute($"DELETE FROM sessions WHERE hash={input.SessionId}"); return Result<bool>.Ok(true);
    });
    public string ExpiredCookie => $"{CookieName}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0" + (secure ? "; Secure" : "");
}
</web:copy>

## Account transitions

Create the account and rotate its session in the same transaction. Use a random salt and PBKDF2-SHA256 with 600,000 iterations; compare hashes in fixed time. Unknown usernames still perform the password derivation. Login returns one generic failure message. This is password/session authentication, not an account recovery or MFA product.

<web:copy id="ew-account-transitions">
using System.Security.Cryptography;
using ExplicitWeb.Data;

namespace ExplicitWeb.Security;

public sealed class Accounts
{
    private readonly Database database;
    private readonly Sessions sessions;
    private const int Iterations = 600_000;
    public Accounts(Database database, Sessions sessions) => (this.database, this.sessions) = (database, sessions);
    private static byte[] Derive(string password, byte[] salt) => Rfc2898DeriveBytes.Pbkdf2(password, salt, Iterations, HashAlgorithmName.SHA256, 32);
    public Result<Session> Register(VerifiedInput input, Credentials credentials)
    {
        var salt = RandomNumberGenerator.GetBytes(16); var password = Derive(credentials.Password, salt);
        try
        {
            return database.Transaction(sql =>
            {
                if (sql.Scalar($"SELECT COUNT(*) FROM sessions WHERE hash={input.SessionId}") == 0)
                    return Result<Session>.Fail(403, "Session expired. Reload and try again.");
                var id = Guid.NewGuid().ToString();
                sql.Execute($"INSERT INTO users(id,name,salt,password) VALUES({id},{credentials.Name},{salt},{password})");
                sql.Execute($"DELETE FROM sessions WHERE hash={input.SessionId}");
                return Result<Session>.Ok(sessions.Create(sql, id));
            });
        }
        catch (DatabaseException e) when (e.Code == 19)
        { return Result<Session>.Fail(409, "That username is already registered. Choose another or sign in."); }
    }
    public Result<Session> Login(VerifiedInput input, Credentials credentials)
    {
        var found = database.Transaction(sql => sql.Query($"SELECT id,salt,password FROM users WHERE name={credentials.Name}", r =>
            (Id: r.GetString(0), Salt: (byte[])r[1], Password: (byte[])r[2]))).SingleOrDefault();
        var derived = Derive(credentials.Password, found.Salt ?? new byte[16]);
        if (found.Password is null || !CryptographicOperations.FixedTimeEquals(derived, found.Password))
            return Result<Session>.Fail(401, "Username or password is incorrect. Try again.");
        return database.Transaction(sql =>
        {
            if (sql.Scalar($"SELECT COUNT(*) FROM sessions WHERE hash={input.SessionId}") == 0)
                return Result<Session>.Fail(403, "Session expired. Reload and try again.");
            sql.Execute($"DELETE FROM sessions WHERE hash={input.SessionId}");
            return Result<Session>.Ok(sessions.Create(sql, found.Id));
        });
    }
}
</web:copy>

## Rate limit

Make login throttling a visible application call. Bound the dictionary so arbitrary peer addresses cannot grow memory indefinitely. This is per-process rate limiting; deployments needing distributed limits must replace this mechanism explicitly.

<web:copy id="ew-rate-limit">
namespace ExplicitWeb.Security;

public sealed class RateLimit
{
    private readonly object gate = new();
    private readonly Dictionary<string, (DateTimeOffset Start, int Count)> windows = new();
    private readonly TimeProvider clock;
    private readonly int limit;
    private readonly TimeSpan window;
    public RateLimit(TimeProvider clock, int limit, TimeSpan window)
    {
        if (limit < 1 || window <= TimeSpan.Zero) throw new ArgumentException("Use a positive rate limit and window.");
        (this.clock, this.limit, this.window) = (clock, limit, window);
    }
    public bool Allow(string key)
    {
        lock (gate)
        {
            var now = clock.GetUtcNow();
            foreach (var expired in windows.Where(x => now - x.Value.Start >= window).Select(x => x.Key).ToArray()) windows.Remove(expired);
            if (!windows.TryGetValue(key, out var current))
            {
                if (windows.Count >= 10_000) return false;
                current = (now, 0);
            }
            windows[key] = (current.Start, current.Count + 1);
            return current.Count < limit;
        }
    }
}
</web:copy>

## Identity schema

Applications include this migration before their own tables. The account library owns its users/session layout so a second app can use it without copying the task application’s schema.

<web:copy id="ew-identity-schema">
namespace ExplicitWeb.Security;

public static class IdentitySchema
{
    public const string Migration = """
        CREATE TABLE users(id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE, salt BLOB NOT NULL, password BLOB NOT NULL);
        CREATE TABLE sessions(hash TEXT PRIMARY KEY, user_id TEXT REFERENCES users(id) ON DELETE CASCADE, csrf TEXT NOT NULL, expires INTEGER NOT NULL);
        CREATE INDEX sessions_expiry ON sessions(expires);
        """;
}
</web:copy>

## Assemble these outputs

<web:file path="experiments/explicit-web/Security/Security.csproj"><web:paste select="#ew-Security-Security-csproj"/></web:file>
<web:file path="experiments/explicit-web/Security/Identity.cs"><web:paste select="#ew-identity-values"/></web:file>
<web:file path="experiments/explicit-web/Security/Sessions.cs"><web:paste select="#ew-session-checks"/></web:file>
<web:file path="experiments/explicit-web/Security/Accounts.cs"><web:paste select="#ew-account-transitions"/></web:file>
<web:file path="experiments/explicit-web/Security/RateLimit.cs"><web:paste select="#ew-rate-limit"/></web:file>
<web:file path="experiments/explicit-web/Security/IdentitySchema.cs"><web:paste select="#ew-identity-schema"/></web:file>

</web:doc>
