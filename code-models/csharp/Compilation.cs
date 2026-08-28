// Turning a directory of C# into the model.
//
// A real `CSharpCompilation`, so every answer comes from the compiler rather
// than from a pattern over text. That is the whole reason to prefer this over
// a regex: "is this property marked [NeverExpose]" and "is this a
// DateTimeOffset" survive a type alias, a using alias, and a rename.

using System.Collections.Immutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Acc = Microsoft.CodeAnalysis.Accessibility;

namespace Hick.Model.CSharp;

public sealed class CodeModel
{
    private readonly CSharpCompilation _compilation;
    public string Root { get; }
    public int FileCount { get; }
    public IReadOnlyList<UnresolvedReference> Unresolved { get; }

    private CodeModel(CSharpCompilation compilation, string root, int fileCount,
        IReadOnlyList<UnresolvedReference> unresolved)
    {
        _compilation = compilation;
        Root = root;
        FileCount = fileCount;
        Unresolved = unresolved;
    }

    public static CodeModel Load(string root)
    {
        var files = Directory
            .GetFiles(root, "*.cs", SearchOption.AllDirectories)
            // Build output is not the project's source, and a generator that
            // reads it will find every type twice.
            .Where(p => !p.Contains($"{Path.DirectorySeparatorChar}obj{Path.DirectorySeparatorChar}")
                     && !p.Contains($"{Path.DirectorySeparatorChar}bin{Path.DirectorySeparatorChar}"))
            .OrderBy(p => p, StringComparer.Ordinal)
            .ToList();

        var trees = files
            .Select(p => CSharpSyntaxTree.ParseText(File.ReadAllText(p), path: p))
            .ToList();

        // The SDK's implicit usings. Without them a project with
        // ImplicitUsings enabled — which is every modern one — reads as though
        // System does not exist, and every DateTimeOffset is an unresolved
        // type. Supplying them is this server's job, not the caller's.
        trees.Add(CSharpSyntaxTree.ParseText("""
            global using System;
            global using System.Collections.Generic;
            global using System.IO;
            global using System.Linq;
            global using System.Net.Http;
            global using System.Threading;
            global using System.Threading.Tasks;
            """, path: "<implicit usings>"));

        var references = Directory
            .GetFiles(System.Runtime.InteropServices.RuntimeEnvironment.GetRuntimeDirectory(), "*.dll")
            .Where(p => !Path.GetFileName(p).StartsWith("api-ms-", StringComparison.Ordinal))
            .Select(p => (MetadataReference)MetadataReference.CreateFromFile(p));

        var compilation = CSharpCompilation.Create(
            "HickCodeModel",
            trees,
            references,
            new CSharpCompilationOptions(
                OutputKind.DynamicallyLinkedLibrary,
                nullableContextOptions: NullableContextOptions.Enable));

        // Only the diagnostics that mean a SYMBOL is wrong. A project that
        // does not build for unrelated reasons still has a readable model,
        // and refusing to answer would be less useful than answering with the
        // gaps named.
        var unresolved = compilation.GetDiagnostics()
            .Where(d => d.Severity == DiagnosticSeverity.Error && d.Id is "CS0246" or "CS0234")
            .Take(50)
            .Select(d => new UnresolvedReference(d.GetMessage(), SpanOf(d.Location)))
            .ToList();

        return new CodeModel(compilation, root, files.Count, unresolved);
    }

    public ServerInfo Info() => new(
        "hick-model-csharp",
        typeof(CodeModel).Assembly.GetName().Version?.ToString() ?? "0.0.0",
        "csharp",
        Root,
        FileCount);

    /// <summary>Every type declared in THIS project.</summary>
    ///
    /// <remarks>
    /// `compilation.Assembly.GlobalNamespace`, never
    /// `compilation.GlobalNamespace`: the latter is every referenced assembly
    /// as well, and the framework is full of public types called *Service. A
    /// generator that walked it proposed REST endpoints for
    /// ITypeResolutionService, which is the failure this method exists to
    /// make impossible rather than merely documented.
    /// </remarks>
    public IEnumerable<ITypeDecl> Types() =>
        Flatten(_compilation.Assembly.GlobalNamespace)
            .SelectMany(ns => ns.GetTypeMembers())
            .Where(t => t.CanBeReferencedByName)
            .OrderBy(t => t.ToDisplayString(), StringComparer.Ordinal)
            .Select(Convert)
            .OfType<ITypeDecl>();

    private static IEnumerable<INamespaceSymbol> Flatten(INamespaceSymbol ns)
    {
        yield return ns;
        foreach (var child in ns.GetNamespaceMembers())
            foreach (var deeper in Flatten(child))
                yield return deeper;
    }

    // ---- symbol → model -----------------------------------------------

    private ITypeDecl? Convert(INamedTypeSymbol t)
    {
        var name = t.Name;
        var full = t.ToDisplayString();
        var ns = t.ContainingNamespace?.IsGlobalNamespace == true
            ? ""
            : t.ContainingNamespace?.ToDisplayString() ?? "";
        var acc = Map(t.DeclaredAccessibility);
        // Plural by construction: a partial type has one symbol and several
        // declarations, and the model shows both rather than picking one.
        var decls = t.Locations.Where(l => l.IsInSource).Select(SpanOf).OfType<Span>().ToList();
        var attrs = Attributes(t);
        // `!IsImplicitlyDeclared` is load-bearing. A record gets a synthesised
        // `EqualityContract` property that no one wrote and no one wants on
        // the wire — a generator that trusted this list emitted it as the
        // first field of every DTO. Anything the compiler invented is the
        // compiler's business, not the model's.
        var props = t.GetMembers().OfType<IPropertySymbol>()
            .Where(p => p.CanBeReferencedByName && !p.IsImplicitlyDeclared)
            .Select(Convert).ToList();
        var methods = t.GetMembers().OfType<IMethodSymbol>()
            .Where(m => m.MethodKind == MethodKind.Ordinary
                     && m.CanBeReferencedByName
                     && !m.IsImplicitlyDeclared)
            .Select(Convert).ToList();
        var ifaces = t.Interfaces.Select(Ref).ToList();

        if (t.TypeKind == TypeKind.Enum)
        {
            var members = t.GetMembers().OfType<IFieldSymbol>()
                .Where(f => f.HasConstantValue)
                .Select(f => new EnumMember(f.Name, f.ConstantValue?.ToString() ?? "", Attributes(f)))
                .ToList();
            return new EnumDecl(name, full, ns, acc, decls, attrs, props, methods, ifaces,
                Ref(t.EnumUnderlyingType!), members);
        }
        if (t.TypeKind == TypeKind.Interface)
            return new InterfaceDecl(name, full, ns, acc, decls, attrs, props, methods, ifaces);

        if (t.IsRecord)
            return new RecordDecl(name, full, ns, acc, decls, attrs, props, methods, ifaces,
                PositionalParameters(t), t.TypeKind == TypeKind.Struct, t.IsSealed, BaseOf(t));

        if (t.TypeKind == TypeKind.Struct)
            return new StructDecl(name, full, ns, acc, decls, attrs, props, methods, ifaces,
                t.IsReadOnly, t.IsRefLikeType);

        if (t.TypeKind == TypeKind.Class)
            return new ClassDecl(name, full, ns, acc, decls, attrs, props, methods, ifaces,
                t.IsAbstract, t.IsSealed, t.IsStatic, decls.Count > 1, BaseOf(t));

        // Delegates and everything else: not modelled yet, and omitted rather
        // than flattened into a class, which would be a lie of the exact kind
        // this schema exists to avoid.
        return null;
    }

    private TypeRef? BaseOf(INamedTypeSymbol t) =>
        t.BaseType is { SpecialType: not SpecialType.System_Object } b ? Ref(b) : null;

    /// <summary>A record's primary constructor parameters, in order.</summary>
    private IReadOnlyList<Parameter> PositionalParameters(INamedTypeSymbol t)
    {
        var primary = t.InstanceConstructors
            .FirstOrDefault(c => c.Parameters.Length > 0
                && c.DeclaringSyntaxReferences.Any(r =>
                    r.GetSyntax() is Microsoft.CodeAnalysis.CSharp.Syntax.RecordDeclarationSyntax));
        return primary is null
            ? []
            : primary.Parameters.Select(Convert).ToList();
    }

    private Property Convert(IPropertySymbol p) => new(
        p.Name,
        Ref(p.Type),
        Map(p.DeclaredAccessibility),
        p.IsStatic,
        p.IsRequired,
        p.SetMethod?.IsInitOnly ?? false,
        p.GetMethod is not null,
        p.SetMethod is not null,
        Attributes(p),
        p.Locations.FirstOrDefault(l => l.IsInSource) is { } loc ? SpanOf(loc) : null);

    private Method Convert(IMethodSymbol m) => new(
        m.Name,
        Ref(m.ReturnType),
        m.Parameters.Select(Convert).ToList(),
        Map(m.DeclaredAccessibility),
        m.IsStatic,
        m.IsAsync,
        Attributes(m),
        m.Locations.FirstOrDefault(l => l.IsInSource) is { } loc ? SpanOf(loc) : null);

    private Parameter Convert(IParameterSymbol p) => new(
        p.Name,
        Ref(p.Type),
        p.HasExplicitDefaultValue,
        p.HasExplicitDefaultValue ? p.ExplicitDefaultValue?.ToString() : null);

    private static IReadOnlyList<AppliedAttribute> Attributes(ISymbol s) =>
        s.GetAttributes()
            .Select(a => new AppliedAttribute(
                a.AttributeClass?.Name ?? "",
                a.AttributeClass?.ToDisplayString() ?? "",
                a.ConstructorArguments.Select(v => v.ToCSharpString()).ToList()))
            .ToList();

    private TypeRef Ref(ITypeSymbol t)
    {
        var nullable = t.NullableAnnotation == NullableAnnotation.Annotated;
        var bare = nullable && t is INamedTypeSymbol { IsGenericType: true, Name: "Nullable" } n
            ? n.TypeArguments[0]
            : t;

        var args = bare is INamedTypeSymbol { IsGenericType: true } g
            ? g.TypeArguments.Select(Ref).ToList()
            : [];

        // Collection-ness is answered here rather than by the caller matching
        // names, so a rule like "a list response maps each element" is written
        // once and holds for IReadOnlyList, List, IEnumerable and arrays alike.
        TypeRef? element = bare switch
        {
            IArrayTypeSymbol arr => Ref(arr.ElementType),
            INamedTypeSymbol { IsGenericType: true } c when IsCollection(c) => Ref(c.TypeArguments[0]),
            _ => null,
        };

        return new TypeRef(
            bare.Name,
            bare.ToDisplayString(NullableFlowState.None, SymbolDisplayFormat.FullyQualifiedFormat)
                .Replace("global::", ""),
            nullable,
            bare.IsValueType,
            args,
            element);
    }

    private static bool IsCollection(INamedTypeSymbol t) =>
        t.AllInterfaces.Any(i => i.Name == "IEnumerable" && i.IsGenericType)
        || t.Name is "IEnumerable" or "IReadOnlyList" or "IReadOnlyCollection" or "List" or "ICollection" or "IList";

    private static Span? SpanOf(Location loc)
    {
        if (!loc.IsInSource) return null;
        var s = loc.GetLineSpan();
        return new Span(s.Path, s.StartLinePosition.Line + 1, s.StartLinePosition.Character + 1,
            s.EndLinePosition.Line + 1, s.EndLinePosition.Character + 1);
    }

    private static Accessibility Map(Acc a) => a switch
    {
        Acc.Private => Accessibility.Private,
        Acc.ProtectedAndInternal => Accessibility.ProtectedAndInternal,
        Acc.Protected => Accessibility.Protected,
        Acc.Internal => Accessibility.Internal,
        Acc.ProtectedOrInternal => Accessibility.ProtectedOrInternal,
        Acc.Public => Accessibility.Public,
        _ => Accessibility.Unknown,
    };
}
