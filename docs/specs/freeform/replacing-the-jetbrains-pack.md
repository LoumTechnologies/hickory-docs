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
| GoLand | go | yes | get | yes | **Silver, proven end to end** (2026-09-04) |
| CLion | c, cpp | yes | get | yes | **Silver, both proven end to end** (2026-09-04) |
| WebStorm | typescript, javascript | yes | get | yes | **Silver, proven end to end** (2026-09-04) |
| WebStorm | tsx, jsx | yes | get | — | **Bronze — node cannot run either; a component is not a program** |
| RubyMine | ruby | yes | get | — | **Bronze — rdbg does not fit the adapter shape** |
| DataGrip | sql | yes | get | n/a | Silver as a *language*; the tool is missing (below) |
| IntelliJ | java | yes | yes | yes | **Silver, proven end to end** (2026-09-04) |
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

## What CI verifies, per language

Asked and measured on 2026-09-04 rather than assumed. "Covered" means a test
drove the real thing — a server to an answer in document coordinates, an
adapter to a stop — not that the code exists.

| Language | Debugger | Language server (document) | Language server (plain file) |
|---|---|---|---|
| Python | yes | yes | yes |
| Rust | yes | yes | yes |
| Go | yes | yes | yes |
| TypeScript | yes | yes | yes |
| JavaScript | yes | via TypeScript's server | via TypeScript's server |
| Java | yes | yes | yes |
| C | yes | yes (clangd) | yes |
| C++ | yes | yes (clangd) | yes |
| C# | yes (own job) | yes (own job) | yes (own job) |
| PHP | — | yes (intelephense) | yes |
| Ruby, Kotlin, Scala | — | not covered | not covered |

**A project-loading server needs the project where the code actually is.**
This was recorded on 2026-09-04 as "C# cannot work in a document", and that
was wrong — the wrong configuration had been measured. Corrected 2026-09-05
by driving `hick-lsp` by hand and watching csharp-ls's own log.

A document's code is staged into a temp directory, and the child server is
rooted **there**, not at the folder holding the `.hick` file. So a `.csproj`
written beside the document is a project the server never sees, and one the
document *generates* is staged next to the code it describes and loads
normally. With that, hover answers `int Lib.Summarise(string path)` on the
first attempt.

That is not a testing trick: a real C# document generates its own `.csproj`,
because that is exactly what `hick ingest` writes when it takes in what
`dotnet new` produced. The fixture that had one placed beside it was the
artificial case.

The rule generalises to any server that loads a project before it will
answer — and C# is the only one here that does, which is why nothing else
needed it.

Ruby, Kotlin and Scala have servers hick can discover and no fixture drives
them. That is the remaining gap in this table, and it is honest rather than
hidden: adding each means installing its server in CI.

## Cross-IDE features

Every JetBrains IDE ships roughly the same editor; these rows are what
"the pack" actually means day to day.

### Editing

| Feature | State | Evidence / note |
|---|---|---|
| Syntax highlighting | **built** | `a-routed-language-is-drawn.md` |
| Completion, diagnostics, hover, signature help | **built** | `a-language-server-is-found-without-being-configured.md`, `the-meta-lsp-forwards-what-the-child-supports.md`, `a-hover-is-rendered-not-dumped.md` |
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
| Search everywhere / go to file | **built** | `a-typed-path-opens-that-file.md` — the 2026-09-02 ranking defect is fixed |
| Project-wide index across generated files | **built** | `an-index-answers-in-documents.md` |
| Call hierarchy | **built, server side** | `call-hierarchy-answers-in-documents.md` — no panel in the app yet |
| **Type hierarchy** | **blocked** | `lsp-types` 0.94.1 cannot declare the server capability |
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
| Exception breakpoints | **built** | `an-exception-breakpoint-uses-the-adapters-own-filters.md` |
| **Variables pane** | **missing** | values are drawn inline, not in a pane — which is also why set-variable, whose whole path exists, has nowhere to live |
| **Code coverage** | **missing** | — |
| **Profiler (dotTrace/dotMemory)** | **missing** | — |
| **Remote debug / attach to process** | **missing** | — |
| **Run configurations** | **deliberately absent** | there is no `launch.json` and there must not be — everything is found without being configured |
| New project (per language) | **built for .NET, Python (uv), Rust (cargo)** | `a-new-project-uses-the-toolchain-the-language-uses.md`; Go and Node not offered |

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

- **A note can prove itself** — and this was exercised end to end on
  2026-09-04 rather than taken on trust. Changing a fact an AI summarised
  makes `hick test` fail offline with the line, the selector and the
  instruction. `hick cites` resolves a message's every claim back across four
  documents to specific lines. `hick context` says, for a line the agent
  wrote, exactly what was in front of the model — files at a sha256 and a
  commit — and says plainly that it knows nothing when the session is gone.
  Two real defects were found and fixed: a citation resolving to nothing was
  silently dropped (`a-citation-that-points-at-nothing-fails.md`), and an
  edited AI passage went on claiming the model wrote it
  (`an-edited-ai-passage-stops-claiming-the-model-wrote-it.md`).
- **The three provenances.** Three kinds of provenance, drawn apart so they
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

1. ~~**A session tree, for JavaScript and TypeScript.**~~ **Done 2026-09-04.**
   hick answers `startDebugging`, opens a sibling connection, and runs the
   real session on it. Seven languages now debug live — Python, C, C#, Go,
   JavaScript, Rust, TypeScript — and each has a test that stops on a
   document line and reads a value out of the frame
   (`an-adapter-that-runs-on-a-second-connection-is-followed.md`). One child
   is followed; a target that spawns further targets is not.
2. ~~**A JVM debug adapter**~~ — **Java done 2026-09-04.** The measurement
   answered yes: jdt.ls imports a directory with no build file at all, so a
   document need not generate a `pom.xml`, and no `javac` is required.
   `a-debugger-that-lives-in-a-language-server-is-asked-for.md`. **Kotlin,
   Scala and Groovy are still Bronze** and share none of this — each is its
   own adapter and its own investigation.
3. **Audit the debugger UI against the engine.** ~~Conditional breakpoints and
   logpoints~~ — **done 2026-09-03**, and the prediction held exactly: the
   engine was complete, `debugStateEffects` hardcoded `conditional: false`,
   and the gutter's own conditional styling had never been reachable. Watches,
   evaluate and step-back are exposed. Still unreached: **exception
   breakpoints**, which `hick-dap` carries as `exception_filters` and no UI
   offers, and **set-variable**, which the wire supports and nothing calls.
4. ~~**Prove the unproven Silvers.**~~ **Done 2026-09-04, and all four were
   broken.** Every language with a live test worked; every language without
   one did not, exactly. Go and C/C++ now work and have tests
   (`an-adapter-that-listens-is-connected-to.md`); JavaScript, TypeScript and
   Ruby are now reported as **not** debuggable, because they are not. The
   next thing that would make WebStorm's languages real is a **session
   tree**: js-debug answers `launch`, marks the breakpoint provisional, and
   asks the client to start a second session where breakpoints actually
   bind.
5. ~~**Call hierarchy**~~ — **server side done 2026-09-04**; the app still
   needs a panel to draw it. **Type hierarchy** is blocked on `lsp-types`,
   which cannot declare the capability, so a client would never ask.
6. **Structure view and breadcrumbs** — `documentSymbol` is already
   forwarded; this is a pane, not a protocol.
7. ~~**The command bar's ranking defect**~~ — **done 2026-09-03**. It was two
   defects: no ranking at all, and a result cap applied during the tree walk
   rather than after ranking, which threw the best match away unscored.
8. **The HTTP client**, which is nearly free given cells.
9. **The database tool**, which is not free at all and is its own design.

Rows 1–7 are what "ready to replace the pack" means for one engineer's daily
loop. Rows 8–9 are two of its products, and should be designed rather than
squeezed in.

**The rule that earned its place on 2026-09-04, and applies to every row
above: write the live test first.** Four languages claimed a debugger and
none of them had one, and the thing that distinguished the three that worked
from the four that did not was, exactly and only, whether a test had ever
driven them to a stop.
