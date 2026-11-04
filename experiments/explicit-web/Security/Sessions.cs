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
