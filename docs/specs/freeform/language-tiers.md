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

**A schema per language, and no universal model.** This is a feature. The
shared core is `File`, `Span`, `SymbolId` and a version, and everything else
is where languages genuinely differ. Introspection is the documentation, which
is the property that lets a *model* write a generator you can review: it can
ask what a property exposes instead of guessing at an API it half-remembers.

**Read only.** CodeIO writes as well as reads; this surface must not. Writes
need ordering, atomicity and formatting, all of which GraphQL handles badly,
and Hickory already has a better answer — the script emits text and the
document owns the bytes, where lineage and the drift gate already work.

### The two things that are expensive to retrofit

**The model response is an input, so it belongs in the cache key.** A
generator's inputs stop being only files. Hickory keys a recording on the
cell's inputs; if the model's answer is not in that key, a changed domain will
not invalidate the generated output and `hick test` will pass on stale code.

**A replayed document needs the model's version.** `hick lineage --at` replays
at a commit. For a generated file to stay explicable later, the server's
version must be recorded beside the query — the same reason a build now stamps
the commit it came from.

## What this does not replace

SCIP. An index answers *where else is this used*, uniformly and across
languages, and a per-language model cannot answer across a document that
weaves C# and TypeScript together. The two have different rules already: an
index is *a cache, never a record* (`an-index-beside-the-language-server.md`);
a code model is neither, it is an input to generation whose output the drift
gate checks. Revisit only after three model servers exist, and decide about
the cross-language join rather than about redundancy.
