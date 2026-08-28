# Gold, Silver, Bronze: how much Hickory knows about a language

*Status: adopted 2026-08-28. Phase 0 (the ladder and `hick lang`) is built.
The code model it names is designed here and not yet built.*

A document holds code in whatever language its author writes. What Hickory
can *do* with that code varies enormously, and until now there was no way to
say how much — not for a user deciding whether to write their project here,
and not for us deciding what to build next.

Three rungs, each containing the one below.

**Bronze — the text is right.** The extension routes to a language id, so the
bytes in a `hick:file` are more than bytes: they are highlighted, they reach a
language server if one turns up, and a cell can run the toolchain. This should
be free for any language with a grammar.

**Silver — the editor is right.** A language server and a debug adapter are
reachable, so a *generated* file has completions and diagnostics, and can be
stepped through in **document coordinates** — the line you edited, not the
line the weave produced.

**Gold — the code is data.** An index answers questions about the whole
project, and a **code model server** answers questions about the language's
own type system, so a script can *generate* against it. This is the rung that
makes `owning-what-a-scaffolder-wrote.md` and the repetitious-layer problem
tractable: you review the generator, not its output.

## A tier is measured, never declared

This is the load-bearing rule, and it is written down because the alternative
has now failed in this repository **four times**, identically each time: a
hand-maintained list kept beside the thing it describes.

1. `lang_detect` had no `cs` row, so `hick lsp install csharp` fetched a
   server that could never be reached.
2. `hick init`'s reported-languages list omitted C#, so a C# project was told
   about Go and YAML.
3. `hick_lsp::discovery::known_languages` omitted `javascriptreact` and
   `typescriptreact` while `candidates` served them, so React files were
   reported as having no language server on machines where one was running.
4. `hick_dap::discovery::known_languages` omitted the same two, so a `.tsx`
   file was called undebuggable with its adapter installed.

None was caught by a test, because in each case the list was the only
statement of its own contents. So `hick lang` **computes** every column from
the catalogue or discovery path that would actually serve the request, and the
tests derive their expectations from those same sources. There is no fifth
table.

The generalisation of "never claim a language is debuggable before it is"
(`launching-what-a-document-builds.md`) is: never claim a rung before it holds,
and make the claim by measurement so it cannot be claimed by accident.

## The exemption, and why it is not a fudge

A data or markup language — JSON, YAML, TOML, Markdown, HTML, CSS, SQL — has
nothing to step through and no source-level type model to generate from.
Asking it for a debugger is not a gap; it is the wrong question, and advice to
"install a debugger for YAML" is advice nobody can take.

So those languages are marked, their debugger/index/model columns read `n/a`,
and **Silver is the top of their ladder** rather than a shortfall. They are
not rounded up to Gold on three `n/a`s, because that would claim a code model
that cannot exist.

SQL is the interesting one: it *does* have a rich model, but the model is the
live database schema rather than anything in the source. That is a different
feature and should get a different name if it is ever built.

## The code model (Gold's second half)

Not built. The shape it should take:

**A spawned server, one per language**, installed the way language servers and
indexers already are, into the same confined prefix. The process boundary is
doing two jobs: it keeps each language's model in the ecosystem that can
maintain it, and it is a licence firewall — libclang, Sorbet and the Kotlin
Analysis API each carry their own terms and none of them ever links into this
product.

**GraphQL, over a local socket.** Code is a graph; selection sets are laziness,
which matters because semantic resolution is the expensive part and a
generator asking for property names must not pay for method bodies. Interfaces
and unions express a language's own type system exactly — a record is not a
class with a flag — which is precisely what SCIP flattens.

**A schema per language, and no universal model.** This is a feature, and
after three servers the size of the shared core is known rather than
estimated: **three types** — `ServerInfo`, `Span`, `UnresolvedReference` —
about a dozen fields. Everything else that shares a *name* across two servers
has different *fields*, and the near-misses are the ones that would have hurt
most to hoist: Go's `Method` returns several values, Go's `StructDecl` means
something else than C#'s, and Go has no `Accessibility` at all because
exportedness is spelling. See `code-models/README.md` for the diff. Introspection is the documentation, which
is the property that lets a *model* write a generator you can review: it can
ask what a property exposes instead of guessing at an API it half-remembers.

**Read only.** CodeIO writes as well as reads; this surface must not. Writes
need ordering, atomicity and formatting, all of which GraphQL handles badly,
and Hickory already has a better answer — the script emits text and the
document owns the bytes, where lineage and the drift gate already work.

### How a cell reaches it: a tool, not a service

*Amended 2026-08-28, after building it.* The first design here was a socket,
and the second was "hick runs one declared query before the cell and drops the
answer in as a file". Both are wrong, and the second is wrong for a reason
worth recording: **real queries are parametric.** A generator asks about a
type it just found, then about that type's base, then about what implements an
interface. One pre-run query cannot serve that, and a document listing every
question it might ask is not a document anyone would write.

The answer removes the problem instead of solving it. **The model server is a
tool the cell runs, like `python3` or `dotnet` — not a service Hickory
brokers.** Hickory binds `.hick-cache/models` into the sandbox read-only and
puts it on the cell's `PATH`; the cell spawns the server itself and talks to
it over an ordinary pipe, for as many queries as it likes.

Everything that looked expensive to retrofit then costs nothing:

* **The cache key is already right.** The server answers about the source
  *mounted into the cell*, because the sandbox hides everything else — and a
  volume's contents are already part of `input_digest`. Change the domain and
  the cell re-executes, verified with `--cache` against a modified domain.
  There is no second mechanism, and nothing had to learn what a model is.
* **Replay is already right.** A weave replays the cell's recording, so it
  reproduces without the model server for exactly the same reason it
  reproduces without `dotnet`. Nothing new is recorded because nothing new
  needs to be.
* **The sandbox is not weakened.** Verified in a cell: the model binary is
  runnable, `~/.bashrc` is not readable, and unmounted project source is not
  visible.

The only new surface is `<hick:needs bin="hick-model-csharp" />`, which
already existed and now means what it says.

## What this does not replace

*Settled 2026-08-28. Then reopened the same day, because the first settlement
rested on a claim that was false.*

### The claim that was wrong

The first version of this section argued that SCIP stays because **a code
model is about what is DECLARED and an index is about where it is USED**, and
offered as evidence that none of the three servers had grown a references
query. That evidence was circular: none had one because nobody had written
one, and I concluded from its absence that it did not belong.

Nate pointed out what it is for. A generator exists to make a **meta-pattern**
official — "the API layer holds no business validation", "the domain layer
holds no formatting" — and a meta-pattern worth stating almost always has
exceptions worth stating too: *except for fields the persistence layer
reads*. An exception phrased in terms of use sites **is** a references query.
Without one, the exceptions become a hand-maintained list beside the rule,
which is the shape of every bug in this repository's history.

### It was also much less work than claimed

"Re-implementing an indexer" was wrong in all three languages:

* **Go** — `packages.NeedTypesInfo` already fills `Info.Uses`, an
  identifier-to-object map for every file loaded. The index was computed
  before anyone asked; grouping it is the implementation.
* **TypeScript** — `findReferencesAsNodes` is the checker's own answer, so a
  same-named property on an unrelated object is not a reference and one
  reached through a re-export is.
* **C#** — the only one needing real work, and not much. `SymbolFinder` wants
  a `Solution` and therefore MSBuild, which is too large a dependency; asking
  the semantic model once per identifier and grouping gives the same map. Two
  point three seconds to build, **0.3 ms marginal per query** thereafter,
  which is the right way round for a generator that asks hundreds of times in
  one pass.

All three now answer `references(symbol)` with the thing that makes it usable:
**who referred**, as a declaration name, plus whether the use is a write. A
bare file and line cannot answer "is this written by the API layer"; a
`fromDeclaration` can.

### So where does that leave SCIP

**Still here, and for a narrower reason than before.** The honest line is not
declarations-versus-uses — that was an artifact of not having built this. It
is:

> **A code model is per-language and live. An index is cross-language and
> precomputed.**

Which means, concretely:

* **For generation, SCIP now adds nothing.** One language, one project, one
  pass, and the interesting queries join declaration facts to use facts —
  "properties marked `[NeverExpose]` that the audit layer nonetheless reads" —
  which is one conversation with a model and an awkward join against an index.
* **For navigation, SCIP still wins**, on three counts the model cannot
  answer at any price: a document that weaves C# *and* TypeScript has one
  reference graph and three servers cannot join it; an index is a file that
  can be published and read without a compile; and fourteen Silver languages
  have an indexer available and no model server, nor will they soon.

The one thing that would close it is a model server for every language a
project uses *plus* something to join across them — at which point the
question is whether that join is worth maintaining, not whether the two
overlap.

| | answers | rule |
|---|---|---|
| language server | this buffer, right now | the live authority; wins any disagreement |
| index | where a symbol is used, across languages | *a cache, never a record* |
| code model | what is declared AND where it is used, in one language's own terms, live | an **input to generation**, checked by the drift gate |

The rules are unchanged; only the middle column of the last row has grown, and
only the reason for keeping the middle row has moved.
