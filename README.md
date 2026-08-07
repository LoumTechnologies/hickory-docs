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

`hickory run` executes it. `hickory check` fails — with the line number and a
diff — if the tool's behavior ever changes. `match="regex-lines"` handles
timestamps and hashes.

## Install

Hickory is a single binary you run on your own machine. There is no service to
sign up for and nothing phones home.

```sh
cargo install --git https://github.com/LoumTechnologies/hickory-docs hickory-cli
```

Or from a clone:

```sh
git clone https://github.com/LoumTechnologies/hickory-docs
cd hickory-docs
cargo install --path crates/hickory-cli --locked
```

Requires a recent stable Rust. That puts `hickory` on your PATH.

Working on hickory itself? Build before testing — the server's LSP tests
spawn the `hick-lsp` binary, which `cargo test` alone does not produce.
`just test` does both.

## Try it

```sh
hickory run examples/text-tools-tour.hick
hickory check examples/text-tools-tour.hick
```

That one needs only a POSIX shell. The other examples invoke more tools —
local execution runs cells against your **host** toolchain, so see
[examples/README.md](examples/README.md) for what each needs.

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

- `hickory run <doc|dir>` — execute, weave, write outputs
- `hickory check <doc|dir>` — verify; non-zero exit on any drift
- `hickory weave <doc>` — render from cached transcripts without executing
- `hickory agent "<prompt>"` — run an agent session (writes `sessions/*.hick`)
- `hickory promote <session.hick>` — compact a session into a pipeline
- `hickory refresh <doc>` — rewrite stale `hick:transform` passages
- `hickory init` — install the pre-commit drift gate in a git repo
- Language reference: `docs/` · Architecture: `docs/specs/freeform/architecture.md`

## Licence

GPL-3.0-or-later. See [LICENSE](LICENSE).

Hickory is free software: you may run, study, share, and modify it. If you
distribute a modified version, or run one as the basis of a network service
you distribute, those changes must be available under the same licence.
