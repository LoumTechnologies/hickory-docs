# Developer environment

Short and procedural. See `docs/specs/freeform/architecture.md` for the
design and `docs/specs/freeform/local-only.md` for what this product is;
this page only gets you to a running app.

There is no database, no API server, and nothing to log into. The desktop app
runs the engine in its own process, so the dev environment is that app,
in dev mode, on a folder of documents.

## Machine dependencies

- Rust (pinned by `rust-toolchain.toml` — `rustup` picks it up automatically)
- Node.js 22+
- [`just`](https://github.com/casey/just)
- `cargo-tauri` — `cargo install tauri-cli --version '^2' --locked`
- On Linux, the system webview and bundler dependencies:
  `sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`

`just dev` checks each of these before doing anything and names the one
command that fixes whichever is missing.

## From clone to running app

1. `just dev` — idempotent. Installs the pre-commit hook, installs frontend
   dependencies if they are absent, seeds a scratch project if there is none,
   then opens the desktop app with the UI hot-reloading. Vite listens on a
   port derived from this worktree's path, so two checkouts can run it at
   once without agreeing on anything. Closing the window stops everything.
2. `just dev-seed` — writes `.dev/project/`: a two-stage chain
   (`decisions.hick` → `stats.hick` → `stats.py`), because a single document
   cannot show the lineage browser doing its job. Safe to run repeatedly —
   files you have edited are kept, never overwritten.
3. `just dev-stop` — for a run that was killed in a way that left the dev
   server behind. The normal exit is closing the window.
4. `just dev-clean` — stop, then delete **this worktree's** `.dev/` scratch.
   Never another worktree's, and never a cache anyone would have to download
   again.

To open a different folder, set `HICKORY_PROJECT_DIR` — the app uses it as
given and says so if it cannot be opened, rather than falling back to a
picker and hiding the typo.

## API client codegen

**Gone with the server.** `just codegen` and `just check-codegen` generated a
typed client from `apps/server`'s OpenAPI spec, and there is no
`apps/server` — the local server is `hickory_cli::serve`, reached only by the
app hosting it in the same process.

`apps/web/src/api/client.ts` and `types.ts` are hand-written and stay that
way. The contract they encode is the one in
`docs/specs/freeform/api.md`; when a route changes, both ends of it are in
this repository and change together.

## Checks before you push

- `cargo fmt --all` / `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `npm --prefix apps/web run typecheck && npm --prefix apps/web test`
- `hick test examples/ docs/` — the documents in this repository are part of
  the build, and drift in them fails CI the same way a failing test does.

The pre-commit hook runs the subset that applies to what you changed; CI runs
all of it on `master`. If the hook passes and CI fails on something the hook
could have caught, that gap is a bug in the hook —
see `.instructions/pre-commit-ci-parity.md`.
