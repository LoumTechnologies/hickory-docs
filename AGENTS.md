# Hickory Docs

A downloadable **note-taking IDE** whose notes are `.hick` documents:
reproducible, verifiable, executable documents in the hick language, meeting
transcripts and their AI summaries ingested as ordinary notes, and an AI agent
whose output is literate-programming files in git. Notes can run, and an AI
summary in one can be proven to still describe what it summarized.

Read three documents before changing anything:
`docs/specs/freeform/notes-ide.md` for what the product is **for** (notes are
documents; meetings are inputs; the phone reads and captures but never
executes), then
`docs/specs/freeform/local-only.md` for what the product **is** (a program you
download; no server, no account, no relay, nothing to buy; `hick up` and a
desktop app over one engine — still true, and widened on purpose only by
`notes-ide.md`), then
`docs/specs/freeform/architecture.md` for how it is **built** — accurate on the
language, crates, execution boundary, and verification, and superseded on
everything hosted.

The surface syntax is settled separately in
`docs/specs/freeform/bare-documents.md`: the `<hick:doc>` wrapper is optional, a
document may begin with markdown or YAML frontmatter, and every document weaves
a `.md` of its own name.

An open question, investigated and deliberately not built, is
`docs/specs/freeform/two-branches-in-one-document.md`: whether feature flags
could replace branching in a `.hick` document. The short answer is that the
branch→feature-set half is worth doing and the branches-as-flags half would
cost the provenance that this product exists for.

A second open question, investigated and deliberately not built, is
`docs/specs/freeform/scaffolded-files-and-derived-edits.md`: how a document
owns files a scaffolder (`dotnet new`) wrote. The short answer is that "a block
that is a set of edits on top of something else" is a missing primitive worth
having, and that CRDT edits are the wrong way to express it — a re-run of a
scaffolder shares no history with the previous run, so the property that makes
CRDTs merge is absent exactly where it would be needed.

How a sentence someone posts proves itself — meeting → analysis → message,
with ribbons across documents — is walked through in
`docs/specs/freeform/receipts-for-a-message.md`, including what is still
clunky. How outside material enters a notes folder is `docs/specs/freeform/ingest.md`;
how the result is marked is `docs/specs/freeform/provenance-and-standing.md`.
The rule that governs both: **provenance is derived and checkable, standing is
declared and unverifiable, and the two must never render alike.** Say
"AI-touched" or "no evidence of AI" — **never** "human-written" or
"human-verified", which nothing can prove.

## Stack (settled — do not relitigate)

- CLI + language + local server: Rust (edition 2024), axum, tokio. No database.
- Frontend: React + TypeScript + Vite, shipped as the **desktop and mobile
  apps'** UI via Tauri v2 — one package, three entry points (`index.html` the
  editor, `site.html` the marketing page, mobile its own). No other frontend
  framework. It is not a client for any server we run — the only server it
  talks to is the one in the same process.
- Mobile (`notes-ide.md`): read and capture only. **No executor on iOS** — it
  cannot spawn a subprocess, so no cells, no terminal, no LSP, no DAP. Notes
  render via `hick weave` from cached transcripts. Devices meet through the
  **user's own git remote**, never through anything we run.
- Distribution (`shipping-mobile-and-desktop.md`): **App Store and Play Store**
  for mobile; **`.dmg`, `AppImage`, `.msi`** for desktop, from GitHub releases.
  Two thin Tauri shells over one core — never one shell threaded with
  `#[cfg(mobile)]`. The portable half is the language, documents, weave, and
  transcripts (`hick-lang` and `hick-transcript` verified compiling for
  `aarch64-linux-android`); the host-process half never goes near a phone.
- Live sync: Yrs (Yjs) CRDTs (`hick-grove`, `hickory-collab`). Durable state:
  the user's git repository. The CRDT survives the removal of collaboration
  because the app's editor buffer and the file on disk are still two writers.
- Execution: `Executor` trait; `LocalExecutor` (default) and the Docker
  executor. NEVER reintroduce the wasm container runtime, and NEVER integrate
  third-party CLIs (cram, VHS, etc.) — verification and transcript capture are
  first-party.
- Product shape: **local-only** (`local-only.md`). There is no backend, no
  account, no relay, and no monetization. Do not add one. Anything that would
  need a server we operate is out of scope, not a later phase. Say "nothing
  talks to a server we run" — **never** "your notes never leave your machine",
  which sync to the user's own git remote makes untrue.
- Delivery: a **downloadable product**. `master` only. The release channels
  (`unstable-release.yml`, `stable-release.yml`) and the one-line installer are
  the delivery path; hickorydocs.com is static files and a download link.
  See `.instructions/continuous-delivery-downloadable.md`, as amended by
  `shipping-mobile-and-desktop.md`: mobile publication is App Review's verb, not
  a green CI run, and signing is part of delivery rather than a later polish
  step — an unsigned installer reads to a new user as malware.

## Rules

- `just` is the only task-runner entry point; recipes live at repo root.
- The hick parser's no-escaping invariant is sacred: only namespace-prefixed
  tags are structured; all other text is raw, byte-for-byte. No CDATA, no
  entity escaping. Docs about hick use a different prefix (`h:`) so `hick:`
  examples stay literal — and because rebinding the prefix requires the explicit
  `<h:doc xmlns:h="…">` root, those documents keep their wrapper while ordinary
  notes drop it (`bare-documents.md`).
- Public API of each crate is its explicit `pub use` facade in `lib.rs`.
- Guarantees live in `docs/guarantees/` (one file per guarantee, Given/When/Then
  + verification block); update them in the same change as the implementation.
- Vendored crates keep their names; upstream repos are frozen references —
  fixes happen here, not there.
- cloud-canopy is being modified concurrently by another agent — only
  `hickory-executor-canopy` may know its API; never edit the cloud-canopy repo.
  It stays an **optional** executor pointed at a node the *user* runs; it is
  not a service we operate, and nothing may require it.
- **MIT only.** A dependency whose licence is GPL or otherwise copyleft cannot
  be linked into this product. The live example: `grit` splits its licence —
  `grit-lib` is MIT and usable, `grit-cli` is GPL-2.0 and is not. Check the
  crate, not the project.
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
