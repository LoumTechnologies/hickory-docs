using ExplicitWeb;
using ExplicitWeb.Security;

namespace TaskApp;

public static class Pages
{
    public static readonly Route<Guid> Item = Routes.Id("GET", "/tasks/{id}");
    public static readonly Route<Unit> Home = Routes.At("GET", "/");
    public static Response Layout(string title, Html body) => Response.Page(Html.Template($"""
        <!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
        <title>{title} · Explicit Tasks</title><link rel="stylesheet" href="/site.css"></head>
        <body><main><header><a href="/">Explicit Tasks</a><span>Ordinary functions. Checked boundaries.</span></header>{body}</main></body></html>
        """));
    public static Response Account(Session session, bool register) => Layout(register ? "Create account" : "Sign in", Html.Template($"""
        <h1>{(register ? "Create your account" : "Welcome back")}</h1>
        <form method="post" action="{(register ? "/register" : "/login")}">
        <input type="hidden" name="csrf" value="{session.Csrf}">
        <label>Username <input name="name" required minlength="3" maxlength="40" autocomplete="username"></label>
        <label>Password <input name="password" type="password" required minlength="12" maxlength="128" autocomplete="{(register ? "new-password" : "current-password")}"></label>
        <button>{(register ? "Create account" : "Sign in")}</button></form><p><a href="{(register ? "/login" : "/register")}">{(register ? "Already registered? Sign in" : "Need an account? Register")}</a></p>
        """));
    public static Response Welcome() => Layout("Welcome", Html.Template($"""
        <h1>A small place to get things done.</h1><p>Create a private task list. Your work stays with your account.</p>
        <p><a class="button" href="/register">Create account</a> <a href="/login">Sign in</a></p>
        """));
    public static Response List(Session session, Principal user, IEnumerable<TaskItem> tasks) => Layout("Your tasks", Html.Template($"""
        <div class="heading"><h1>{user.Name}’s tasks</h1><form method="post" action="/logout"><input type="hidden" name="csrf" value="{session.Csrf}"><button class="secondary">Sign out</button></form></div>
        <form method="post" action="/tasks" class="new-task"><input type="hidden" name="csrf" value="{session.Csrf}"><label>New task <input name="title" required maxlength="200" placeholder="What needs doing?"></label><button>Add task</button></form>
        <ul class="tasks">{Html.Join(tasks.Select(t => Html.Template($"""
            <li><a href="{Item.Link(t.Id).Value}">{t.Title}</a><span>{(t.Done ? "Complete" : "Open")}</span></li>
            """)))}</ul><p class="hint">Select a task to edit, complete or delete it.</p>
        """));
    public static Response Edit(Session session, TaskItem task) => Layout("Edit task", Html.Template($"""
        <p><a href="/">← All tasks</a></p><h1>Edit task</h1><form method="post" action="{Item.Link(task.Id).Value}">
        <input type="hidden" name="csrf" value="{session.Csrf}"><input type="hidden" name="version" value="{task.Version}">
        <label>Title <input name="title" value="{task.Title}" required maxlength="200"></label>
        <label>Status <select name="done"><option value="{(task.Done ? "true" : "false")}">{(task.Done ? "Complete" : "Open")}</option><option value="{(task.Done ? "false" : "true")}">{(task.Done ? "Open" : "Complete")}</option></select></label>
        <div class="actions"><button name="action" value="save">Save changes</button><button class="danger" name="action" value="delete">Delete task</button></div></form>
        """));
    public static Response Error(Problem problem) => Layout("Please try again", Html.Template($"""
        <h1>We couldn’t complete that request</h1><p>{problem.Message}</p><p><a href="/">Return home</a> or use your browser’s Back button to correct the form.</p>
        """)).WithStatus(problem.Status);
    public const string Css = """
        :root{font-family:system-ui,sans-serif;color:#18332c;background:#f4f5ee}*{box-sizing:border-box}body{margin:0}main{max-width:760px;margin:auto;padding:32px 24px}header{display:flex;justify-content:space-between;gap:20px;padding-bottom:28px;border-bottom:1px solid #cdd7cb;margin-bottom:48px}header a{font-weight:750;text-decoration:none}header span,.hint{font-size:.85rem;color:#567063}h1{font-size:2.3rem;letter-spacing:-.04em}a{color:#245c46}form{display:grid;gap:20px}label{display:grid;gap:8px}input,select{font:inherit;padding:12px;border:1px solid #b8c9ba;border-radius:6px;background:#fff}button,.button{font:inherit;border:0;border-radius:6px;background:#245c46;color:white;padding:12px 20px;cursor:pointer;text-decoration:none;display:inline-block}button.secondary{background:#e2e9df;color:#245c46}.danger{background:#873d35}.heading,.actions{display:flex;align-items:center;justify-content:space-between;gap:12px}.tasks{list-style:none;padding:0;margin:32px 0}.tasks li{padding:18px 0;border-bottom:1px solid #cdd7cb;display:flex;justify-content:space-between;gap:20px}.tasks span{font-size:.85rem;color:#567063}.new-task{background:#e8eee1;padding:24px;border-radius:8px}@media(max-width:540px){header span{display:none}.heading{display:block}h1{font-size:2rem}}
        """;
}
