# Hickory Docs — Architecture

Reproducible, verifiable, executable documents. A `.hick` file is simultaneously
prose, a program, a test suite, and an audit trail. Hickory Docs is an open-source
Jupyter/RMarkdown-class tool built on the hick language, with execution on
Cloud Canopy microVM nodes, a collaborative web/mobile app, and an AI agent whose
output is literate-programming files committed to git.

## Why this exists

- **Runnable, verifiable documentation for CLI tools** — docs whose examples
  execute on every change; drift between docs and code is a build failure
  (the exedocs vision, implemented natively — no cram, no VHS, no third-party CLIs).
- **Readable statistical papers** — documents whose figures and numbers are
  reproduced from source data at render time.
- **AI output as literate programming** — agent sessions are `.hick` files
  (`hick:session`), promoted into clean pipelines (`hick:doc`); the git repo is
  the state, with full provenance from every output byte back to its origin.
- An exploration of literate programming, the semantic web, code scaffolding,
  executable notebooks, and agent harnesses as one coherent system.

## Provenance of the code

Vendored from Nate's prior work (the `hickory-docs-task-63` monorepo and the
extracted `hick-*` repos), minus everything related to the wasm container
runtime (`hick-container`, `wasm-net`, `native2wasm`, container2wasm images).

| Vendored crate | From | Role |
|---|---|---|
| `hick-lang` | hick-lang repo / monorepo | Parser: `HickDocument` (pipelines) + `SessionDocument` (agent sessions), spans, includes |
| `hick-condition` | hick-exec | Boolean condition grammar for `when=` / features |
| `hick-flow` | hick-reactive | Reactive `Node` graph, converge, `ProvenanceMap` |
| `hick-exec` | hick-exec | DAG builder (containers, volumes, copy/paste, fork, attenuate), multi-doc state |
| `hick-literate` | monorepo `crates/hick-literate` | Pipeline orchestration, weave (literate output), promote (session→pipeline), compact/equiv, cache, provenance |
| `hick-store`, `hick-merge` | hick-store | Git-backed version store (`.hick/git/` plumbing, separate from user `.git`), three-way merge |
| `hick-token`, `hick-classify`, `hick-sink`, `hick-secrets` | hick-security | Macaroon capability tokens, data classification, egress sinks, age-encrypted secrets |
| `hick-grove` (+ server routes) | hick-reactive | Yrs (Yjs) CRDT engine over XML — the shared-state layer |
| `hick-agent-sdk` | hick-agent | `LlmClient` trait + Anthropic client, ReAct loop, session logging (`SessionLog`/`XmlSessionLog`), conversation tree |

Explicitly **not** vendored: `hick-container`, `hick-shell` (wasm binaries),
`hick-api` (in-wasm gateway), `wasm-net/*`, `native2wasm/*`, `hick-policy`
(shells out to Python — revisit), LSP/Zed extension (stays in its own repos).

## The execution boundary (the core surgery)

The monorepo fused execution into a concrete `ContainerExecutor`
(wasm-only). This repo replaces it with a trait, sized to the surface the
pipeline actually calls:

```rust
#[async_trait]
pub trait Executor: Send + Sync {
    async fn ensure_started(&self, container: &str, image: &str) -> Result<()>;
    async fn execute(&self, container: &str, command: &str, stdin: Option<&[u8]>) -> Result<ExecOutput>;
    async fn register_fork(&self, target: &str, from: &str, caps: ContainerCapabilities) -> Result<()>;
    async fn inject_volume(&self, container: &str, mount: &VolumeMount, data: &[u8]) -> Result<()>;
    async fn extract_volume(&self, container: &str, mount: &VolumeMount) -> Result<Vec<u8>>;
    async fn shutdown(&self) -> Result<()>;
    fn transcripts(&self) -> Transcripts;
}
```

Implementations:

1. **`LocalExecutor`** — commands run as host processes in per-container temp
   workdirs. No isolation guarantees; for dev, CI, and self-hosters who accept
   it. This is what proves the pipeline end-to-end tonight.
2. **`CanopyExecutor`** — Cloud Canopy: spawn a Firecracker sandbox via GraphQL
   (`spawnSandbox`, bearer capability token), stream I/O via the sandbox channel,
   destroy on shutdown. Canopy is mid-refactor (non-bare-metal mode in flight),
   so this adapter is the *only* file that knows canopy's API. Known gap: canopy's
   terminal WebSocket is cookie-authed; until token auth or an exec verb lands,
   output attachment over HTTP is limited — the adapter degrades gracefully and
   the gap is documented in `docs/specs/freeform/canopy-integration.md`.
3. Fork semantics degrade to command-history replay on both backends (as the
   original design did).

`.hick` adaptation: `image=` attributes name OCI-style references
(`python:3.12`). LocalExecutor ignores them (host tools). CanopyExecutor maps
them through a configured image table to Nix store sandbox images declared in
the node's ledger. Documents don't change shape; the mapping is deployment config.

## Native verification (exedocs vision, first-party)

- `<hick:exec ...>` bodies already capture transcripts. New: `<hick:expect>`
  as a child of `hick:exec` — expected stdout, with `match="exact|regex-lines"`
  (regex-lines: each line is a full-line regex, for timestamps/hashes).
- `hickory test <doc|dir>`: re-runs the pipeline, fails on any expectation
  mismatch or output drift vs the committed woven output. Exit code drives CI
  and the pre-commit hook (`hickory init` installs a sentinel-delimited hook,
  same idempotent design exedocs used — but implemented here, no cram).
- Recording: every run captures a timed transcript (JSON events: cmd, chunk,
  exit). The web UI plays these back as animated terminal sessions — replacing
  VHS GIFs with something better (scrubbing, copyable text).

## Product architecture

```
apps/
  server/        axum: REST + GraphQL-free JSON API, WS (Yrs sync + run events),
                 auth (argon2 + JWT sessions), Postgres (sqlx), billing webhooks
  web/           React + TypeScript + Vite: notebook UI (doc rendering, cell run,
                 transcript playback, live CRDT collab), pricing page from plans.json
  mobile/        Tauri v2 wrapping the same React+TS app; ios/ + android/ targets
crates/
  (vendored language + abilities crates, above)
  hickory-executor/        Executor trait + LocalExecutor
  hickory-executor-canopy/ CanopyExecutor
  hickory-cli/             `hickory` binary: run, check, weave, promote, serve, init
docs/  guarantees/  specs/freeform/  users/  developers/
```

- **State model**: git repo of `.hick` files is durable truth (hick-store's
  `GitVersionStore`); Yrs CRDT docs are the live collaborative layer; Postgres
  holds accounts, workspaces, run metadata, billing. Sessions/pipelines round-trip:
  CRDT ⇄ `.hick` text on save/load.
- **Sharing state across web/iOS/Android**: all three are the same React+TS
  client speaking the same WS Yrs protocol; Tauri contributes native shell +
  offline file access, nothing else diverges.
- **Where the server runs is not fixed**: `hickory serve` puts the same rooms,
  the same client, and the same executor on a contributor's own machine, with
  the `.hick` file standing in for Postgres. The collaboration layer is shared
  (`crates/hickory-collab`, one `DocStore` trait, two implementations) rather
  than reimplemented per host. See `local-collaboration.md` for the trust model
  (capability links, and why a shared session that grants `run` refuses to
  start on the unsandboxed local executor).
- **AI agent**: `hick-agent-sdk`'s ReAct loop with `LlmClient` (Anthropic,
  `claude-sonnet-5` default), executing through the same `Executor`. Every
  session is written as a `hick:session` document; "promote" turns it into a
  clean pipeline. Agent output is literally literate programming committed to git.

## Delivery

- Portfolio conventions: `master` branch only, `portfolio.toml` (tier=full),
  `justfile` at root, `docs/guarantees/`, Stripe catalog from `plans.json`.
- Substrate: **Fly.io** (managed PaaS; server + Postgres), app
  `hickory-docs-production`, described by the committed `fly.toml`. Railway was
  evaluated and dropped. The app reaches the local Cloud Canopy node via a
  portzero tunnel (or later canopy's nginx/ACME public console); node endpoint
  + token are environment config.
- Delivery: **continuous deployment to production**. Every green CI run on
  `master` deploys to hickorydocs.com (`.github/workflows/deploy-production.yml`).
  There is no staging environment and no promote gate — a deliberate pre-launch
  exception whose expiry conditions live in
  `.instructions/continuous-delivery-shared.md`. One consequence worth stating:
  with no staging, PostHog has a single production project rather than one per
  environment, so local-dev events must stay unconfigured rather than pointed
  at it.
- Open source (license: MIT, matching the prior hick codebases) once Nate
  flips repos public; private until then.
