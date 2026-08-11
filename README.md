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

`hickory run` executes it. `hickory test` fails — with the line number and a
diff — if the tool's behavior ever changes. `match="regex-lines"` handles
timestamps and hashes.

## Run it

Hickory is a hosted, collaborative platform: teams work on the same documents
in the browser, with live editing, shared execution, and their own LLM keys.

It is GPL-3.0-or-later, so you can run your own instance — the whole thing
(server, web app, executor) is in this repository and deploys to Fly.io from
the included `fly.toml` and `Dockerfile`. See
[docs/operators/deploy-fly.md](docs/operators/deploy-fly.md).

The `hickory` CLI in `crates/hickory-cli` is the same engine, used for CI
verification and local development of documents. Install it in one line
(see [Reference](#install)) and run:

```sh
hickory test examples/text-tools-tour.hick
```

From a checkout, without installing anything:

```sh
cargo run -p hickory-cli -- test examples/text-tools-tour.hick
```

The examples are executable documents you can run against either executor:

```sh
HICKORY_EXECUTOR=docker cargo run -p hickory-cli -- test examples/
```

With `docker`, each document's `image=` is the environment it runs in. With
`local`, cells run against your host toolchain and `image=` is ignored — see
[examples/README.md](examples/README.md) for what each example needs.

The second example, `examples/bootstrap-ci.hick`, is a small statistical
paper: it generates its data from a fixed seed, computes a bootstrap
confidence interval, renders its own SVG histogram, and verifies its own
numbers. The figure cannot drift from the data because the figure is
computed from the data on every run.

## What just happened

1. The parser read the document and built a dependency graph from container
   declarations, volume reads/writes, and copy/paste references — not from
   source order.
2. Each `<hick:exec>` block ran in its declared container (locally: a plain
   process per container; hosted: a Firecracker microVM on a
   [Cloud Canopy](https://github.com/LoumTechnologies/cloud-canopy) node).
   Output was captured as a timed transcript.
3. Expectations were checked, outputs woven into `examples/*.md`, and every
   output byte tagged with provenance back to its source span.

## The agent writes literate programs

`hickory agent "add a section benchmarking sort vs awk"` runs an AI agent
whose *entire session* — your prompt, its reasoning, every script it ran,
every observation — is saved as a replayable `hick:session` document.

Bring your own key. Anthropic, OpenAI, DeepSeek, and Grok are supported;
`--provider` picks one and the matching `*_API_KEY` environment variable is
read. An unknown provider or a missing key fails before the first request,
naming the variable to set.
`hickory promote` compacts a session into a clean pipeline: last-write wins,
dead ends dropped. The agent's work product is a literate program in your git
history, not a chat log that evaporated.

## Status and boundaries

- The CLI, local execution, verification, weaving, and agent sessions work
  today (this repo, `cargo test` covers them).
- Local git-repo mode works today: `hickory init` installs the pre-commit
  drift gate and agent instructions — see
  [docs/users/local-mode.md](docs/users/local-mode.md).
- The LSP exists: `hick-lsp` (in `crates/hick-lsp`) multiplexes real language
  servers into `hick:file` blocks; a Zed extension lives in
  `editors/zed-hick` — see
  [docs/users/editor-setup.md](docs/users/editor-setup.md). Using AI agents
  with either mode: [docs/users/ai-agents.md](docs/users/ai-agents.md).
- A collaborative web app (notebook UI, live CRDT editing) and Tauri
  iOS/Android shells live in `apps/`. You can run the server yourself; it is
  part of this repository and under the same licence. Hickory is not a
  hosted service.
- Local execution is **not sandboxed** — it runs your documents' commands as
  your user, like `make`, and `image=` is recorded but ignored. Treat running
  an untrusted document the way you would treat running an untrusted
  Makefile. Sandboxed execution is what the Cloud Canopy backend is for.
- This is not a Markdown preprocessor: documents are DAGs with containers,
  capabilities, forks, and volumes, so a doc can prove things like "the
  report generator never talks to the network."

## Reference

### Install

```sh
curl -fsSL https://raw.githubusercontent.com/LoumTechnologies/hickory-docs/master/scripts/install.sh | sh
```

That puts `hickory` in `~/.local/bin` — no Rust toolchain, no clone. Add
`HICKORY_CHANNEL=unstable` for the build cut from the tip of `master` instead
of the latest stable release, or `HICKORY_INSTALL_DIR=…` to put it elsewhere.

Prefer to pick the file yourself? Every platform's archive is on the
[releases page](https://github.com/LoumTechnologies/hickory-docs/releases):
macOS (Apple Silicon and Intel, macOS 11+), Linux (x86_64 and aarch64,
glibc 2.35+ — Ubuntu 22.04, Debian 12, RHEL 9 and later), and Windows. Each
archive carries the binary, the licence, and `examples/`, so this works
straight out of it:

```sh
hickory test examples/text-tools-tour.hick
```

Windows is a `.zip` rather than part of the installer, and executing documents
there needs a POSIX `sh` — Git Bash or WSL both provide one. Building from
source stays a one-liner too: `cargo build --release -p hickory-cli`. Full
details, including how to verify a download:
[docs/users/install.md](docs/users/install.md).

### Commands

- `hickory run <doc|dir>` — execute, weave, write outputs
  - `--cache` — record each executed cell under `.hick-cache/transcripts/`,
    and answer a cell from its recording while the recording still matches.
    This is the only command that writes a recording, so it is how a
    `freeze="true"` cell gets its baseline.
  - `--freeze` — freeze every cell that does not say otherwise: answer it
    from its recording, never execute it. A cell's own `freeze="false"` still
    wins. Records nothing; a cell with no recording is an error.
- `hickory test <doc|dir>` — verify; four outcomes, four exit codes (below).
  It is `test`, not `check`, because it re-executes every cell in the
  document — the slowest, most side-effecting verb here. `cargo check`
  promises the opposite ("don't build, don't run"), so that name described
  a command hickory does not have.
  - `--freeze` — verify against recordings without executing. There is no
    `--cache` here on purpose: a verifier that can write its own baseline is
    not verifying anything, so an unrecorded cell is reported unverifiable
    (exit `2`).
- `hickory weave <doc>` — render from cached transcripts without executing
- `hickory agent "<prompt>"` — run an agent session (writes `sessions/*.hick`)
- `hickory promote <session.hick>` — compact a session into a pipeline
- `hickory refresh <doc>` — rewrite stale `hick:transform` passages
- `hickory init` — install the pre-commit drift gate in a git repo
- Language reference: `docs/` · Architecture: `docs/specs/freeform/architecture.md`

### `hickory test` exit codes

CI branches on these, so they are part of the CLI's public contract.

| Code | Outcome | What to do about it |
|---|---|---|
| `0` | verified | Re-derivation matches what is committed. Nothing to do. |
| `1` | drifted | A committed output no longer reproduces, or a `<hick:transform>` passage is stale — you forgot to regenerate. Re-run `hickory run <doc>` (or `hickory refresh <doc>` for a transform) and commit the result. Safe for CI to auto-fix. |
| `2` | unverifiable | A cell has no baseline at all — it neither executed nor was answered from a recording — so nothing was checked. Give it a recording (`hickory run --cache <doc>`) or stop freezing it. |
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
