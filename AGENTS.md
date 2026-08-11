# Hickory Docs

Reproducible, verifiable, executable documents in the hick language, with
Cloud Canopy execution, a React+TS web/mobile app, and an AI agent whose output
is literate-programming files in git.

Read two documents before changing anything:
`docs/specs/freeform/local-first.md` for what the product **is** (a program you
install; `hickory serve` hosts collaboration from the user's own machine; the
cloud is a relay, billing, and a static site), then
`docs/specs/freeform/architecture.md` for how it is **built** — accurate on the
language, crates, execution boundary, and verification, and superseded on the
hosted-workspace parts.

## Stack (settled — do not relitigate)

- Backend + CLI + language: Rust (edition 2024), axum, sqlx/Postgres, tokio.
- Frontends: React + TypeScript + Vite — one codebase for web AND Tauri v2
  (iOS + Android). No other frontend framework.
- Live sync: Yrs (Yjs) CRDTs over WebSocket (`hick-grove`). Durable state: git.
- Execution: `Executor` trait; `LocalExecutor` (dev/CI) and `CanopyExecutor`
  (Cloud Canopy GraphQL + capability tokens). NEVER reintroduce the wasm
  container runtime, and NEVER integrate third-party CLIs (cram, VHS, etc.) —
  verification and transcript capture are first-party.
- Product shape: **local-first** (`local-first.md`). `apps/server` is a hosted
  workspace kept running while the relay is built, not the product's backend —
  do not build new product features into it. New collaboration behaviour goes
  in `hickory-collab` (shared by both hosts) or `hickory serve`.
- Substrate: **Fly.io** (PaaS), app `hickory-docs-production`, config in the
  committed `fly.toml`. Railway was evaluated and dropped — do not reintroduce
  it. Branch: `master` only. **Production is continuously deployed**: every
  green CI run on `master` ships to hickorydocs.com via the Deploy Production
  workflow. There is no staging environment and no promote gate; this is a
  deliberate pre-launch exception, recorded with its expiry conditions in
  `.instructions/continuous-delivery-shared.md`.

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

@.instructions/api-client-codegen.md
@.instructions/config-and-environments.md
@.instructions/continuous-delivery-paas.md
@.instructions/continuous-delivery-shared.md
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
@.instructions/start-with-production.md
@.instructions/third-party-integration-mocking.md
@.instructions/user-facing-errors.md
