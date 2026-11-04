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
