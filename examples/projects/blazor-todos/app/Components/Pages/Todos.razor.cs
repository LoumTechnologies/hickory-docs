using System.Data.Common;
using System.Security.Claims;
using HickoryTodos.Data;
using HickoryTodos.Models;
using Microsoft.AspNetCore.Components;
using Microsoft.AspNetCore.Components.Authorization;
using Microsoft.EntityFrameworkCore;

namespace HickoryTodos.Components.Pages;

public partial class Todos
{
    [Inject] private AuthenticationStateProvider Authentication { get; set; } = default!;
    [Inject] private IDbContextFactory<ApplicationDbContext> Db { get; set; } = default!;
    [Inject] private ILogger<Todos> Logger { get; set; } = default!;

    private List<TodoItem> todos = [];
    private TodoItem draft = new();
    private string? error;
    private bool busy;

    protected override Task OnInitializedAsync() => WithOwnerAsync(ReadAsync);

    private static IQueryable<TodoItem> Owned(ApplicationDbContext db, string ownerId) =>
        db.Todos.Where(todo => todo.OwnerId == ownerId);

    private async Task ReadAsync(ApplicationDbContext db, string ownerId) =>
        todos = await Owned(db, ownerId).AsNoTracking().OrderBy(todo => todo.Id).ToListAsync();

    private void Edit(TodoItem item)
    {
        draft = new TodoItem { Id = item.Id };
        CopyEditable(item, draft);
    }

    private void ResetDraft() => draft = new TodoItem();

    private Task SaveAsync() => WithOwnerAsync(async (db, ownerId) =>
    {
        var creating = draft.Id == 0;
        var item = creating ? new TodoItem { OwnerId = ownerId }
            : await Owned(db, ownerId).SingleOrDefaultAsync(todo => todo.Id == draft.Id);
        if (item is null)
        {
            await MissingTodoAsync(db, ownerId);
            return;
        }

        CopyEditable(draft, item);
        if (creating)
            db.Todos.Add(item);
        await db.SaveChangesAsync();
        ResetDraft();
        await ReadAsync(db, ownerId);
    });

    private Task DeleteAsync(int id) => WithOwnerAsync(async (db, ownerId) =>
    {
        var item = await Owned(db, ownerId).SingleOrDefaultAsync(todo => todo.Id == id);
        if (item is null)
        {
            await MissingTodoAsync(db, ownerId);
            return;
        }

        db.Todos.Remove(item);
        await db.SaveChangesAsync();
        if (draft.Id == id)
            ResetDraft();
        await ReadAsync(db, ownerId);
    });

    private async Task MissingTodoAsync(ApplicationDbContext db, string ownerId)
    {
        ResetDraft();
        await ReadAsync(db, ownerId);
        error = "That todo is no longer available.";
    }

    private async Task WithOwnerAsync(Func<ApplicationDbContext, string, Task> operation)
    {
        if (busy)
            return;

        busy = true;
        error = null;
        try
        {
            var user = (await Authentication.GetAuthenticationStateAsync()).User;
            if (user.Identity?.IsAuthenticated != true ||
                user.FindFirstValue(ClaimTypes.NameIdentifier) is not { Length: > 0 } ownerId)
            {
                todos = [];
                ResetDraft();
                error = "Sign in to access your todos.";
                return;
            }

            await using var db = await Db.CreateDbContextAsync();
            await operation(db, ownerId);
        }
        catch (Exception exception) when (exception is DbException or DbUpdateException)
        {
            Logger.LogError(exception, "Could not complete a todo operation.");
            error = "There was a database error. Reload the page to see the current list.";
        }
        finally
        {
            busy = false;
        }
    }
}
