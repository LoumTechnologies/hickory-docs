# Hickory Docs

A downloadable tool for literate programming where you can edit the generated
files: reproducible, verifiable, executable documents in the hick language,
with an AI agent whose output is literate-programming files in git.

Read two documents before changing anything:
`docs/specs/freeform/local-only.md` for what the product **is** (a program you
download; no server, no account, no relay, nothing to buy; `hick up` and a
desktop app over one engine), then
`docs/specs/freeform/architecture.md` for how it is **built** — accurate on the
language, crates, execution boundary, and verification, and superseded on
everything hosted.

## Stack (settled — do not relitigate)

- CLI + language + local server: Rust (edition 2024), axum, tokio. No database.
- Frontend: React + TypeScript + Vite, shipped as the **desktop app's** UI via
  Tauri v2. No other frontend framework. It is not a client for any server we
  run — the only server it talks to is the one in the same process.
- Live sync: Yrs (Yjs) CRDTs (`hick-grove`, `hickory-collab`). Durable state:
  the user's git repository. The CRDT survives the removal of collaboration
  because the app's editor buffer and the file on disk are still two writers.
- Execution: `Executor` trait; `LocalExecutor` (default) and the Docker
  executor. NEVER reintroduce the wasm container runtime, and NEVER integrate
  third-party CLIs (cram, VHS, etc.) — verification and transcript capture are
  first-party.
- Product shape: **local-only** (`local-only.md`). There is no backend, no
  account, no relay, and no monetization. Do not add one. Anything that would
  need a server we operate is out of scope, not a later phase.
- Delivery: a **downloadable product**. `master` only. The release channels
  (`unstable-release.yml`, `stable-release.yml`) and the one-line installer are
  the delivery path; hickorydocs.com is static files and a download link.
  See `.instructions/continuous-delivery-downloadable.md`.

## Rules

- `just` is the only task-runner entry point; recipes live at repo root.
- The hick parser's no-escaping invariant is sacred: only namespace-prefixed
  tags are structured; all other text is raw, byte-for-byte. No CDATA, no
  entity escaping. Docs about hick use a different prefix (`h:`) so `hick:`
  examples stay literal.
- Public API of each crate is its explicit `pub use` facade in `lib.rs`.
- Guarantees live in `docs/guarantees/` (one file per guarantee, Given/When/Then
  + verification block); update them in the same change as the implementation.
- Vendored crates keep their names; upstream repos are frozen references —
  fixes happen here, not there.
- cloud-canopy is being modified concurrently by another agent — only
  `hickory-executor-canopy` may know its API; never edit the cloud-canopy repo.
  It stays an **optional** executor pointed at a node the *user* runs; it is
  not a service we operate, and nothing may require it.
- No server, no account, no payment, no telemetry. A change that needs any of
  them is out of scope by decision — see `local-only.md`.

@.instructions/config-and-environments.md
@.instructions/continuous-delivery-downloadable.md
@.instructions/continuous-integration.md
@.instructions/dev-environment.md
@.instructions/documentation-layout.md
@.instructions/framework-agnostic-system-tests.md
@.instructions/github-issues.md
@.instructions/just.md
@.instructions/one-man-team.md
@.instructions/pre-commit-ci-parity.md
@.instructions/pre-launch.md
@.instructions/semble.md
@.instructions/specification-levels.md
@.instructions/third-party-integration-mocking.md
@.instructions/user-facing-errors.md
