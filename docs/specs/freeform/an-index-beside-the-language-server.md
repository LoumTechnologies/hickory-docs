# An index beside the language server

*Status: proposal, 2026-08-26. **The install-and-spawn half is built
(2026-08-27)**; the reading half is deliberately not — see the licence
question below, which is still open. `hick index install` and `hick index
build` work, and nothing consults what they produce. Guarantee:
`docs/guarantees/editor-intelligence/an-index-is-installed-and-spawned-and-read-by-nothing.md`.*

A language server answers questions about **the file you are looking at, as it
is right now, including the parts you have not saved**. An index answers
questions about **the whole project, as it was when the index was built**.
JetBrains feels the way it does because it has both — a persistent stub index
under live analysis — and every editor that has only one of them feels like
the half it has.

So: **SCIP is in addition to LSP, never in place of it.** The two are not
substitutes and a plan that treats them as substitutes loses whichever half it
drops.

| | LSP | SCIP |
|---|---|---|
| what it is | a live conversation with a running analyzer | a Protobuf **index format**, produced by an indexer |
| completions, diagnostics, hovers, rename | yes | **none — it has no such concept** |
| your unsaved buffer | yes | no; the index is as old as its last build |
| go-to-definition, find-references | yes, within what the server has loaded | **across the entire project, precomputed** |
| across documents and repositories | no | yes |
| cost | a process per language, always running | a build, occasionally |

SCIP is Apache-2.0. Indexers exist for C# (`scip-dotnet`), TypeScript
(`scip-typescript`), Python (`scip-python`), Rust (rust-analyzer emits it),
and Java, Kotlin, Scala, C, C++, Ruby, Dart and PHP.

## Why this product wants an index more than most editors do

A `.hick` project's code does not live in files a language server can open. It
lives in **documents**, and the files appear when something weaves them. That
has two consequences an index answers and a language server cannot:

1. **The interesting question spans documents.** "Where else is
   `invoice_ref` used" means across every document in the folder, and the
   files those documents weave — most of which no server has open, and some
   of which are not on disk at this moment.
2. **The answer must land in the document, not in the generated file.** A
   reference found in `orders.py` is only useful if it opens the `hick:copy`
   fragment that contributed those bytes. That mapping already exists —
   lineage, and the same `Mapping` the debugger uses to put a breakpoint on a
   woven line back onto a document line.

So an index here is not "go-to-definition, but faster". It is the first
mechanism that can answer a question about the *project* rather than about a
file, and the only one that can answer it about generated code.

`read across, write local` (`expression-and-log.md`) already names the
cross-repository half of this. An index is what makes reading across possible
without opening anything.

## What has to be true for it to be honest

**An index is a cache, and must be marked as one.** It is stale the moment you
type, and the product must never present an indexed answer as though it were
live. The rule that already governs recordings governs this: an index may
speed up an answer, and may never *be* the answer to a question about
correctness. Say "**indexed at 14:02**" where the age matters, and let the
language server's live answer win wherever the two disagree.

**Staleness is the same problem, already solved once.** What makes an index
stale is exactly what makes a recording stale: an input changed. The cell's
input digest and the document's own hash are the signal, and a second
mechanism for "is this current" would be a second answer to one question.

**Positions must go through lineage or not be shown.** An index of woven files
holds `(file, line, column)` for files the user never edits. Presenting one
without mapping it back to a document span would send a person to a file that
regenerates over their edit — the exact failure `a-generated-file-refuses-an-edit`
exists to prevent. A reference that cannot be mapped back is not shown.

**Indexing is an install-shaped act, and inherits that machinery.** An indexer
is somebody else's binary fetched over the network, which is what
`tool_install` already refuses to do unsandboxed. `hick index install <lang>`
belongs beside `hick lsp install` and `hick dap install`, in the same
catalogue, with the same confinement and the same "your own copy wins"
discovery.

## The licence question, stated rather than assumed

`AGENTS.md` says **MIT only**, and the rule beneath the heading says a
dependency whose licence is *copyleft* cannot be linked in. Apache-2.0 is
permissive, not copyleft, so it passes the rule and contradicts the heading.
That is a decision to make deliberately, and it splits in two:

- **The `scip` crate would be LINKED** into this product, to read an index.
  That is the case the heading is about.
- **The indexers would be SPAWNED**, exactly as language servers and debug
  adapters already are. Nothing links them, and their licences are the user's
  business the way `rust-analyzer`'s already is.

The second needs no decision, and is what was built. The first does, and the
honest options are: read the Protobuf with a schema of our own (the format is
stable and documented), or amend the heading to say what the rule says.
Pretending Apache-2.0 is MIT is not one of them.

**Left open on purpose (2026-08-27).** The recommendation, recorded in the
guarantee rather than acted on: amend the heading. The rule beneath it — no
copyleft — is what expresses the actual concern, and Apache-2.0 does not
threaten it; the heading is a shorthand that has drifted from its own rule.
Writing our own Protobuf schema trades a dependency for a maintenance burden
on somebody else's evolving format and buys nothing legally. But it is a
licence decision about somebody else's product, so it is a recommendation and
not a change.

## Refusals

- **Never present an indexed answer as a live one.** An index that is allowed
  to be authoritative is a cache pretending to be a record, which is the
  mistake this product has already refused twice.
- **Never show a position that cannot be mapped back to a document.**
- **Never make the index required.** Every navigation feature that exists
  today must keep working with no index at all, on a machine that has never
  run an indexer. The index makes answers better and wider; it is never the
  reason something works.
- **Never index into the repository.** `.hick-cache/` is where a build
  artifact of this machine belongs, and `hick init` already ignores it.
