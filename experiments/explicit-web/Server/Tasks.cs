using ExplicitWeb;
using ExplicitWeb.Data;
using ExplicitWeb.Security;

namespace TaskApp;

public sealed class Title
{
    public string Value { get; }
    private Title(string value) => Value = value;
    public static Result<Title> Parse(string? value)
    {
        value = value?.Trim();
        return string.IsNullOrEmpty(value) || value.Length > 200 || value.Any(char.IsControl)
            ? Result<Title>.Fail(422, "Enter a task title of 1–200 characters without control characters.")
            : Result<Title>.Ok(new(value));
    }
}
public sealed record TaskItem(Guid Id, string Title, bool Done, long Version);
public sealed record TaskInput(string? Title, bool Done = false, long Version = 0);
public sealed record LoginInput(string? Name, string? Password);
public sealed class Tasks(Database database)
{
    private static TaskItem Read(ExplicitWeb.Data.Row r) => new(Guid.Parse(r.GetString(0)), r.GetString(1), r.GetBoolean(2), r.GetInt64(3));
    public List<TaskItem> List(Principal user) => database.Transaction(sql => sql.Query($"SELECT id,title,done,version FROM tasks WHERE owner={user.UserId} ORDER BY rowid DESC", Read));
    public Result<TaskItem> Find(Principal user, Guid id) => database.Transaction(sql => sql.Query($"SELECT id,title,done,version FROM tasks WHERE owner={user.UserId} AND id={id.ToString()}", Read))
        .SingleOrDefault() is { } task ? Result<TaskItem>.Ok(task) : Result<TaskItem>.Fail(404, "Task not found. Return to your task list.");
    public TaskItem Create(WritePermit permit, Title title) => database.Transaction(sql =>
    {
        var task = new TaskItem(Guid.NewGuid(), title.Value, false, 1);
        sql.Execute($"INSERT INTO tasks(id,owner,title,done,version) VALUES({task.Id.ToString()},{permit.UserId},{task.Title},{false},{1})");
        return task;
    });
    public Result<TaskItem> Update(WritePermit permit, Guid id, Title title, bool done, long version) => database.Transaction(sql =>
    {
        var count = sql.Execute($"UPDATE tasks SET title={title.Value},done={done},version=version+1 WHERE id={id.ToString()} AND owner={permit.UserId} AND version={version}");
        if (count == 1) return Result<TaskItem>.Ok(new(id, title.Value, done, version + 1));
        return Failure<TaskItem>(sql, permit, id);
    });
    public Result<bool> Delete(WritePermit permit, Guid id, long version) => database.Transaction(sql =>
    {
        var count = sql.Execute($"DELETE FROM tasks WHERE id={id.ToString()} AND owner={permit.UserId} AND version={version}");
        return count == 1 ? Result<bool>.Ok(true) : Failure<bool>(sql, permit, id);
    });
    private static Result<T> Failure<T>(Sql sql, WritePermit permit, Guid id) where T : notnull =>
        sql.Scalar($"SELECT COUNT(*) FROM tasks WHERE id={id.ToString()} AND owner={permit.UserId}") == 0
            ? Result<T>.Fail(404, "Task not found. Return to your task list.")
            : Result<T>.Fail(409, "This task changed since you loaded it. Reload before editing again.");
}
