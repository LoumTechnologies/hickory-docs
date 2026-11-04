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
