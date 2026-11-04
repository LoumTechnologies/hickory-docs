<web:doc xmlns:web="http://www.hickorydocs.com/1.0">
# Persist data through parameterized transactions

For app authors adding a durable aggregate. This small library uses SQLite rather than inventing an ORM. SQL templates parameterize values; table and column names stay literal. One transaction owns a complete callback, with rollback on exceptions. The reference app uses optimistic versions for conflicting edits.

## Data data csproj

Keep assembly boundaries real: internal constructors cannot be called by consumers. Nullable warnings are errors; deterministic builds and dependency locks make evaluation reproducible.

<web:copy id="ew-Data-Data-csproj">
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup><PackageId>ExplicitWeb.Data</PackageId>
    <TargetFramework>net10.0</TargetFramework><Nullable>enable</Nullable><ImplicitUsings>enable</ImplicitUsings>
    <TreatWarningsAsErrors>true</TreatWarningsAsErrors><UseAppHost>false</UseAppHost>
    <RestorePackagesWithLockFile>true</RestorePackagesWithLockFile><Deterministic>true</Deterministic>
  </PropertyGroup>
  <ItemGroup><ProjectReference Include="../Core/Core.csproj" /></ItemGroup>
</Project>
</web:copy>

## Database

Each operation opens a connection, enables foreign keys, and sets a finite lock wait. Schema migrations run in a transaction and reject a database created by a newer application. A command accepts FormattableString, so ordinary interpolated data becomes a bound parameter rather than SQL syntax. This does not permit dynamic identifiers.

<web:copy id="ew-database">
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
</web:copy>

## Sqlite abi

The storage driver targets the stable SQLite C ABI supplied by the OS, not a custom database engine. It opens a full-mutex connection, binds copied UTF-8/blob values, checks every native status, and finalizes statements before closing the connection. Row snapshots copy native values before statement disposal, so retaining a row cannot read freed memory. macOS and Linux normally supply SQLite; Windows requires sqlite3.dll on the library search path. These are deployment prerequisites. The driver has focused transaction, parameter and restart witnesses; it is not a full ADO.NET provider.

<web:copy id="ew-sqlite-abi">
using System.Text;
using System.Runtime.InteropServices;

namespace ExplicitWeb.Data;

internal static class Native
{
    static Native() => NativeLibrary.SetDllImportResolver(typeof(Native).Assembly, (name, assembly, search) =>
        name == "explicit_sqlite" ? NativeLibrary.Load(OperatingSystem.IsMacOS() ? "/usr/lib/libsqlite3.dylib" : OperatingSystem.IsWindows() ? "sqlite3.dll" : "libsqlite3.so.0", assembly, search) : IntPtr.Zero);
    private const string Library = "explicit_sqlite";
    [DllImport(Library)] internal static extern int sqlite3_open_v2([MarshalAs(UnmanagedType.LPUTF8Str)] string path, out IntPtr db, int flags, IntPtr vfs);
    [DllImport(Library)] internal static extern int sqlite3_close_v2(IntPtr db);
    [DllImport(Library)] internal static extern int sqlite3_busy_timeout(IntPtr db, int milliseconds);
    [DllImport(Library)] internal static extern IntPtr sqlite3_errmsg(IntPtr db);
    [DllImport(Library)] internal static extern int sqlite3_exec(IntPtr db, [MarshalAs(UnmanagedType.LPUTF8Str)] string sql, IntPtr callback, IntPtr arg, out IntPtr error);
    [DllImport(Library)] internal static extern void sqlite3_free(IntPtr value);
    [DllImport(Library)] internal static extern int sqlite3_prepare_v2(IntPtr db, [MarshalAs(UnmanagedType.LPUTF8Str)] string sql, int bytes, out IntPtr statement, IntPtr tail);
    [DllImport(Library)] internal static extern int sqlite3_finalize(IntPtr statement);
    [DllImport(Library)] internal static extern int sqlite3_step(IntPtr statement);
    [DllImport(Library)] internal static extern int sqlite3_changes(IntPtr db);
    [DllImport(Library)] internal static extern int sqlite3_bind_null(IntPtr statement, int index);
    [DllImport(Library)] internal static extern int sqlite3_bind_int64(IntPtr statement, int index, long value);
    [DllImport(Library)] internal static extern int sqlite3_bind_double(IntPtr statement, int index, double value);
    [DllImport(Library)] internal static extern int sqlite3_bind_text(IntPtr statement, int index, [MarshalAs(UnmanagedType.LPUTF8Str)] string value, int bytes, IntPtr destructor);
    [DllImport(Library)] internal static extern int sqlite3_bind_zeroblob(IntPtr statement, int index, int bytes);
    [DllImport(Library)] internal static extern int sqlite3_bind_blob(IntPtr statement, int index, byte[] value, int bytes, IntPtr destructor);
    [DllImport(Library)] internal static extern int sqlite3_column_count(IntPtr statement);
    [DllImport(Library)] internal static extern int sqlite3_column_type(IntPtr statement, int index);
    [DllImport(Library)] internal static extern double sqlite3_column_double(IntPtr statement, int index);
    [DllImport(Library)] internal static extern long sqlite3_column_int64(IntPtr statement, int index);
    [DllImport(Library)] internal static extern IntPtr sqlite3_column_text(IntPtr statement, int index);
    [DllImport(Library)] internal static extern IntPtr sqlite3_column_blob(IntPtr statement, int index);
    [DllImport(Library)] internal static extern int sqlite3_column_bytes(IntPtr statement, int index);
}
</web:copy>

## Sqlite connection

Own one native connection for one transaction. Opening failures close the partially opened handle; operation errors retain SQLite’s private diagnostic for the host logger.

<web:copy id="ew-sqlite-connection">
internal sealed class Connection : IDisposable
{
    internal IntPtr Handle { get; }
    internal Connection(string path)
    {
        int status = Native.sqlite3_open_v2(path, out var handle, 2 | 4 | 0x10000, IntPtr.Zero); Handle = handle;
        if (status != 0)
        {
            var message = Marshal.PtrToStringUTF8(Native.sqlite3_errmsg(handle)) ?? "Cannot open SQLite database.";
            Native.sqlite3_close_v2(handle); throw new DatabaseException(status, message);
        }
        Check(Native.sqlite3_busy_timeout(Handle, 5000));
    }
    internal void Check(int status)
    {
        if (status != 0) throw new DatabaseException(status & 255, Marshal.PtrToStringUTF8(Native.sqlite3_errmsg(Handle)) ?? "SQLite operation failed.");
    }
    public void Dispose() => Native.sqlite3_close_v2(Handle);
}
</web:copy>

## Sqlite statement

Bind values before stepping. SQLITE_TRANSIENT tells SQLite to copy marshaled text/blob memory. Finalize the statement on every managed exit path.

<web:copy id="ew-sqlite-statement">
internal sealed class Statement : IDisposable
{
    private readonly Connection connection;
    private readonly IntPtr handle;
    public Row Row => new(handle);
    internal Statement(Connection connection, string sql)
    {
        this.connection = connection;
        connection.Check(Native.sqlite3_prepare_v2(connection.Handle, sql, -1, out handle, IntPtr.Zero));
    }
    public void Bind(int index, object? value)
    {
        var transient = new IntPtr(-1);
        int status = value switch
        {
            null => Native.sqlite3_bind_null(handle, index),
            byte[] bytes when bytes.Length == 0 => Native.sqlite3_bind_zeroblob(handle, index, 0),
            byte[] bytes => Native.sqlite3_bind_blob(handle, index, bytes, bytes.Length, transient),
            bool flag => Native.sqlite3_bind_int64(handle, index, flag ? 1 : 0),
            int number => Native.sqlite3_bind_int64(handle, index, number),
            long number => Native.sqlite3_bind_int64(handle, index, number),
            double number => Native.sqlite3_bind_double(handle, index, number),
            string text => Native.sqlite3_bind_text(handle, index, text, Encoding.UTF8.GetByteCount(text), transient),
            _ => throw new ArgumentException("Use null, string, integer, bool, double or byte[] as a SQL parameter.")
        };
        connection.Check(status);
    }
    public bool Step()
    {
        int status = Native.sqlite3_step(handle);
        if (status == 100) return true;
        if (status == 101) return false;
        connection.Check(status); return false;
    }
    public void Dispose() => Native.sqlite3_finalize(handle);
}
</web:copy>

## Sqlite row

Copy columns into managed values while the statement is valid. A caller may retain a row after the transaction without reading freed native memory; the runtime witness checks that lifetime boundary.

<web:copy id="ew-sqlite-row">
public sealed class Row
{
    private readonly object?[] values;
    internal Row(IntPtr statement)
    {
        values = new object?[Native.sqlite3_column_count(statement)];
        for (int i = 0; i < values.Length; i++)
        {
            int type = Native.sqlite3_column_type(statement, i);
            if (type == 5) values[i] = null;
            else if (type == 1) values[i] = Native.sqlite3_column_int64(statement, i);
            else if (type == 2) values[i] = Native.sqlite3_column_double(statement, i);
            else if (type == 3) values[i] = Marshal.PtrToStringUTF8(Native.sqlite3_column_text(statement, i), Native.sqlite3_column_bytes(statement, i)) ?? "";
            else if (type == 4)
            {
                var bytes = new byte[Native.sqlite3_column_bytes(statement, i)];
                if (bytes.Length > 0) Marshal.Copy(Native.sqlite3_column_blob(statement, i), bytes, 0, bytes.Length);
                values[i] = bytes;
            }
            else throw new InvalidOperationException("Unsupported SQLite column type.");
        }
    }
    public bool IsDBNull(int index) => values[index] is null;
    public string GetString(int index) => (string)(values[index] ?? throw new InvalidOperationException("Column is null."));
    public long GetInt64(int index) => (long)(values[index] ?? throw new InvalidOperationException("Column is null."));
    public double GetDouble(int index) => Convert.ToDouble(values[index], System.Globalization.CultureInfo.InvariantCulture);
    public bool GetBoolean(int index) => GetInt64(index) != 0;
    public object this[int index] => values[index] ?? throw new InvalidOperationException("Column is null.");
}
</web:copy>

## Assemble these outputs

<web:file path="experiments/explicit-web/Data/Data.csproj"><web:paste select="#ew-Data-Data-csproj"/></web:file>
<web:file path="experiments/explicit-web/Data/Database.cs"><web:paste select="#ew-database"/></web:file>
<web:file path="experiments/explicit-web/Data/Native.cs"><web:paste select="#ew-sqlite-abi"/><web:paste select="#ew-sqlite-connection"/><web:paste select="#ew-sqlite-statement"/><web:paste select="#ew-sqlite-row"/></web:file>

</web:doc>
