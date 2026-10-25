<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/developers/developer-environment.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `8ae8b99f34fbd1609eb0124c942c1fda60d76f9dc43b42529b2989ea8acf90a7`.

<book:entry path="docs/developers/developer-environment.md">
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
- The WebAssembly target and `wasm-pack`, which build the editor's parser
  (`crates/hick-lang-wasm` → `apps/web/src/editor/generated/hick-lang`):
  `rustup target add wasm32-unknown-unknown` and
  `cargo install wasm-pack --version 0.15.0 --locked`. The built parser is
  committed, so these are needed to *change* the parser, and by the
  local codegen check — not to run the app.
- On Linux, the system webview and bundler dependencies:
  `sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`

`just dev` checks each of these before doing anything and names the one
command that fixes whichever is missing.

## From clone to running app

1. `just dev` — idempotent, and the only command you need. Every run brings
   the whole environment up to date before opening anything: the local
   hook, frontend dependencies (reinstalled when `package-lock.json` is newer
   than `node_modules`), the `hick` binary, and the seeded scratch project.
   Then it opens the desktop app with the UI hot-reloading — a frontend
   change lands in the open window without a rebuild or a reload.

   Two ports, both derived from this worktree's path so two checkouts can run
   at once without agreeing on anything: **Vite**, which the window loads, and
   the **engine** one above it, in the app's own process. Vite proxies `/api`
   — the live-sync WebSocket included — back to the engine, so the browser
   still sees a single origin. `just dev` prints both.

   That means the app is also **openable in a browser** at the Vite address,
   against the same engine and the same scratch project: useful for devtools,
   and the only way to inspect the UI without the webview. Closing the window
   stops everything.

   A **shipped** app works the other way round and has no hot reload in it:
   the window loads the engine, which serves the UI compiled into the binary
   from `apps/web/dist`. Both dev variables are absent there, which is what
   makes the downloaded app the thing this checkout describes rather than a
   configuration of it (`apps/desktop/src-tauri/src/dev.rs`).

   **There is no second command to remember.** If `just dev` shows you
   something, it is what is in this checkout — with one exception, which it
   prints by name: a fixture *you* edited is kept rather than overwritten.
2. `just dev-seed` — writes `.dev/project/`: a two-stage chain
   (`decisions.md` → `stats.md` → `stats.py`), because a single document
   cannot show the lineage browser doing its job, plus `cards.md`, the
   editor-chrome fixture. `just dev` runs this for you; run it directly only
   to re-seed without opening the app.

   It records what it wrote in `.dev/seed-manifest`, which is what lets it
   tell "you edited this" from "this is merely old". A file you have not
   touched is replaced when the fixture in git changes; a file you have
   edited is kept and listed at the end of the run, with the one way to take
   the new version (delete it and re-run, or `just dev-clean`).
3. `just dev-stop` — for a run that was killed in a way that left the dev
   server behind. It frees both ports and clears the scratch project's
   directory lock, which a killed engine has no chance to release; without
   that the next `just dev` refuses to start on a lock nobody holds. The
   normal exit is closing the window.
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

Run `just checks` for a blocking check run with one progress line per check.
Successful command output stays in log files; failures show the last 50 lines
and the full log path. It exits nonzero if a check fails. It accepts no commit
argument. Run `just checks-status` to see HEAD's recorded result and the latest
local run without starting checks.

Committing starts `just checks` in the background through `post-commit` and
returns immediately. There is no pre-commit gate. Launch errors and background
progress go to the Git common directory's `checks/hook.log`. Only one check run
executes at a time on the machine; additional invocations wait. Each waiting
invocation examines HEAD and the working tree after acquiring the lock, so
rapid commits can coalesce onto the same latest state rather than guaranteeing
a separate check of every intermediate commit.

A clean working tree (including no untracked, non-ignored files) is checked in
a disposable detached worktree and its result belongs to that exact commit.
You can keep editing while it runs. A dirty tree is checked in place; its
result never updates a commit's record, even if those edits are committed
while checks run. Avoid editing during a dirty run if you need a consistent
result. Ignored files are not part of the cleanliness test.

Results and complete logs stay under the Git common directory's `checks/`:
`latest.txt`, `commits/<full-hash>.txt`, and `runs/<run-id>/`. Records distinguish
running, passed, failed, and unchecked; the status command identifies a dead
supervisor as interrupted. A forcibly killed run can leave a disposable
worktree under that directory for later `git worktree remove --force` cleanup.
These are local records, not remotely published statuses. The lock is released
by the OS; on Unix a running child check retains it if its supervisor dies.

Checks retain the shared path selection in `scripts/affected-checks.sh`.
On master the comparison starts at HEAD's parent so a clean post-commit run
actually checks the new commit; branches compare with their merge base.
Untracked paths also participate in selection. A passing result means the
selected checks passed on this machine, not that every CI/platform job ran.
Clean runs install web dependencies from the lockfile when web checks apply;
Cargo build artifacts are reused from the original working tree's target
folder. No GitHub Actions service is needed for these checks.
</book:entry>

</book:doc>
