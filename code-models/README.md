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

## What is here

| language | server | built on |
|---|---|---|
| C# | `csharp/` | Roslyn (`Microsoft.CodeAnalysis.CSharp`) |

Building it: `cd csharp && dotnet publish -c Release -o <somewhere>`, then put
`hick-model-csharp` on `PATH` or in the project's `.hick-cache/models/`.
There is no installer yet — `hick code-model` finds it, and `hick lang`
reports the language as Gold once it does.
