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
