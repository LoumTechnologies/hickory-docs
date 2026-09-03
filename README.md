# Hickory Docs

Executable documents that refuse to lie. A `.hick` file is prose, a program,
and a test suite in one artifact: every example in it actually runs, every
output shown is the output produced, and drift between the document and
reality is a build failure.

*For engineers whose READMEs, quickstarts, and analysis notebooks have
silently rotted — and who want CI to catch it.*

## The problem

You wrote install docs six months ago. A flag was renamed, an output format
changed, and nobody noticed until a user did. Or: a colleague's analysis
notebook says "mean = 51.2" but re-running it says otherwise, and nobody can
tell which cells were run in what order.

Documentation drift isn't an editing problem. It's a verification problem.

## The model

A `.hick` document is XML-ish with one unusual rule: **only tags carrying the
`hick:` namespace prefix are structure; every other byte is raw text**. No
escaping, no CDATA — shell one-liners, Rust generics, and heredocs paste in
verbatim. Commands live in `<hick:exec>`, expected output in `<hick:expect>`,
generated files in `<hick:file>`, and the whole document weaves into clean
Markdown with the real transcripts inlined.

```xml
<hick:exec container="shell">
sort -t, -k2 -n fruit.csv
<hick:expect match="exact">apple,3
banana,5
cherry,7
apple,9
</hick:expect>
</hick:exec>
```

`hick run` executes it. `hick test` fails — with the line number and a
diff — if the tool's behavior ever changes. `match="regex-lines"` handles
timestamps and hashes.

## Run it

Hickory is a program you download. The `hick` CLI (in `crates/hickory-cli`)
runs on your machine, edits files in your repository, and executes on your
hardware — no server, no account, no telemetry, and every command except the
AI agent works fully offline.

Install it in one line (see [Reference](#install)) and run:

```sh
hick test examples/text-tools-tour.hick
```

From a checkout, without installing anything:

```sh
cargo run -p hickory-cli -- test examples/text-tools-tour.hick
```

By default each cell runs **sandboxed** against your host toolchain: it may
write only its own working directory, your dotfiles and keys are not there,
and it has no network unless the document declares one. `HICKORY_EXECUTOR`
selects another backend:

```sh
HICKORY_EXECUTOR=docker cargo run -p hickory-cli -- test examples/
```

With `docker`, each document's `image=` is the environment it runs in. With
`sandbox` (the default) or `local`, cells run against your host toolchain and
`image=` is ignored — see [examples/README.md](examples/README.md) for what
each example needs.

The second example, `examples/bootstrap-ci.hick`, is a small statistical
paper: it generates its data from a fixed seed, computes a bootstrap
confidence interval, renders its own SVG histogram, and verifies its own
numbers. The figure cannot drift from the data because the figure is
computed from the data on every run.

## What just happened

1. The parser read the document and built a dependency graph from container
   declarations, volume reads/writes, and copy/paste references — not from
   source order.
2. Each `<hick:exec>` block ran in its declared container — by default a
   confined process per container (bubblewrap on Linux, Seatbelt on macOS,
   AppContainer on Windows), or the declared `image=` under
   `HICKORY_EXECUTOR=docker`. Output was captured as a timed transcript.
3. Expectations were checked, outputs woven into `examples/*.md`, and every
   output byte tagged with provenance back to its source span.

## The agent writes literate programs

`hick agent "add a section benchmarking sort vs awk"` runs an AI agent
whose *entire session* — your prompt, its reasoning, every script it ran,
every observation — is saved as a replayable `hick:session` document.

Bring your own key. Anthropic, OpenAI, DeepSeek, and Grok are supported:
`--provider` (or `HICKORY_LLM_PROVIDER`) picks the vendor and the matching
`*_API_KEY` environment variable is read — and with exactly one key set, the
provider is picked for you. Your key stays in your environment and goes only
to the vendor you chose; hick never stores it, and nothing else in the tool
touches the network. An unknown provider or a missing key fails before the
first request, naming what to set.
`hick ingest --from session` compacts a session into a clean pipeline: last-write wins,
dead ends dropped. The agent's work product is a literate program in your git
history, not a chat log that evaporated.

## Status and boundaries

- The CLI, local execution, verification, weaving, and agent sessions work
  today (this repo, `cargo test` covers them).
- Bring your own agent: `hick doc read|read-output|edit|edit-output|verify`
  gives Claude Code, Codex, Grok CLI — anything that can run a command — the
  same hashline-anchored, lineage-backed tools the built-in agent uses, and
  `hick mcp` serves them over MCP. `HICKORY_SESSION` records the work as a
  replayable `hick:session`.
- Local git-repo mode works today: `hick init` installs the pre-commit
  drift gate, the MCP registration, agent instructions, and the editor wiring
  below — see [docs/users/local-mode.md](docs/users/local-mode.md).
- The LSP ships with the download: `hick-lsp` (in `crates/hick-lsp`) is in the
  same release archive as `hick` and multiplexes real language servers into
  `hick:file` blocks. `hick init` adopts whichever servers this repository's
  editor config already names (`.vscode/settings.json`, `.zed/settings.json`,
  `.helix/languages.toml`) into `.hick-lsp.json`, so a document's Python is
  checked by the same server as the `.py` file it generates. A Zed extension
  lives in `editors/zed-hick` — see
  [docs/users/editor-setup.md](docs/users/editor-setup.md). Using AI agents
  with either mode: [docs/users/ai-agents.md](docs/users/ai-agents.md).
- The desktop app is in development: `apps/desktop` is a Tauri window around
  the notebook UI. It runs the same engine in-process (no second
  implementation of weaving, lineage, or output edits) and answers the
  editor's language questions through the same `hick-lsp`. It builds from
  this repository today, but installable bundles are not yet a published
  download — the download is the CLI. It is not a client for any server;
  Hickory is not a hosted service.
- Execution is **sandboxed by default** (`HICKORY_EXECUTOR=sandbox`): each
  cell may write only its own working directory, your dotfiles and keys are
  not mounted, and there is no network unless the document declares one.
  Enforcement is bubblewrap on Linux, Seatbelt on macOS, AppContainer on
  Windows — and where none of them is available, `hick` refuses to run rather
  than quietly running unconfined. `HICKORY_EXECUTOR=local` opts out: commands
  run as your user, like `make` — reasonable for documents you wrote, a
  deliberate decision for anything else. Details:
  [docs/users/local-mode.md](docs/users/local-mode.md).
- Every executed cell runs under a **wall-clock limit** (default 120 seconds),
  so a cell that blocks on stdin or loops forever fails with a clear error
  instead of hanging a run or CI. Override it per cell with
  `timeout="<seconds>"` on the `<hick:exec>` tag (`timeout="0"` = unbounded,
  explicitly), or machine-wide with `HICKORY_CELL_TIMEOUT=<seconds>`. See the
  guide's "Cell timeouts" section.
- This is not a Markdown preprocessor: documents are DAGs with containers,
  capabilities, forks, and volumes, so a doc can prove things like "the
  report generator never talks to the network."

## Reference

### Install

```sh
curl -fsSL https://raw.githubusercontent.com/LoumTechnologies/hickory-docs/master/scripts/install.sh | sh
```

That puts `hick` in `~/.local/bin` — no Rust toolchain, no clone. Add
`HICKORY_CHANNEL=unstable` for the build cut from the tip of `master` instead
of the latest stable release, or `HICKORY_INSTALL_DIR=…` to put it elsewhere.

Prefer to pick the file yourself? Every platform's archive is on the
[releases page](https://github.com/LoumTechnologies/hickory-docs/releases):
macOS (Apple Silicon and Intel, macOS 11+), Linux (x86_64 and aarch64,
glibc 2.35+ — Ubuntu 22.04, Debian 12, RHEL 9 and later), and Windows. Each
archive carries the binary, the licence, and `examples/`, so this works
straight out of it:

```sh
hick test examples/text-tools-tour.hick
```

Windows is a `.zip` rather than part of the installer, and executing documents
there needs a POSIX `sh` — Git Bash or WSL both provide one. Building from
source stays a one-liner too: `cargo build --release -p hickory-cli`. Full
details, including how to verify a download:
[docs/users/install.md](docs/users/install.md).

### Commands

- `hick up [dir]` — weave a folder and keep it woven. Writes every document's
  outputs, then watches: a change to a document re-weaves it, and **a change
  saved in one of the generated files lands back in the document it came
  from**, mapped through lineage byte-exactly. This is what makes a `.hick`
  document editable with an editor that has never heard of hick.
  A generated file with nothing editable in it — a woven report, a pure
  transcript — is marked read-only while the loop runs, so the editor says so
  before you type; a save that touches generated text inside an otherwise
  editable file is refused, restored, and explained.
  - `--run` — execute a changed document in full on every save, rather than
    answering cells from `.hick-cache/transcripts/`.
- `hick run <doc|dir>` — execute, weave, write outputs
  A cell that declares `freeze="true"` runs exactly once — on the run that
  records it — and replays from then on, with no flag and no edit to the
  document.
  - `--cache` — record each executed cell under `.hick-cache/transcripts/`,
    and answer a cell from its recording while the recording still matches.
    Extends to the whole document what `freeze="true"` asks for one cell.
  - `--freeze` — freeze every cell that does not say otherwise: answer it
    from its recording rather than executing it, and record the ones that
    have no recording yet. A cell's own `freeze="false"` still wins.
- `hick test <doc|dir>` — verify; four outcomes, four exit codes (below).
  It is `test`, not `check`, because it re-executes every cell in the
  document — the slowest, most side-effecting verb here. `cargo check`
  promises the opposite ("don't build, don't run"), so that name described
  a command hick does not have.
  - `--freeze` — verify against recordings without executing. `test` writes
    no recording under any flag, and has no `--cache`, on purpose: a verifier
    that can write its own baseline is not verifying anything, so an
    unrecorded cell is reported unverifiable (exit `2`). This is the CI
    command for "is everything already recorded?".
- `hick weave <doc>` — render from cached transcripts without executing
- `hick lineage <doc> --output <file>` — print the byte-precise provenance of a
  generated output file: which source spans produced each byte range
- `hick agent "<prompt>"` — run an agent session (writes `sessions/*.hick`)
- `hick ingest --from session <session.hick>` — compact a session into a pipeline
- `hick refresh <doc>` — rewrite stale `hick:transform` passages; the only
  command that calls a model
- `hick init` — set up a git repo for hick: the pre-commit drift gate, the
  `hick` MCP registration, agent instructions, and editor wiring
- `hick doc read|read-output|edit|edit-output|verify` — read and edit a
  document through hashline anchors and lineage, the same tool set the
  built-in agent uses, for any coding agent that can run a command
- `hick mcp` — serve that tool set over MCP on stdio (Claude Code, Codex,
  Grok CLI, any MCP client)
- `hick search "<query>"` — search the project like semble: ranked chunks
  with exact file:line. Lexical out of the box; `--install-model` adds
  semantic ranking (a one-time ~30 MB download, the only network use);
  `--related FILE:LINE` finds similar code
- `hick lsp` — show or install the language servers that power the editor
- `hick dap` — show or install the debug adapters that power breakpoints
- Language reference: `docs/` · Architecture: `docs/specs/freeform/architecture.md`

### `hick test` exit codes

CI branches on these, so they are part of the CLI's public contract.

| Code | Outcome | What to do about it |
|---|---|---|
| `0` | verified | Re-derivation matches what is committed. Nothing to do. |
| `1` | drifted | A committed output no longer reproduces, or a `<hick:transform>` passage is stale — you forgot to regenerate. Re-run `hick run <doc>` (or `hick refresh <doc>` for a transform) and commit the result. Safe for CI to auto-fix. |
| `2` | unverifiable | A cell has no baseline at all — it neither executed nor was answered from a recording — so nothing was checked. Give it a recording (`hick run <doc>` — a frozen cell records itself the first time) or stop freezing it. |
| `3` | expectation failed | A `<hick:expect>` did not hold: the document claims something untrue of its own output. A human decides whether the claim or the code is wrong — never regenerate this away. |

Each is a different problem with a different fix, which is the whole point of
separating them. Drift means someone forgot to regenerate; unverifiable means
nothing was ever established; a failed expectation means a claim is false. A
CI job may reasonably auto-regenerate `1` and must never auto-anything `3`,
which is impossible if they share a code.

When more than one is present the strongest wins, in the order
verified < drifted < unverifiable < expectation failed. Unverifiable beats
drift because drift computed from a document that could not fully derive is
not trustworthy (those cells are reported and drift comparison is skipped
until they are fixed). A failed expectation beats both because it is the only
outcome asserting something is *definitely* wrong rather than out of date or
unknown — and it is never contaminated by a missing baseline, since an
unverifiable cell never evaluates an expectation. Every finding is still
printed; only the exit code is a single verdict.

A cell served from a recording (`freeze="true"`, see the guide) **is**
verified, not unverifiable: it has a baseline — the recording — and is
checked against it.

## Licence

GPL-3.0-or-later. See [LICENSE](LICENSE).

Hickory is free software: you may run, study, share, and modify it. If you
distribute a modified version, or run one as the basis of a network service
you distribute, those changes must be available under the same licence.
