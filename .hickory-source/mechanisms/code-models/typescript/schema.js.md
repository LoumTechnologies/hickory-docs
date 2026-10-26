<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain code-models/typescript/schema.js

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a4887ceb71e4"></a>
## `code-models/typescript/schema.js`

Expose compiler-backed TypeScript structure to read-only generators; language semantics and unresolved references remain explicit.

<a id="f-a4887ceb71e4-NonNullList"></a>
### `NonNullList`

The TypeScript code model.
The same three rules every model server follows — nothing lowered, nothing
the compiler invented, say what you could not resolve — applied to a type
system that disagrees with C#'s in almost every particular. Writing this
second is the point: it is where the C# schema's accidents show up.
What TypeScript has that C# does not, and which this schema therefore has:
* TYPE ALIASES are declarations. `type Id = string` is a thing you can
find, not sugar to be resolved away.
* UNIONS AND INTERSECTIONS are types, not a pattern over inheritance.
* OPTIONAL is not NULLABLE. `name?: string` and `name: string | undefined`
differ — the first may be absent from the object, the second must be
present and may hold undefined. Collapsing them into one `isNullable`
flag, which is what a C#-shaped schema would do, loses the distinction
a generator needs to decide whether a field is required on the wire.
* INTERFACES ARE STRUCTURAL. Nothing declares that it implements one, so
`implementedBy` is a question the model must answer by checking
assignability rather than by reading a declaration list.
* DECORATORS are not attributes: they are expressions, and they carry
arguments that may be anything.

<storycode:copy id="f-a4887ceb71e4-NonNullList">
// The TypeScript code model.
//
// The same three rules every model server follows — nothing lowered, nothing
// the compiler invented, say what you could not resolve — applied to a type
// system that disagrees with C#'s in almost every particular. Writing this
// second is the point: it is where the C# schema's accidents show up.
//
// What TypeScript has that C# does not, and which this schema therefore has:
//
//   * TYPE ALIASES are declarations. `type Id = string` is a thing you can
//     find, not sugar to be resolved away.
//   * UNIONS AND INTERSECTIONS are types, not a pattern over inheritance.
//   * OPTIONAL is not NULLABLE. `name?: string` and `name: string | undefined`
//     differ — the first may be absent from the object, the second must be
//     present and may hold undefined. Collapsing them into one `isNullable`
//     flag, which is what a C#-shaped schema would do, loses the distinction
//     a generator needs to decide whether a field is required on the wire.
//   * INTERFACES ARE STRUCTURAL. Nothing declares that it implements one, so
//     `implementedBy` is a question the model must answer by checking
//     assignability rather than by reading a declaration list.
//   * DECORATORS are not attributes: they are expressions, and they carry
//     arguments that may be anything.

import {
  GraphQLBoolean, GraphQLEnumType, GraphQLInt, GraphQLInterfaceType, GraphQLList,
  GraphQLNonNull, GraphQLObjectType, GraphQLSchema, GraphQLString,
} from "graphql";

const NonNullList = (t) => new GraphQLNonNull(new GraphQLList(new GraphQLNonNull(t)));

</storycode:copy>
<a id="f-a4887ceb71e4-Span"></a>
### `Span`

The shared core. Three types, and that is genuinely all of it — see
`code-models/README.md`.

<storycode:copy id="f-a4887ceb71e4-Span">
// ---------------------------------------------------------------------------
// The shared core. Three types, and that is genuinely all of it — see
// `code-models/README.md`.
// ---------------------------------------------------------------------------

const Span = new GraphQLObjectType({
  name: "Span",
  description: "Where something is written.",
  fields: {
    file: { type: new GraphQLNonNull(GraphQLString) },
    startLine: { type: new GraphQLNonNull(GraphQLInt) },
    startColumn: { type: new GraphQLNonNull(GraphQLInt) },
    endLine: { type: new GraphQLNonNull(GraphQLInt) },
    endColumn: { type: new GraphQLNonNull(GraphQLInt) },
  },
});

</storycode:copy>
<a id="f-a4887ceb71e4-ServerInfo"></a>
### `ServerInfo`

<storycode:copy id="f-a4887ceb71e4-ServerInfo">
const ServerInfo = new GraphQLObjectType({
  name: "ServerInfo",
  fields: {
    name: { type: new GraphQLNonNull(GraphQLString) },
    version: { type: new GraphQLNonNull(GraphQLString) },
    language: { type: new GraphQLNonNull(GraphQLString) },
    root: { type: new GraphQLNonNull(GraphQLString) },
    fileCount: { type: new GraphQLNonNull(GraphQLInt) },
  },
});

</storycode:copy>
<a id="f-a4887ceb71e4-UnresolvedReference"></a>
### `UnresolvedReference`

<storycode:copy id="f-a4887ceb71e4-UnresolvedReference">
const UnresolvedReference = new GraphQLObjectType({
  name: "UnresolvedReference",
  description:
    "Something the model could not bind. Reported rather than swallowed: a " +
    "generator that ran against a half-resolved program and believed it saw " +
    "everything emits confidently wrong code.",
  fields: {
    message: { type: new GraphQLNonNull(GraphQLString) },
    at: { type: Span },
  },
});

</storycode:copy>
<a id="f-a4887ceb71e4-Accessibility"></a>
### `Accessibility`

Everything below is TypeScript's own, and shares nothing with C#'s.

<storycode:copy id="f-a4887ceb71e4-Accessibility">
// ---------------------------------------------------------------------------
// Everything below is TypeScript's own, and shares nothing with C#'s.
// ---------------------------------------------------------------------------

const Accessibility = new GraphQLEnumType({
  name: "Accessibility",
  description:
    "TypeScript has four, and the fourth is not a keyword: `#name` is a " +
    "JavaScript private field, enforced at runtime, where `private` is erased " +
    "at compile time. A generator emitting a wire shape must not treat them " +
    "alike.",
  values: {
    PUBLIC: {}, PROTECTED: {}, PRIVATE: {}, JS_PRIVATE: {},
  },
});

</storycode:copy>
<a id="f-a4887ceb71e4-Decorator"></a>
### `Decorator`

<storycode:copy id="f-a4887ceb71e4-Decorator">
const Decorator = new GraphQLObjectType({
  name: "Decorator",
  description:
    "Not an attribute. A decorator is an expression, so its arguments are " +
    "source text rather than constant values, and the model says so instead " +
    "of pretending to have evaluated them.",
  fields: {
    name: { type: new GraphQLNonNull(GraphQLString) },
    argumentsText: { type: NonNullList(GraphQLString) },
  },
});

</storycode:copy>
<a id="f-a4887ceb71e4-TypeRef"></a>
### `TypeRef`

<storycode:copy id="f-a4887ceb71e4-TypeRef">
const TypeRef = new GraphQLObjectType({
  name: "TypeRef",
  description: "A type at a use site.",
  fields: () => ({
    text: {
      type: new GraphQLNonNull(GraphQLString),
      description: "As the compiler prints it — the thing to emit.",
    },
    kind: { type: new GraphQLNonNull(GraphQLString) },
    isUnion: { type: new GraphQLNonNull(GraphQLBoolean) },
    isIntersection: { type: new GraphQLNonNull(GraphQLBoolean) },
    // A union's members, which is how `string | undefined` is inspected
    // rather than string-matched.
    members: { type: NonNullList(TypeRef) },
    isArray: { type: new GraphQLNonNull(GraphQLBoolean) },
    elementType: { type: TypeRef },
    isLiteral: {
      type: new GraphQLNonNull(GraphQLBoolean),
      description: "`'a' | 'b'` is two literal types, not a string.",
    },
    literalValue: { type: GraphQLString },
    typeArguments: { type: NonNullList(TypeRef) },
  }),
});

</storycode:copy>
<a id="f-a4887ceb71e4-Property"></a>
### `Property`

<storycode:copy id="f-a4887ceb71e4-Property">
const Property = new GraphQLObjectType({
  name: "Property",
  fields: {
    name: { type: new GraphQLNonNull(GraphQLString) },
    type: { type: new GraphQLNonNull(TypeRef) },
    accessibility: { type: new GraphQLNonNull(Accessibility) },
    isStatic: { type: new GraphQLNonNull(GraphQLBoolean) },
    isReadonly: { type: new GraphQLNonNull(GraphQLBoolean) },
    isOptional: {
      type: new GraphQLNonNull(GraphQLBoolean),
      description:
        "Declared `name?: T` — the property may be ABSENT. Different from a " +
        "type that includes undefined, where the property is present and " +
        "holds it. C#'s single `isNullable` cannot express the difference.",
    },
    decorators: { type: NonNullList(Decorator) },
    declaration: { type: Span },
  },
});

</storycode:copy>
<a id="f-a4887ceb71e4-Parameter"></a>
### `Parameter`

<storycode:copy id="f-a4887ceb71e4-Parameter">
const Parameter = new GraphQLObjectType({
  name: "Parameter",
  fields: {
    name: { type: new GraphQLNonNull(GraphQLString) },
    type: { type: new GraphQLNonNull(TypeRef) },
    isOptional: { type: new GraphQLNonNull(GraphQLBoolean) },
    isRest: { type: new GraphQLNonNull(GraphQLBoolean) },
    hasDefault: { type: new GraphQLNonNull(GraphQLBoolean) },
  },
});

</storycode:copy>
<a id="f-a4887ceb71e4-Method"></a>
### `Method`

<storycode:copy id="f-a4887ceb71e4-Method">
const Method = new GraphQLObjectType({
  name: "Method",
  fields: {
    name: { type: new GraphQLNonNull(GraphQLString) },
    returnType: { type: new GraphQLNonNull(TypeRef) },
    parameters: { type: NonNullList(Parameter) },
    accessibility: { type: new GraphQLNonNull(Accessibility) },
    isStatic: { type: new GraphQLNonNull(GraphQLBoolean) },
    isAsync: { type: new GraphQLNonNull(GraphQLBoolean) },
    decorators: { type: NonNullList(Decorator) },
    declaration: { type: Span },
  },
});

</storycode:copy>
<a id="f-a4887ceb71e4-declFields"></a>
### `declFields`

<storycode:copy id="f-a4887ceb71e4-declFields">
const declFields = () => ({
  name: { type: new GraphQLNonNull(GraphQLString) },
  // No `fullName`: TypeScript has no assembly-qualified name. A declaration
  // is identified by its name and the module it is in, and pretending
  // otherwise would invent a C#-shaped identity TypeScript does not have.
  module: {
    type: new GraphQLNonNull(GraphQLString),
    description: "The file, relative to the root — TypeScript's namespace.",
  },
  isExported: { type: new GraphQLNonNull(GraphQLBoolean) },
  declarations: { type: NonNullList(Span) },
  properties: { type: NonNullList(Property) },
  methods: { type: NonNullList(Method) },
});

</storycode:copy>
<a id="f-a4887ceb71e4-TypeDecl"></a>
### `TypeDecl`

<storycode:copy id="f-a4887ceb71e4-TypeDecl">
const TypeDecl = new GraphQLInterfaceType({
  name: "TypeDecl",
  fields: declFields,
  resolveType: (value) => value.__typename,
});

</storycode:copy>
<a id="f-a4887ceb71e4-ClassDecl"></a>
### `ClassDecl`

<storycode:copy id="f-a4887ceb71e4-ClassDecl">
const ClassDecl = new GraphQLObjectType({
  name: "ClassDecl",
  interfaces: [TypeDecl],
  fields: () => ({
    ...declFields(),
    isAbstract: { type: new GraphQLNonNull(GraphQLBoolean) },
    extendsType: { type: TypeRef },
    // Declared, not structural: a class says `implements` even though the
    // check is structural, and what it SAYS is what a reader relies on.
    implementsTypes: { type: NonNullList(TypeRef) },
    decorators: { type: NonNullList(Decorator) },
  }),
});

</storycode:copy>
<a id="f-a4887ceb71e4-InterfaceDecl"></a>
### `InterfaceDecl`

<storycode:copy id="f-a4887ceb71e4-InterfaceDecl">
const InterfaceDecl = new GraphQLObjectType({
  name: "InterfaceDecl",
  description:
    "Structural. Nothing declares that it satisfies this, so a generator " +
    "asking 'what implements it' is asking an assignability question rather " +
    "than reading a list — see `implementedBy` on the root query.",
  interfaces: [TypeDecl],
  fields: () => ({
    ...declFields(),
    extendsTypes: { type: NonNullList(TypeRef) },
  }),
});

</storycode:copy>
<a id="f-a4887ceb71e4-TypeAliasDecl"></a>
### `TypeAliasDecl`

<storycode:copy id="f-a4887ceb71e4-TypeAliasDecl">
const TypeAliasDecl = new GraphQLObjectType({
  name: "TypeAliasDecl",
  description:
    "`type Id = string`. A declaration in its own right, not sugar: erasing " +
    "it would lose a name the code uses and a generator should emit.",
  interfaces: [TypeDecl],
  fields: () => ({
    ...declFields(),
    aliasedType: { type: new GraphQLNonNull(TypeRef) },
  }),
});

</storycode:copy>
<a id="f-a4887ceb71e4-EnumDecl"></a>
### `EnumDecl`

<storycode:copy id="f-a4887ceb71e4-EnumDecl">
const EnumDecl = new GraphQLObjectType({
  name: "EnumDecl",
  interfaces: [TypeDecl],
  fields: () => ({
    ...declFields(),
    isConst: { type: new GraphQLNonNull(GraphQLBoolean) },
    members: {
      type: NonNullList(
        new GraphQLObjectType({
          name: "EnumMember",
          fields: {
            name: { type: new GraphQLNonNull(GraphQLString) },
            value: { type: GraphQLString },
          },
        }),
      ),
    },
  }),
});

</storycode:copy>
<a id="f-a4887ceb71e4-Reference"></a>
### `Reference`

<storycode:copy id="f-a4887ceb71e4-Reference">
const Reference = new GraphQLObjectType({
  name: "Reference",
  description:
    "One use of a symbol, and WHO used it. The checker's answer, not a text " +
    "search — a same-named property on an unrelated object is not a " +
    "reference, and one reached through a re-export is.",
  fields: {
    span: { type: new GraphQLNonNull(Span) },
    fromModule: { type: new GraphQLNonNull(GraphQLString) },
    fromDeclaration: { type: new GraphQLNonNull(GraphQLString) },
    isWrite: { type: new GraphQLNonNull(GraphQLBoolean) },
  },
});

</storycode:copy>
<a id="f-a4887ceb71e4-buildSchema"></a>
### `buildSchema`

<storycode:copy id="f-a4887ceb71e4-buildSchema">
export function buildSchema(model) {
  return new GraphQLSchema({
    types: [ClassDecl, InterfaceDecl, TypeAliasDecl, EnumDecl],
    query: new GraphQLObjectType({
      name: "Query",
      fields: {
        version: { type: new GraphQLNonNull(ServerInfo), resolve: () => model.info() },
        unresolved: {
          type: NonNullList(UnresolvedReference),
          resolve: () => model.unresolved(),
        },
        types: {
          type: NonNullList(TypeDecl),
          args: {
            moduleIs: { type: GraphQLString },
            nameEndsWith: { type: GraphQLString },
            exportedOnly: { type: GraphQLBoolean },
          },
          resolve: (_r, a) => model.types(a),
        },
        type: {
          type: TypeDecl,
          args: { name: { type: new GraphQLNonNull(GraphQLString) } },
          resolve: (_r, a) => model.types({}).find((t) => t.name === a.name) ?? null,
        },
        references: {
          type: NonNullList(Reference),
          description:
            "Every use of `Type` or `Type.member`. A generator's exceptions " +
            "are written in terms of who uses a thing, so the referring " +
            "declaration is part of the answer rather than something the " +
            "caller reconstructs from a line number.",
          args: { symbol: { type: new GraphQLNonNull(GraphQLString) } },
          resolve: (_r, a) => model.references(a.symbol),
        },
        implementedBy: {
          type: NonNullList(TypeDecl),
          description:
            "Every declaration assignable to this interface. The structural " +
            "answer, which is the only correct one in TypeScript and has no " +
            "counterpart in a language where implementation is declared.",
          args: { interfaceName: { type: new GraphQLNonNull(GraphQLString) } },
          resolve: (_r, a) => model.implementedBy(a.interfaceName),
        },
      },
    }),
  });
}
</storycode:copy>

### Assemble `code-models/typescript/schema.js`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="code-models/typescript/schema.js"><storycode:paste select="#f-a4887ceb71e4-NonNullList"/><storycode:paste select="#f-a4887ceb71e4-Span"/><storycode:paste select="#f-a4887ceb71e4-ServerInfo"/><storycode:paste select="#f-a4887ceb71e4-UnresolvedReference"/><storycode:paste select="#f-a4887ceb71e4-Accessibility"/><storycode:paste select="#f-a4887ceb71e4-Decorator"/><storycode:paste select="#f-a4887ceb71e4-TypeRef"/><storycode:paste select="#f-a4887ceb71e4-Property"/><storycode:paste select="#f-a4887ceb71e4-Parameter"/><storycode:paste select="#f-a4887ceb71e4-Method"/><storycode:paste select="#f-a4887ceb71e4-declFields"/><storycode:paste select="#f-a4887ceb71e4-TypeDecl"/><storycode:paste select="#f-a4887ceb71e4-ClassDecl"/><storycode:paste select="#f-a4887ceb71e4-InterfaceDecl"/><storycode:paste select="#f-a4887ceb71e4-TypeAliasDecl"/><storycode:paste select="#f-a4887ceb71e4-EnumDecl"/><storycode:paste select="#f-a4887ceb71e4-Reference"/><storycode:paste select="#f-a4887ceb71e4-buildSchema"/></storycode:file>

</storycode:doc>
