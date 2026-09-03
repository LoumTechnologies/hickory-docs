# Replacing the JetBrains All Products Pack

**Status: an audit, opened 2026-09-03.** The goal is stated by the person
paying for it: an All Products Pack subscription is being allowed to expire,
and this product has to carry the daily work that pack was carrying. This
document is the inventory that makes that answerable instead of hopeful —
what is built, what is partial, what is missing, and what should never be
built because it does not make sense here.

It is deliberately not a promise. Every row marked **built** names the
guarantee or module that makes it checkable, so the claim can be audited
rather than believed. A row with no evidence is not built, however obvious it
seems.

## The rule this audit follows

**Never claim a capability before it is real.** The rule already exists here
for debuggers — *never claim a language is debuggable before it is*
(`launching-what-a-document-builds.md`) — and this document generalises it to
the whole pack. The failure mode is specific and has already happened five
times: a list written down beside the thing it describes, which drifts, and
is believed because nothing checks it. The most recent instance is why this
audit exists at all — see `a-routed-language-is-drawn.md`, found on the first
day of this work.

## Where the languages stand

`hick lang` measures this on the machine it runs on; the table below is what
it reported on 2026-09-03, after the language table was unified. **DRAW** is
the editor's grammar, **LSP**/**DEBUG** are Silver, and the Gold columns are
omitted because nothing has reached Gold.

| JetBrains IDE | Language | DRAW | LSP | DEBUG | Standing |
|---|---|---|---|---|---|
| RustRover | rust | yes | yes | yes | **Silver, proven end to end** (2026-09-02) |
| PyCharm | python | yes | yes | yes | **Silver, proven end to end** |
| Rider | csharp | yes | get | yes | **Silver, proven end to end** (2026-08-27) |
| WebStorm | typescript, javascript, tsx, jsx | yes | get | get | Silver, **debugger unproven** |
| GoLand | go | yes | get | get | Silver, **unproven** |
| CLion | c, cpp | yes | get | yes | Silver, **unproven** |
| RubyMine | ruby | yes | get | get | Silver, **unproven** |
| DataGrip | sql | yes | get | n/a | Silver as a *language*; the tool is missing (below) |
| IntelliJ | java | yes | get | — | **Bronze — no debug adapter** |
| IntelliJ | kotlin | yes | get | — | **Bronze — no debug adapter** |
| IntelliJ | scala | yes | get | — | **Bronze — no debug adapter** |
| IntelliJ | groovy | yes | — | — | **Bronze — nothing but text** |
| PhpStorm | php | yes | get | — | **Bronze — no debug adapter (Xdebug exists)** |
| Rider | fsharp, vb | yes | — | — | **Bronze — nothing but text** |
| DataSpell | r | yes | yes | — | Bronze |
| AppCode (retired) | swift | yes | get | — | Bronze |
| — | dart, lua, shellscript | yes | get/— | — | Bronze |
| — | nix, zig | **—** | get | — | Bronze, and not even drawn |

The single largest language gap is **the JVM**: Java, Kotlin, Scala and
Groovy are IntelliJ's own subject and are the pack's centre of gravity, and
all four sit at Bronze. Java and Kotlin have a debug adapter that exists and
is permissively licensed — `java-debug` (Eclipse JDT, EPL) needs a licence
check against the no-copyleft rule before it can be considered.

## Cross-IDE features

Every JetBrains IDE ships roughly the same editor; these rows are what
"the pack" actually means day to day.

### Editing

| Feature | State | Evidence / note |
|---|---|---|
| Syntax highlighting | **built** | `a-routed-language-is-drawn.md` |
| Completion, diagnostics, hover, signature help | **built** | `a-language-server-is-found-without-being-configured.md`, `the-meta-lsp-forwards-what-the-child-supports.md` |
| Same intelligence on plain repo files | **built** | `a-plain-file-has-the-same-language-server.md` |
| Format on save, format now | **built** | `save-can-format-first.md` |
| Multiple cursors, rectangular selection | **built** | `several-cursors-edit-at-once.md` |
| Code folding | **built** | `folding.test.ts` |
| Find and replace, in file and tree | **built** | `find-and-replace-is-exhaustive.md` |
| Word navigation | **built** | `word-navigation-is-the-persons-choice.md` |
| Local history | **built** | `local-history-is-the-interval-below-the-commit.md` |
| Unsaved work survives a crash | **built** | `unsaved-work-survives-closing-the-app.md` |
| Inlay hints, code lens, semantic tokens | **built** | forwarded by `child_lsp` |
| **Structure view / breadcrumbs** | **partial** | `documentSymbol` is forwarded; no pane or breadcrumb bar draws it |
| **Live templates / postfix completion** | **missing** | — |
| **Bookmarks** | **missing** | — |
| **TODO view** | **missing** | — |
| **Scratch files** | **missing** | — |
| EditorConfig | **missing** | rustfmt/formatters honour it via the server; the editor does not |

### Navigation and refactoring

| Feature | State | Evidence / note |
|---|---|---|
| Go to definition, references, rename | **built** | `prepareRename`, `textDocument/references` forwarded |
| Search everywhere / go to file | **built** | `CommandBar.tsx` — **known defect**: ranks a typed full path below a fuzzier hit |
| Project-wide index across generated files | **built** | `an-index-answers-in-documents.md` |
| **Call hierarchy** | **missing** | `callHierarchy` appears nowhere |
| **Type hierarchy** | **missing** | `typeHierarchy` appears nowhere |
| Extract method/variable, change signature, safe delete | **missing** | only the server's own code actions are surfaced |

### Run, debug, test

| Feature | State | Evidence / note |
|---|---|---|
| Breakpoints, stepping, frames, variables | **built** | `hick-dap`, `a-plain-file-has-the-same-debugger.md` |
| Three breakpoint states, honestly reported | **built** | `a-breakpoint-that-is-not-bound-yet-is-not-refused.md` |
| Conditional breakpoints, hit counts, logpoints | **built** | `a-breakpoint-can-carry-a-condition.md` — engine since day one, given a door 2026-09-03 |
| Watches, evaluate expression, set variable | **built in the engine** | `client.ts` carries `eval`, `jump`, `runTo`, `drop_frame` |
| Build before launch for compiled languages | **built** | `a-compiled-language-launches-what-a-build-produced.md` |
| A missing debugger installs itself | **built** | `a-missing-debugger-is-a-button.md` |
| Run a single test from its gutter | **built** | `a-test-runs-from-the-line-it-is-written-on.md` |
| Terminal | **built** | the whole `docs/guarantees/terminal/` tree |
| **Exception breakpoints** | **partial** | reachable in `hick-dap`; no UI |
| **Code coverage** | **missing** | — |
| **Profiler (dotTrace/dotMemory)** | **missing** | — |
| **Remote debug / attach to process** | **missing** | — |
| **Run configurations** | **deliberately absent** | there is no `launch.json` and there must not be — everything is found without being configured |

### Version control

| Feature | State | Evidence / note |
|---|---|---|
| Stage, commit, amend, push, pull, branch, stash, diff | **built** | `the-git-pane-does-the-daily-loop.md` |
| Blame | **built** | `the-blame-column-is-optional-and-honest.md` |
| History as a readable story | **built** | `the-history-lens-reads-the-repository-as-a-story.md` |
| Interactive rebase (reword/move/drop) | **built** | `the-past-is-edited-by-rebase-above-the-floor.md` |
| Merge conflicts | **built** | `hick-documents-merge-through-hick.md`, `DivergedBanner` |
| **Cherry-pick, shelve, patch** | **missing** | — |
| **GitHub pull requests in the IDE** | **out of scope** | it would need a service; `gh` is a terminal away |

### The tool windows that are whole products

| Feature | State | Note |
|---|---|---|
| **Database tool (DataGrip)** | **missing** | The largest single missing product. It needs a driver, a connection store holding credentials, and a results grid. The credentials rule already exists: secrets belong to the user and stay on their machine, never written to a file we create. Nothing about it needs a server we run, so it is **in scope** — and a query console is very close to a cell that already runs. |
| **HTTP client** | **missing** | In scope and small: a `.http` file is a document whose cells are requests. Fits the existing model almost exactly. |
| Docker | **partial** | An executor, not a tool window |
| **Kubernetes, remote dev, Space** | **out of scope** | Each needs infrastructure this product refuses by decision (`local-only.md`) |
| AI assistant | **built, and the reason the product exists** | the whole `docs/guarantees/agent/` tree — and unlike the pack's, its output is auditable: see below |

## What this product has that the pack does not

Worth writing down, because the trade is not one-directional and these are
the reason to make it at all:

- **A note can prove itself.** Three kinds of provenance, drawn apart so they
  can never be mistaken for one another — lineage (the weave), context (what
  the model was actually shown), and declared (`cites=`)
  (`three-provenances-are-drawn-apart.md`). An AI summary in a note can be
  shown to still describe what it summarised.
- **Ribbons across documents** (`a-ribbon-crosses-documents.md`), so "why is
  this sentence here" is answerable across a whole folder.
- **Provenance and standing never render alike** — "AI-touched" or "no
  evidence of AI", never "human-written", which nothing can prove.
- **Documents that run**, with recordings keyed by their inputs and drift
  reported as drift.
- **No server, no account, no subscription.**

## The order to do this in

Ranked by how much daily work each unblocks, not by size:

1. **A JVM debug adapter** — Java and Kotlin. It is the pack's centre and the
   biggest single hole. Blocked on a licence check: `java-debug` is EPL, and
   the no-copyleft rule has to be applied to it explicitly rather than
   assumed either way.
2. **Audit the debugger UI against the engine.** ~~Conditional breakpoints and
   logpoints~~ — **done 2026-09-03**, and the prediction held exactly: the
   engine was complete, `debugStateEffects` hardcoded `conditional: false`,
   and the gutter's own conditional styling had never been reachable. Watches,
   evaluate and step-back are exposed. Still unreached: **exception
   breakpoints**, which `hick-dap` carries as `exception_filters` and no UI
   offers, and **set-variable**, which the wire supports and nothing calls.
3. **Prove the unproven Silvers.** Go, TypeScript, Ruby and C/C++ claim a
   debugger nobody has run. Each needs the end-to-end test C# and Rust got.
4. **Call hierarchy and type hierarchy** — two LSP requests, forwarded and
   drawn. Small, and among the most-used IntelliJ navigation.
5. **Structure view and breadcrumbs** — `documentSymbol` is already
   forwarded; this is a pane, not a protocol.
6. **The command bar's ranking defect** — a typed full path must win. Already
   found by dogfooding on 2026-09-02 and still open.
7. **The HTTP client**, which is nearly free given cells.
8. **The database tool**, which is not free at all and is its own design.

Rows 1–6 are what "ready to replace the pack" means for one engineer's daily
loop. Rows 7–8 are two of its products, and should be designed rather than
squeezed in.
