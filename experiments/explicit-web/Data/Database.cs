using System.Globalization;

namespace ExplicitWeb.Data;

public sealed class DatabaseException(int code, string message) : Exception(message)
{
    public int Code { get; } = code;
}
public sealed class Database
{
    private readonly string path;
    public Database(string path)
    {
        if (!Path.IsPathFullyQualified(path)) throw new ArgumentException("Set the database path to an absolute writable file path.");
        Directory.CreateDirectory(Path.GetDirectoryName(path)!); this.path = path;
    }
    public T Transaction<T>(Func<Sql, T> work)
    {
        using var connection = new Connection(path);
        var sql = new Sql(connection);
        sql.Schema("PRAGMA foreign_keys=ON; BEGIN IMMEDIATE;");
        try { var result = work(sql); sql.Schema("COMMIT;"); return result; }
        catch { sql.Schema("ROLLBACK;"); throw; }
    }
    public void Migrate(params string[] migrations)
    {
        Transaction(sql =>
        {
            sql.Execute($"CREATE TABLE IF NOT EXISTS schema_version(version INTEGER NOT NULL)");
            int version = (int)(sql.Scalar($"SELECT COALESCE(MAX(version),0) FROM schema_version") ?? 0L);
            if (version > migrations.Length) throw new InvalidOperationException("Database schema is newer than this app. Run the matching app version.");
            for (int i = version; i < migrations.Length; i++) sql.Schema(migrations[i]);
            sql.Execute($"DELETE FROM schema_version");
            sql.Execute($"INSERT INTO schema_version VALUES({migrations.Length})"); return true;
        });
    }
}
public sealed class Sql
{
    private readonly Connection connection;
    internal Sql(Connection connection) => this.connection = connection;
    private Statement Command(FormattableString template)
    {
        var args = template.GetArguments();
        var names = args.Select((_, i) => (object)("?" + (i + 1))).ToArray();
        var statement = new Statement(connection, string.Format(CultureInfo.InvariantCulture, template.Format, names));
        try { for (int i = 0; i < args.Length; i++) statement.Bind(i + 1, args[i]); return statement; }
        catch { statement.Dispose(); throw; }
    }
    public int Execute(FormattableString sql)
    {
        using var command = Command(sql); while (command.Step()) { }
        return Native.sqlite3_changes(connection.Handle);
    }
    public long? Scalar(FormattableString sql)
    {
        using var command = Command(sql); return command.Step() && !command.Row.IsDBNull(0) ? command.Row.GetInt64(0) : null;
    }
    public List<T> Query<T>(FormattableString sql, Func<Row, T> read)
    {
        using var command = Command(sql); var result = new List<T>();
        while (command.Step()) result.Add(read(command.Row)); return result;
    }
    internal void Schema(string trustedMigration)
    {
        int status = Native.sqlite3_exec(connection.Handle, trustedMigration, IntPtr.Zero, IntPtr.Zero, out var error);
        if (error != IntPtr.Zero) Native.sqlite3_free(error);
        connection.Check(status);
    }
}
