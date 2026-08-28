// `hick-model-csharp <root>` — the C# code model, served as GraphQL.
//
// One JSON object per line on stdin, one per line on stdout. Line-delimited
// rather than Content-Length framed because that is what `hick mcp` already
// speaks, and a second framing in one product is a second thing to get wrong.
//
// A local pipe rather than a port: this answers questions about somebody's
// source code, and a socket anyone on the machine can reach is not the
// default a tool like that should have.

using System.Text.Json;
using HotChocolate;
using HotChocolate.Execution;
using Hick.Model.CSharp;
using Microsoft.Extensions.DependencyInjection;

var root = args.Length > 0 ? args[0] : Directory.GetCurrentDirectory();
if (!Directory.Exists(root))
{
    Console.Error.WriteLine($"hick-model-csharp: no such directory: {root}");
    Console.Error.WriteLine("  Pass the source root to model, e.g. `hick-model-csharp src/Domain`.");
    return 2;
}

// Loaded once. Binding a compilation is the expensive part, and the server
// exists so a generator pays it once instead of per query.
CodeModel model;
try
{
    model = CodeModel.Load(System.IO.Path.GetFullPath(root));
}
catch (Exception e)
{
    Console.Error.WriteLine($"hick-model-csharp: could not read {root}: {e.Message}");
    return 2;
}

var executor = await new ServiceCollection()
    // The model is a singleton because binding a compilation is the whole
    // cost, and it is registered BEFORE AddGraphQL so it lands on the service
    // collection rather than the executor builder.
    .AddSingleton(model)
    .AddGraphQL()
    .AddQueryType<Query>()
    .AddType<ClassDecl>()
    .AddType<RecordDecl>()
    .AddType<StructDecl>()
    .AddType<InterfaceDecl>()
    .AddType<EnumDecl>()
    .BuildRequestExecutorAsync();

Console.Error.WriteLine(
    $"hick-model-csharp: {model.FileCount} file(s) under {model.Root}; ready");
foreach (var gap in model.Unresolved.Take(5))
    Console.Error.WriteLine($"  unresolved: {gap.Message}");

// GraphQL over the wire spells its fields lowercase, and System.Text.Json
// matches property names case-sensitively unless told otherwise — so
// `{"query": …}` bound to nothing and every request looked empty.
var JsonOptions = new JsonSerializerOptions { PropertyNameCaseInsensitive = true };
var stdout = Console.Out;
string? line;
while ((line = Console.In.ReadLine()) is not null)
{
    if (line.Trim().Length == 0) continue;

    string response;
    try
    {
        var request = JsonSerializer.Deserialize<GraphQLRequest>(line, JsonOptions)
            ?? throw new InvalidOperationException("empty request");
        var builder = OperationRequestBuilder.New().SetDocument(request.Query ?? "");
        if (request.Variables is { Count: > 0 })
            builder.SetVariableValues(request.Variables.ToDictionary(
                kv => kv.Key, kv => (object?)ToClr(kv.Value)));
        await using var result = await executor.ExecuteAsync(builder.Build());
        response = result.ToJson(withIndentations: false);
    }
    catch (Exception e)
    {
        // A malformed request must not end the session: the caller is a
        // generator being written, and being written means getting it wrong.
        response = JsonSerializer.Serialize(new
        {
            errors = new[] { new { message = e.Message } },
        });
    }

    // One line, always: a response containing a newline would desynchronise
    // the caller's reader, which is the same rule `hick mcp` follows about
    // stdout carrying the protocol and nothing else.
    stdout.WriteLine(response.Replace("\n", " ").Replace("\r", ""));
    stdout.Flush();
}
return 0;

static object? ToClr(JsonElement e) => e.ValueKind switch
{
    JsonValueKind.String => e.GetString(),
    JsonValueKind.Number => e.TryGetInt64(out var i) ? i : e.GetDouble(),
    JsonValueKind.True => true,
    JsonValueKind.False => false,
    JsonValueKind.Null or JsonValueKind.Undefined => null,
    JsonValueKind.Array => e.EnumerateArray().Select(ToClr).ToList(),
    _ => e.GetRawText(),
};

internal sealed class GraphQLRequest
{
    public string? Query { get; set; }
    public Dictionary<string, JsonElement>? Variables { get; set; }
}
