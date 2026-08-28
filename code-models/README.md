# Code model servers

The second half of a language's **Gold** rung (`hick lang`,
`docs/specs/freeform/language-tiers.md`). A code model server answers
questions about source code in that language's own terms, over GraphQL, so a
script can *generate* against your code instead of parsing it.

One server per language, spawned like a language server or an indexer. The
process boundary does two jobs: each model can only be built on its own
ecosystem's compiler front end — Roslyn, the TypeScript compiler, `go/types` —
and none of those is a Rust crate; and it is a licence firewall, since
libclang, Sorbet and the Kotlin Analysis API carry their own terms and none of
them ever links into this product.

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

## What is shared, measured across three languages

The plan said the shared core would be small. It is smaller than that, and
the number is worth writing down because it is the whole argument for
per-language schemas.

Introspecting all three servers and diffing their type sets: **exactly three
types are identical everywhere.**

| shared by all three, identically | `ServerInfo`, `Span`, `UnresolvedReference` |
|---|---|
| same name, different fields | `TypeRef`, `Method`, `InterfaceDecl`, `Query` |
| two of three, different fields | `ClassDecl`, `EnumDecl`, `Property`, `Parameter`, `StructDecl` |
| one language only | `RecordDecl`, `AppliedAttribute` · `TypeAliasDecl`, `Decorator` · `NamedDecl`, `StructTag`, `Field`, `Implementor` |

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
* **`Accessibility`** is identical in C# and TypeScript and **absent from Go**,
  which has no accessibility keyword at all — only spelling. That single
  omission is the clearest evidence the C# enum was never universal.

So the core is **the protocol and the rules, not the schema**. Three types,
about a dozen fields.

## References, and the rule they exist for

Every server answers `references(symbol)`, returning each use with **who
used it** — a declaration name — and whether the use is a write.

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

| language | server | built on | needs at runtime |
|---|---|---|---|
| C# | `csharp/` | Roslyn (`Microsoft.CodeAnalysis.CSharp`) | the .NET runtime |
| TypeScript | `typescript/` | ts-morph over the TS compiler API | `node` |
| Go | `go/` | `go/packages` + `go/types` | `go` on PATH |

That last column is part of the contract: a model server may shell out to its
own toolchain, so a document using one declares both —
`<hick:needs bin="hick-model-go" />` and `<hick:needs bin="go" />`.

Building them:

```
cd csharp     && dotnet publish -c Release -o <somewhere>
cd typescript && npm install                     # then hick-model-typescript
cd go         && go build -o hick-model-go ./...
```

Put the result on `PATH` or in the project's `.hick-cache/models/`. There is
no installer yet — `hick code-model` finds it, and `hick lang` reports the
language as Gold once it does.
