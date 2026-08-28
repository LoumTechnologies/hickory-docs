# Clients

A generator's first forty lines are always the same: spawn the model server,
frame the JSON, notice that GraphQL reports failure inside a
successful-looking body, and reach into an untyped response. None of it is
specific to a project or a language, and every generator rewrites it.

Measured on the warehouse's API generator, that plumbing was **36% of the
file** and the three rules it existed to serve were **9%**.

| | lines | pays for itself at |
|---|---|---|
| generator, hand-rolled transport | 326 | ~16 endpoints |
| generator, using the client | 202 | ~10 endpoints |
| the client, written once | 189 | — |

## What a client is, and is not

**Is:** the transport, the error check, attribute access over the response,
and a small buffer for building text. Four things, all of them the same in
every language.

**Is not:** a typed client generated from the schema. That would need a code
generator per language — the thing this exists to make unnecessary — and
attribute access gets most of the benefit for none of the machinery. What it
does buy over raw dictionaries is the failure mode: a mistyped field raises
here and returns `None` from a dict, and a generator that silently reads
`None` emits silently wrong code.

## A bug worth keeping

The first version of the Python client made `Node` a fresh wrapper on every
access, so two reads of the same field produced objects that compared
unequal. A generator testing `parameter in route_params` then found nothing
ever matched, and emitted a handler taking `string sku, string sku`. It was
caught by diffing the rewritten generator's output against the previous
generator's, byte for byte — which is the only reason it is not still there,
and the argument for keeping an equivalence gate on anything that generates.

`Node.__eq__` compares the underlying response by identity now, and access is
memoised.

## Typed clients: `--client`

```
hick code-model csharp src/Domain --client queries.graphql --target python -o client.py
```

Types and the query text are generated together, so they cannot drift the way
a hand-written struct beside a hand-written query string does.

Two languages are involved and it is worth keeping them apart: `csharp` is
what is being **modelled**, `--target python` is what the **generator** is
written in.

### Why this is written here rather than delegated

Every ecosystem has a GraphQL codegen, and each is good at what it is for —
which is building an application client. Measured before deciding:

| tool | cost |
|---|---|
| `graphql-codegen` (TS) | 166 npm packages, 72 MB, needs Node |
| `genqlient` (Go) | needs the Go toolchain |
| `ariadne-codegen` (Python) | needs Python and pydantic |
| StrawberryShake (C#) | a reactive client framework with stores and DI |

Four tools, four toolchains, four config formats, and four differently-shaped
clients for a generator author to relearn — which is the per-language expense
this whole design exists to avoid. StrawberryShake is not even the right
shape: it builds application clients, not "spawn a subprocess and ask three
questions".

Against that, the hard part of query codegen is **language-independent**: walk
the query against the schema and work out what the response looks like,
including nullability, lists, and the variants an interface selection can
produce. That is computed once. An emitter is about a hundred lines, so
adding a language is a file rather than a toolchain.

### What the comparison actually showed

Not that ours is better. `graphql-codegen` got something right that the first
version here got wrong, and finding out cost nothing because the comparison
was run: **an interface selection produces every possible type, not only the
ones a fragment named.** Selecting `types { name }` returns `EnumDecl`s
whether or not anyone wrote `... on EnumDecl`. The first resolver listed two
of five, and the TypeScript it generated did not compile.

The second bug came from the third language, as it did with the schemas. The
fields an interface has in common are cloned into each variant, so a nested
shape under them is reached once per variant and kept one name — five
identical declarations. **TypeScript merges identical interfaces silently and
Go refuses to compile**, so the language that could not hide it is the one
that reported it.

Both are fixed, and all three targets now compile: `tsc --strict`, `go build`,
and a Python import.

The lesson is not "roll your own". It is that a generated client must be
compiled to be believed, which is cheap, and that a mature tool is worth
diffing against even when you do not adopt it.

### Limits, refused by name rather than dropped

Named fragments, directives, mutations and subscriptions are not supported. A
generator's queries are selections over a local read-only schema, and a
feature nobody uses is a feature that rots. Anything unsupported fails with a
message naming it, because a silently dropped selection is a field the caller
expects and the response never carries.

## Untyped, and why it is still the default

The client returns responses reachable with dots, not generated types. That is
a decision, not a limitation, and the reasoning is worth keeping:

**The safety property people want from typing is already covered.** The danger
is a generator reading a field that is not there, getting nothing, and
emitting silently wrong code. `Node` raises on an unknown field and lists what
the response does have. Types would catch it earlier; nothing catches it
*later* than a wrong file on disk, which is what dictionaries allow.

**What typing adds beyond that is editor-time feedback**, and that is real —
but a query-typed client is a code generator per language, which is the
expense this whole design exists to avoid, and **every Gold language's
ecosystem already has one**: `genqlient` for Go, `graphql-codegen` for
TypeScript, StrawberryShake for C#.

So Hickory emits SDL and stops:

```
hick code-model csharp src/Domain --sdl > schema.graphql
npx graphql-codegen          # or genqlient, or dotnet graphql
```

Verified end to end: all three servers emit SDL that the reference GraphQL
implementation accepts, and `graphql-codegen` turns the C# schema plus a query
into a discriminated union over `ClassDecl | RecordDecl | StructDecl |
InterfaceDecl | EnumDecl` with nullability intact — Hickory having written no
code generator at all.

The SDL printer lives in `hickory-cli`, once, rather than in each server: it
is the same transformation from the same introspection response everywhere,
which is the shared core doing its job.

## What is here

| language | client | status |
|---|---|---|
| Python | `python/hick_model.py` | in use by the warehouse's generator |
| TypeScript | — | not written; the same four things |
| Go | — | not written; the same four things |

Python first because a generator can be written in any language and Python is
the one most people reach for when the answer is "not the language I am
generating". The other two are about a hundred and ninety lines each, and the
measurement above is the case for writing them.
