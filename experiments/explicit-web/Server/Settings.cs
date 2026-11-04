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
