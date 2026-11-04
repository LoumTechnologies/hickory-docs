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
