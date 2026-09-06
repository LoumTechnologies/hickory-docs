# Code model servers

The second half of a language's **Gold** rung (`hick lang`,
`docs/specs/freeform/language-tiers.md`). A code model server answers
questions about source code in that language's own terms, over GraphQL, so a
script can *generate* against your code instead of parsing it.

One server per language, spawned like a language server or an indexer. The
process boundary does two jobs: each model can only be built on its own
ecosystem's front end — Roslyn, the TypeScript compiler, `go/types`, `syn` —
and only one of those is a Rust crate; and it is a licence firewall, since
libclang, Sorbet and the Kotlin Analysis API carry their own terms and none of
them ever links into this product.

Rust is the interesting case for that boundary, because `syn` *is* a crate and
could have been linked. It is still a separate process, and the reason is the
rest of the dependency graph rather than the parser: a GraphQL server, an
executor and `cargo metadata` are the price of the model, and none of them is
a price this product should pay in its own binary for a feature four
languages implement four ways.

## The protocol

One JSON object per line on stdin, one per line on stdout — the same framing
`hick mcp` uses, because a second framing in one product is a second thing to
get wrong.

```
→ {"query": "{ types(nameEndsWith: \"Service\") { name } }"}
← {"data":{"types":[{"name":"InventoryService"}]}}
```

Standard GraphQL introspection is supported and is how the schema documents
itself. `hick code-model <language> <source>` with no `--query` prints it.

Diagnostics go to **stderr**; stdout carries the protocol and nothing else.

## The rules every server follows

**Nothing is lowered.** A record is not a class with a flag, an enum is not a
type with constants, and a partial class is one type whose `declarations` list
is plural. If a caller has to reconstruct a distinction the language makes,
the schema has failed. This is the thing an index format flattens and the
reason a code model is not a second SCIP.

**No universal schema.** Each language ships its own. The shared core is
`File`, `Span`, `SymbolId` and a version; everything else is where languages
genuinely differ.

**Read only.** Writes need ordering, atomicity and formatting, all of which
GraphQL handles badly, and a document already owns its bytes through
`hick:file` where lineage and the drift gate work.

**Nothing the compiler invented.** A record's synthesised `EqualityContract`
is the compiler's business. A server that reports it puts it in somebody's
DTO — which happened, and is why `csharp/Compilation.cs` filters
`IsImplicitlyDeclared`.

**Say what you could not resolve.** A generator that ran against a half-bound
compilation and believed it saw everything emits confidently wrong code. The
`unresolved` field is not optional politeness.

## What is shared, measured across four languages

The plan said the shared core would be small. It is smaller than that, and
the number is worth writing down because it is the whole argument for
per-language schemas. **A fourth language did not add to it.**

Introspecting all four servers and diffing their type sets: **exactly three
types are identical everywhere**, the same three as when there were three
servers.

| shared by all four, identically | `ServerInfo`, `Span`, `UnresolvedReference` |
|---|---|
| same name in all four, different fields | `TypeRef`, `Query` |
| three of four, different fields | `Method`, `InterfaceDecl`, `Reference` · `StructDecl`, `EnumDecl` |
| two of four, different fields | `ClassDecl`, `Property`, `Parameter`, `Accessibility`, `EnumMember`, `TypeDecl` · `TypeAliasDecl` · `Field`, `Param` |
| one language only | 8 (C#) · 4 (TypeScript) · 5 (Go) · **21 (Rust)** |

Rust owning twenty-one types of its own is not verbosity. Nine of them are
the item kinds behind one interface, and the rest are things no other
language in this list has: `Visibility` is an object because `pub(in
crate::api)` carries a path; `Generics` and `GenericParam` exist because a
`where` clause is text a generator has to emit back; `ImplBlock` exists
because Rust has a declaration site that belongs to neither the type nor the
trait, and no other schema here has anywhere to put it.

The near-misses are the interesting part, because hoisting any of them would
have been actively wrong:

* **`TypeRef`** has three disjoint field sets. C# asks `isNullable` and
  `isValueType`; TypeScript asks `isUnion`, `isLiteral` and `members`; Go asks
  `isPointer`, `isSlice`, `isMap` and `key`. A common `TypeRef` would be the
  union of all of them with two thirds null at any moment.
* **`Method`** looks shared and is not: Go returns **several** values, so it
  has `results`, and its `pointerReceiver` decides the method set. A single
  `returnType` cannot hold a Go signature.
* **`StructDecl`** exists in C# and Go and means different things — a C#
  struct is a value type with properties; a Go struct is a field list with
  tags and embedding. Same word, different concept, and the trap a universal
  schema walks straight into.
* **`Accessibility`** is identical in C# and TypeScript, **absent from Go**,
  which has no accessibility keyword at all — only spelling — and **an object
  in Rust**, where `pub(in crate::api)` carries a path an enum would throw
  away. Three answers to one question, from four languages.
* **`StructDecl`** is now in three servers and means three things: a C# value
  type with properties, a Go field list with tags and embedding, and a Rust
  product type that is `NAMED`, `TUPLE` or `UNIT` and whose tuple fields are
  named by their index. The trap a universal schema walks straight into, and
  it got worse rather than better with a fourth language.
* **`EnumDecl`** is in three and is a different construct in one of them: a
  Rust enum variant **carries data**, so `EnumVariant` has `fields` and a
  `shape`, and the C# and TypeScript versions are lists of names.

So the core is **the protocol and the rules, not the schema**. Three types,
about a dozen fields.

## References, and the rule they exist for

**Three servers of four** answer `references(symbol)`, returning each use with
**who used it** — a declaration name — and whether the use is a write. Rust
does not, and that is the rule being kept rather than a corner cut: see below.

The referrer is the whole point. A generator makes a meta-pattern official,
and a meta-pattern worth stating has exceptions worth stating:

> The API layer holds no business validation — **except** for fields the
> persistence layer also writes.

Both halves are checkable, and only the second needs references. Written as a
rule in the generator, the exception cannot drift from the code the way a
hand-maintained allow-list beside it would. `isWrite` matters for the same
reason: "nothing outside the domain may SET this" is a different rule from
"nothing may read it", and a location alone cannot tell them apart.

A note on cost, because the first version of this file claimed references
would mean re-implementing an indexer. It does not. Go's `Info.Uses` is
already an identifier-to-object map; TypeScript's checker answers directly;
only C# needed an index built by hand, at 2.3 s once and 0.3 ms per query
after — the right way round for a generator that asks hundreds of times in one
pass.

## What is here

| language | server | built on | needs at runtime | `references` |
|---|---|---|---|---|
| C# | `csharp/` | Roslyn (`Microsoft.CodeAnalysis.CSharp`) | the .NET runtime | yes |
| TypeScript | `typescript/` | ts-morph over the TS compiler API | `node` | yes |
| Go | `go/` | `go/packages` + `go/types` | `go` on PATH | yes |
| Rust | `rust/` | `syn` + `cargo metadata` | `cargo` on PATH | no — see above |

That last column is part of the contract: a model server may shell out to its
own toolchain, so a document using one declares both —
`<hick:needs bin="hick-model-go" />` and `<hick:needs bin="go" />`.

Building them:

```
hick code-model rust --install          # builds code-models/rust, installs it
hick code-model go   --install          # same, with `go build`
cd csharp     && dotnet publish -c Release -o <somewhere>
cd typescript && npm install                     # then hick-model-typescript
```

`--install` **builds** rather than fetches, because there is nowhere to
publish four binaries in four ecosystems to; it needs a checkout of this
repository, and `--from <path>` says which one. It offers only the two
servers that produce a single self-contained binary. C# and TypeScript each
publish a directory whose parts have to stay together — a `dotnet publish`
output is an apphost beside its assemblies, and the TypeScript launcher
resolves `server.js` relative to its own real path — so installing one file
out of either would produce a server that starts and then fails, which is
worse than not offering the command.

Put the result on `PATH` or in the project's `.hick-cache/models/`.
`hick code-model` finds it there, and `hick lang` reports the language as Gold
once it does — measured, never declared.
