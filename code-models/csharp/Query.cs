// The root fields. Deliberately few and deliberately WIDE.
//
// GraphQL's failure mode here is N+1: each resolve may bind a symbol, and a
// generator that walks a project one query per type pays for the compilation
// over and over. So the natural query is "every type matching this filter,
// with the members I need", answered in one pass.

using HotChocolate;
using Hick.Model.CSharp;

namespace Hick.Model.CSharp;

public sealed class Query
{
    /// <summary>What this server is and what it read.</summary>
    public ServerInfo Version([Service] CodeModel model) => model.Info();

    /// <summary>
    /// Types the model could not fully bind. Reported rather than swallowed:
    /// a generator that ran against a half-bound compilation and believed it
    /// saw everything would emit confidently wrong code.
    /// </summary>
    public IReadOnlyList<UnresolvedReference> Unresolved([Service] CodeModel model) =>
        model.Unresolved;

    /// <summary>Every type declared in this project, filtered.</summary>
    /// <param name="namespaceIs">Exact namespace match.</param>
    /// <param name="nameEndsWith">Suffix match on the simple name — the
    /// convention most generators key on (`*Service`, `*Controller`).</param>
    /// <param name="withAttribute">Only types carrying this attribute, named
    /// with or without the `Attribute` suffix.</param>
    public IEnumerable<ITypeDecl> Types(
        [Service] CodeModel model,
        string? namespaceIs = null,
        string? nameEndsWith = null,
        string? withAttribute = null)
    {
        var types = model.Types();
        if (namespaceIs is not null)
            types = types.Where(t => t.Namespace == namespaceIs);
        if (nameEndsWith is not null)
            types = types.Where(t => t.Name.EndsWith(nameEndsWith, StringComparison.Ordinal));
        if (withAttribute is not null)
            types = types.Where(t => HasAttribute(t.Attributes, withAttribute));
        return types;
    }

    /// <summary>One type by its fully qualified name.</summary>
    public ITypeDecl? Type([Service] CodeModel model, string fullName) =>
        model.Types().FirstOrDefault(t => t.FullName == fullName);

    /// <summary>`[NeverExpose]` and `NeverExposeAttribute` are the same
    /// attribute, and a caller should not have to know which spelling the
    /// author used.</summary>
    internal static bool HasAttribute(IReadOnlyList<AppliedAttribute> attributes, string name)
    {
        var bare = name.EndsWith("Attribute", StringComparison.Ordinal)
            ? name[..^"Attribute".Length]
            : name;
        return attributes.Any(a =>
            a.Name == name || a.Name == bare || a.Name == bare + "Attribute");
    }
}
