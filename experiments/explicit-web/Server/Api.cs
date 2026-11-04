using ExplicitWeb;
using ExplicitWeb.Security;

namespace TaskApp;

public sealed partial class Application
{
    private Response ApiCollection(Request request, Session session)
    {
        if (Reading(request)) return session.RequireUser().Match(user => Response.Json(tasks.List(user)), p => Error(request, p));
        if (request.Method != "POST") return Method("GET, HEAD, POST");
        return Permit(request, session).Bind(permit => request.Json<TaskInput>().Bind(input => Title.Parse(input.Title)
            .Match(title => Result<TaskItem>.Ok(tasks.Create(permit, title)), p => Result<TaskItem>.Fail(p.Status, p.Message))))
            .Match(task => Response.Json(task, 201).WithHeader("Location", ApiItem.Link(task.Id).Value), p => Error(request, p));
    }
    private Response ApiTask(Request request, Session session)
    {
        var route = Routes.Id(request.Method, "/api/tasks/{id}");
        if (Reading(request)) return session.RequireUser().Bind(user => route.Match(request).Bind(match => tasks.Find(user, match.Value)))
            .Match(task => Response.Json(task), p => Error(request, p));
        if (request.Method is not ("PUT" or "DELETE")) return Method("GET, HEAD, PUT, DELETE");
        return Permit(request, session).Match(permit => route.Match(request).Match(match =>
            request.Json<TaskInput>().Match(input =>
            {
                if (input.Version < 1) return Error(request, new Problem(422, "Supply the current positive task version."));
                if (request.Method == "DELETE") return tasks.Delete(permit, match.Value, input.Version).Match(_ => Response.Empty(), p => Error(request, p));
                return Title.Parse(input.Title).Bind(title => tasks.Update(permit, match.Value, title, input.Done, input.Version))
                    .Match(task => Response.Json(task), p => Error(request, p));
            }, p => Error(request, p)), p => Error(request, p)), p => Error(request, p));
    }
}
