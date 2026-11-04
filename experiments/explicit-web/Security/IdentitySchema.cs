namespace ExplicitWeb.Security;

public static class IdentitySchema
{
    public const string Migration = """
        CREATE TABLE users(id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE, salt BLOB NOT NULL, password BLOB NOT NULL);
        CREATE TABLE sessions(hash TEXT PRIMARY KEY, user_id TEXT REFERENCES users(id) ON DELETE CASCADE, csrf TEXT NOT NULL, expires INTEGER NOT NULL);
        CREATE INDEX sessions_expiry ON sessions(expires);
        """;
}
