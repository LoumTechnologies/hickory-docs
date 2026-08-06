# Local mode: `.hick` documents in your own git repo

*For engineers who want executable documents in a repository they already
own — plain files, plain git, CI-style verification — instead of (or before)
a hosted workspace.*

## The two storage modes

| | Cloud workspace | Local git repo (this doc) |
|---|---|---|
| Where documents live | Per-project server-side git, mirrored to Postgres | `.hick` files in your repo |
| Editing | Web/mobile notebook UI with live CRDT sync (multiple cursors, offline merge) | Your editor + `hick-lsp` ([editor setup](editor-setup.md)) |
| Execution | Firecracker microVMs on Cloud Canopy nodes | `hickory` CLI, executor of your choice (below) |
| Drift gate | Server re-checks on save/run | `hickory init` pre-commit hook + `hickory check` in CI |
| Agent | Built-in Agent panel / `hickory agent` | Either the built-in agent or your own coding agent ([ai-agents](ai-agents.md)) |

The document format is identical in both. A repo can graduate to a cloud
workspace later (the server's durable state is also git), and cloud projects
can be cloned down.

## Set up a local repo

```sh
cargo install --path crates/hickory-cli   # installs `hickory`
cd your-repo
hickory init
```

`hickory init` is idempotent — run it again any time. It:

1. installs a **pre-commit hook** (a sentinel-delimited `### HICKORY ###`
   block, appended to any hook you already have; `core.hooksPath` is
   respected). At commit time the hook discovers all tracked `*.hick` files
   and runs `hickory check` on each; any drift blocks the commit. No `.hick`
   files, no-op.
2. adds `.hick-cache/` to `.gitignore` (cached transcripts).
3. writes a managed `<!-- HICKORY -->` section into `AGENTS.md` teaching
   coding agents the hick grammar and the golden rules (edit sources, run
   `hickory run` then `hickory check`), and points `CLAUDE.md` at it.
4. prints a toolchain doctor: warnings (non-fatal) for missing child language
   servers like `rust-analyzer` or `pyright-langserver`.

Daily loop:

```sh
$EDITOR docs/quickstart.hick
hickory run docs/quickstart.hick    # execute, weave quickstart.md, write outputs
hickory check docs/quickstart.hick  # verify — same command the hook runs
git add -A && git commit            # hook re-checks every tracked .hick doc
```

## Choosing an executor

`hickory run`/`check` execute each `h:exec` block through an executor,
selected by the `HICKORY_EXECUTOR` environment variable:

- **`HICKORY_EXECUTOR=local` (the default).** Commands run as ordinary host
  processes in per-container temp directories. Fast, zero setup, works
  anywhere the tools themselves run — and **not sandboxed**: documents run
  with your permissions, like `make`. `image=` attributes are recorded but
  ignored (your host tools are used).
- **`HICKORY_EXECUTOR=canopy`.** Each container becomes a Firecracker microVM
  on a [Cloud Canopy](https://github.com/LoumTechnologies/cloud-canopy) node —
  the same isolation the hosted platform uses, pointed at a node you run
  yourself. Configure the node endpoint and capability token per the
  `hickory-executor-canopy` crate docs.

## Platform reality for `canopy` locally

Firecracker requires **Linux with KVM**. Local sandboxed execution is
**Linux-first today**; everything else below is a workaround or future work.

- **Linux**: install cloud-canopy, verify `/dev/kvm` exists, run a node.
  Works today.
- **macOS**: no Firecracker. Options, in order of practicality:
  - Run a thin Linux VM with [Lima](https://lima-vm.io/) and host the canopy
    node inside it (nested virtualization needs Apple Silicon M3 or newer on
    macOS 15+ for KVM-in-VM to work).
  - Future: a libkrun-style backend for canopy using Hypervisor.framework
    directly — planned, not built.
  - Or just use `HICKORY_EXECUTOR=local` (unsandboxed) or a cloud workspace.
- **Windows**: run the canopy node inside **WSL2** with nested virtualization
  enabled (KVM inside WSL2 works on current Windows 11 builds). Point
  `hickory` on either side at it.

If you only need verification — not isolation — `local` is fine on all three
platforms.

## Don't assume

- **The pre-commit hook checks *tracked* `.hick` files, not just staged
  ones** — a doc drifted by someone else's change still blocks your commit,
  which is the point.
- **`hickory check` re-executes documents.** A slow document makes commits
  slow; keep heavyweight docs out of the hook by not tracking them, or gate
  them in CI only.
- **`.hick-cache/` is disposable** — never commit it; `hickory weave` uses it
  to render without executing.
- **Local mode has no live sync.** Two people editing the same `.hick` file
  merge through git like any other file. CRDT sync is a cloud-workspace
  feature.
