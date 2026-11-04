namespace ExplicitWeb;

public sealed record Problem(int Status, string Message);
public sealed class Result<T> where T : notnull
{
    private readonly T? value;
    private readonly Problem? problem;
    private Result(T value) => this.value = value;
    private Result(Problem problem) => this.problem = problem;
    public static Result<T> Ok(T value) => new(value ?? throw new ArgumentNullException(nameof(value)));
    public static Result<T> Fail(int status, string message) => new(new Problem(status, message));
    public R Match<R>(Func<T, R> success, Func<Problem, R> failure)
        => problem is null ? success(value!) : failure(problem);
    public Result<R> Bind<R>(Func<T, Result<R>> next) where R : notnull
        => Match(next, p => Result<R>.Fail(p.Status, p.Message));
}
