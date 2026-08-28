// The C# code model, as Hickory's Gold rung defines it: the language's own
// type system, kept intact.
//
// The rule the whole schema follows is that nothing is lowered. A record is
// not a class with a flag; an enum is not a type with constants; a partial
// class is one type with several declarations, and the plural is visible. If
// a caller has to reconstruct a distinction C# makes, this file has failed.
//
// Read only. CodeIO writes as well as reads and this surface must not: writes
// need ordering, atomicity and formatting, and Hickory already has a better
// answer in `hick:file`, where lineage and the drift gate work.

using Microsoft.CodeAnalysis;

namespace Hick.Model.CSharp;

public enum Accessibility
{
    Private,
    ProtectedAndInternal,
    Protected,
    Internal,
    ProtectedOrInternal,
    Public,
    Unknown,
}

/// <summary>Where something is written, in the file a document wove.</summary>
public sealed record Span(string File, int StartLine, int StartColumn, int EndLine, int EndColumn);

/// <summary>An attribute as applied, not as declared.</summary>
public sealed record AppliedAttribute(string Name, string FullName, IReadOnlyList<string> Arguments);

/// <summary>
/// A reference to a type at a use site. Deliberately not a TypeDecl: the type
/// of a property may be <c>int</c>, or from another assembly, or constructed
/// from generic arguments, and none of those has a declaration in this
/// project to point at.
/// </summary>
public sealed record TypeRef(
    string Name,
    string FullName,
    bool IsNullable,
    bool IsValueType,
    IReadOnlyList<TypeRef> TypeArguments,
    // Non-null when this is a collection of something: the element type. A
    // generator asking "does this endpoint return a list" should not have to
    // pattern-match names.
    TypeRef? ElementType);

public sealed record Parameter(string Name, TypeRef Type, bool HasDefault, string? DefaultValue);

public sealed record Property(
    string Name,
    TypeRef Type,
    Accessibility Accessibility,
    bool IsStatic,
    bool IsRequired,
    bool IsInitOnly,
    bool HasGetter,
    bool HasSetter,
    IReadOnlyList<AppliedAttribute> Attributes,
    Span? Declaration);

public sealed record Method(
    string Name,
    TypeRef ReturnType,
    IReadOnlyList<Parameter> Parameters,
    Accessibility Accessibility,
    bool IsStatic,
    bool IsAsync,
    IReadOnlyList<AppliedAttribute> Attributes,
    Span? Declaration);

public sealed record EnumMember(string Name, string Value, IReadOnlyList<AppliedAttribute> Attributes);

/// <summary>
/// What every declared type has. The concrete kinds add what only they have —
/// which is the point, and the thing an index format flattens away.
/// </summary>
public interface ITypeDecl
{
    string Name { get; }
    string FullName { get; }
    string Namespace { get; }
    Accessibility Accessibility { get; }
    /// <summary>Every place this type is declared. Plural because C# has
    /// partial types, and combining them is this model's job rather than the
    /// caller's.</summary>
    IReadOnlyList<Span> Declarations { get; }
    IReadOnlyList<AppliedAttribute> Attributes { get; }
    IReadOnlyList<Property> Properties { get; }
    IReadOnlyList<Method> Methods { get; }
    IReadOnlyList<TypeRef> Interfaces { get; }
}

public sealed record ClassDecl(
    string Name, string FullName, string Namespace, Accessibility Accessibility,
    IReadOnlyList<Span> Declarations, IReadOnlyList<AppliedAttribute> Attributes,
    IReadOnlyList<Property> Properties, IReadOnlyList<Method> Methods,
    IReadOnlyList<TypeRef> Interfaces,
    bool IsAbstract, bool IsSealed, bool IsStatic, bool IsPartial, TypeRef? BaseType) : ITypeDecl;

public sealed record RecordDecl(
    string Name, string FullName, string Namespace, Accessibility Accessibility,
    IReadOnlyList<Span> Declarations, IReadOnlyList<AppliedAttribute> Attributes,
    IReadOnlyList<Property> Properties, IReadOnlyList<Method> Methods,
    IReadOnlyList<TypeRef> Interfaces,
    // The positional parameters of a primary constructor, in order. This is
    // the thing that makes a record a record, and it is the first casualty of
    // a universal model.
    IReadOnlyList<Parameter> PositionalParameters,
    bool IsStruct, bool IsSealed, TypeRef? BaseType) : ITypeDecl;

public sealed record StructDecl(
    string Name, string FullName, string Namespace, Accessibility Accessibility,
    IReadOnlyList<Span> Declarations, IReadOnlyList<AppliedAttribute> Attributes,
    IReadOnlyList<Property> Properties, IReadOnlyList<Method> Methods,
    IReadOnlyList<TypeRef> Interfaces,
    bool IsReadOnly, bool IsRefLike) : ITypeDecl;

public sealed record InterfaceDecl(
    string Name, string FullName, string Namespace, Accessibility Accessibility,
    IReadOnlyList<Span> Declarations, IReadOnlyList<AppliedAttribute> Attributes,
    IReadOnlyList<Property> Properties, IReadOnlyList<Method> Methods,
    IReadOnlyList<TypeRef> Interfaces) : ITypeDecl;

public sealed record EnumDecl(
    string Name, string FullName, string Namespace, Accessibility Accessibility,
    IReadOnlyList<Span> Declarations, IReadOnlyList<AppliedAttribute> Attributes,
    IReadOnlyList<Property> Properties, IReadOnlyList<Method> Methods,
    IReadOnlyList<TypeRef> Interfaces,
    TypeRef UnderlyingType, IReadOnlyList<EnumMember> Members) : ITypeDecl;

public sealed record ServerInfo(string Name, string Version, string Language, string Root, int FileCount);

/// <summary>
/// One use of a symbol, and — the part that matters — WHO used it.
/// </summary>
/// <remarks>
/// A bare file and line cannot answer "is this read by the persistence
/// layer". The referring declaration can, and that is the form a generator's
/// exceptions are actually written in: a meta-pattern worth stating ("the API
/// layer holds no business validation") almost always has exceptions worth
/// stating too ("except fields the audit layer reads"), and an exception
/// phrased in terms of use sites is a references query.
/// </remarks>
public sealed record Reference(Span Span, string FromDeclaration, string FromFile, bool IsWrite);

/// <summary>A diagnostic the model could not resolve past, reported rather
/// than swallowed: a generator must never run against a half-bound
/// compilation and believe it saw everything.</summary>
public sealed record UnresolvedReference(string Message, Span? At);
