using ExplicitWeb;
using ExplicitWeb.Security;

namespace TaskApp;

public sealed partial class Application
{
    private Response FormCreate(Request request, Session session) => Permit(request, session).Match(permit =>
        Fields(request).Bind(fields => Title.Parse(Field(fields, "title"))).Match(title =>
        { tasks.Create(permit, title); return Home(); }, p => Error(request, p)), p => Error(request, p));
    private Response FormTask(Request request, Session session)
    {
        var route = Routes.Id(request.Method, "/tasks/{id}");
        if (Reading(request)) return session.RequireUser().Bind(user => route.Match(request).Bind(match => tasks.Find(user, match.Value)))
            .Match(task => Pages.Edit(session, task), p => Error(request, p));
        if (request.Method != "POST") return Method("GET, HEAD, POST");
        return Permit(request, session).Match(permit => route.Match(request).Match(match => Fields(request).Match(fields =>
        {
            if (!long.TryParse(Field(fields, "version"), out var version) || version < 1)
                return Error(request, new Problem(422, "Reload the task to get its current version."));
            if (Field(fields, "action") == "delete") return tasks.Delete(permit, match.Value, version).Match(_ => Home(), p => Error(request, p));
            if (Field(fields, "action") != "save") return Error(request, new Problem(422, "Choose Save changes or Delete task."));
            return Title.Parse(Field(fields, "title")).Bind(title => tasks.Update(permit, match.Value, title, Field(fields, "done") == "true", version))
                .Match(_ => Home(), p => Error(request, p));
        }, p => Error(request, p)), p => Error(request, p)), p => Error(request, p));
    }
}
