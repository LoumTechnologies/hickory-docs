// Talking to a hick code model server, so a generator does not have to.
//
// The C# half of what `code-models/clients/python/hick_model.py` does for
// Python, and it exists for the same measured reason: every generator written
// against a model server was re-implementing the same forty lines — spawn the
// process, frame the JSON, notice that GraphQL reports failure inside a
// successful-looking body — and none of it is the interesting part.
//
//     using var model = Model.Start(binary, root);
//     var data = model.Query<ApiSurfaceResult>(ApiSurface.Query);
//     foreach (var service in data.Services) { … }
//
// Deliberately paired with `--client`, not a replacement for it. The client
// emits the records and the query text together so the two cannot drift; this
// is only the transport that carries one to the other. Neither is useful
// alone.
//
// Written for a **file-based app** (`dotnet run gen.cs`), which is why it
// takes no dependency beyond the base class library: adding a NuGet package
// to a generator means a restore, a lock file and a network hole in a cell
// that otherwise needs none.

#nullable enable

using System.Diagnostics;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Hick.Model.Client;

/// <summary>The server refused, died, or was never there.</summary>
public sealed class ModelException : Exception
{
    public ModelException(string message) : base(message) { }
}

/// <summary>
/// One server, held open across a generation pass.
///
/// Held open because binding a compilation is the entire cost — seconds for
/// Roslyn on a small project against a fraction of a millisecond per query
/// afterwards — so a generator that spawns per question pays it per question.
/// </summary>
public sealed class Model : IDisposable
{
    private readonly Process _process;

    private static readonly JsonSerializerOptions Options = new()
    {
        PropertyNameCaseInsensitive = true,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
    };

    private Model(Process process) => _process = process;

    /// <summary>Start a model server over `root`.</summary>
    public static Model Start(string binary, string root)
    {
        var info = new ProcessStartInfo(binary)
        {
            RedirectStandardInput = true,
            RedirectStandardOutput = true,
            UseShellExecute = false,
        };
        info.ArgumentList.Add(root);
        Process process;
        try
        {
            process = Process.Start(info)
                ?? throw new ModelException($"could not start `{binary}`");
        }
        catch (Exception e) when (e is not ModelException)
        {
            // The message a person can act on. A document that uses a model
            // server should declare `<hick:needs bin="…" />` so this is caught
            // before the cell runs rather than from inside a generator.
            throw new ModelException(
                $"no model server at `{binary}`.\n"
                + "  `hick lang` shows which languages have one, and a document that uses one\n"
                + "  should declare it with a needs element so this is caught before the cell\n"
                + $"  runs rather than inside the script. ({e.Message})");
        }
        return new Model(process);
    }

    /// <summary>
    /// Ask one question, and get it back as <typeparamref name="T"/> — which
    /// is the record `hick code-model --client` emitted for this very query.
    /// </summary>
    public T Query<T>(string query, object? variables = null)
    {
        var request = JsonSerializer.Serialize(
            new Request(query, variables), Options);
        string? line;
        try
        {
            _process.StandardInput.WriteLine(request);
            _process.StandardInput.Flush();
            line = _process.StandardOutput.ReadLine();
        }
        catch (IOException e)
        {
            // A server that died between starting and answering closes the
            // pipe, and the raw IOException is a stack trace through
            // StreamWriter that says nothing about what went wrong. Almost
            // always: the binary is not a model server, or it refused the
            // root it was given.
            throw new ModelException(
                $"the model server stopped before answering ({e.Message}).\n"
                + "  Its own output above usually says why — a root it could not read is the\n"
                + "  common one.");
        }
        if (line is null)
            throw new ModelException("the model server closed without answering");

        using var answer = JsonDocument.Parse(line);
        // GraphQL reports failure inside a 200-shaped body, so an unchecked
        // caller treats "you asked for a field that does not exist" as an
        // empty result and generates nothing, silently. This is the single
        // most valuable line in the file.
        if (answer.RootElement.TryGetProperty("errors", out var errors)
            && errors.ValueKind == JsonValueKind.Array
            && errors.GetArrayLength() > 0)
        {
            var messages = errors.EnumerateArray().Select(e =>
                e.TryGetProperty("message", out var m) ? m.GetString() : e.ToString());
            throw new ModelException(
                "the model refused the query:\n  " + string.Join("\n  ", messages));
        }

        if (!answer.RootElement.TryGetProperty("data", out var data))
            throw new ModelException("the model answered with no data");

        return data.Deserialize<T>(Options)
            ?? throw new ModelException($"the answer did not fit {typeof(T).Name}");
    }

    public void Dispose()
    {
        try
        {
            _process.StandardInput.Close();
            if (!_process.WaitForExit(10_000)) _process.Kill(entireProcessTree: true);
        }
        catch (InvalidOperationException)
        {
            // Already gone. Nothing to close and nothing to report: this is a
            // read-only query process, and its death costs the caller nothing
            // a failed query has not already told them about.
        }
        _process.Dispose();
    }

    private sealed record Request(
        [property: JsonPropertyName("query")] string Query,
        [property: JsonPropertyName("variables")] object? Variables);
}

/// <summary>
/// A buffer for building source text.
///
/// The other third of a generator, and the third nobody enjoys: appending to
/// a list and remembering where the commas go. <see cref="Join"/> exists
/// because a parameter list is the single most common thing a generator emits
/// and getting its separators right by hand is where the fiddly bugs live.
/// </summary>
public sealed class Emit
{
    private readonly List<string> _lines = new();

    public Emit Line(params string[] lines)
    {
        _lines.AddRange(lines);
        return this;
    }

    /// <summary>One entry per line with `separator` between — a parameter list.</summary>
    public Emit Join(IEnumerable<string> parts, string separator = ",\n")
    {
        var list = parts.ToList();
        if (list.Count > 0) _lines.Add(string.Join(separator, list));
        return this;
    }

    public Emit Blank()
    {
        _lines.Add("");
        return this;
    }

    public string Text() => string.Join("\n", _lines) + "\n";

    public void Write(string path)
    {
        var directory = Path.GetDirectoryName(path);
        if (!string.IsNullOrEmpty(directory)) Directory.CreateDirectory(directory);
        File.WriteAllText(path, Text());
    }
}
