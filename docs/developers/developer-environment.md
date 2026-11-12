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

## Commit-triggered restarts

`just dev`, `just dev-site`, `just dev-spatial-mock`, `just dev-web API_ORIGIN`,
and `just simple-dev` (every target, including `ios`, `android` and `all`) start immediately and poll the resolved
commit of this worktree every 300 ms. Commits, amends, checkout, reset, merge
and rebase restart the session. Saving alone retains existing Vite hot reload;
uncommitted edits are included when the next session starts. Linked worktrees,
packed refs, detached HEAD and the first commit on an unborn branch use Git's
own revision resolution. Outside Git, the command fails with guidance.

The foreground Rust supervisor stops the owned process tree before restarting
preparation and the app, with the same arguments and environment. It allows
15 seconds for shutdown, then terminates remaining owned processes and refuses
a replacement if they do not exit. Ctrl-C or termination stops supervision and
children. `dev-stop` and `dev-clean` also end the desktop supervisor through a
worktree-local stop token under `target/`, so a later commit cannot revive it.
A successful window closure ends the command. Startup, build or
runtime failures stay visible and wait for the next commit; they do not retry
the same revision. Rapid commits coalesce; commits during preparation are checked
while it runs. Do not run two sessions of the same target in one worktree.

The desktop command retains its existing build, seed and dependency preparation.
Scoped web commands install missing or changed lockfile dependencies automatically;
unchanged dependencies are reused. Machine prerequisites remain Rust, Git and
Node/npm (plus the existing native SDKs for native targets). Unix cleanup uses
`ps`; Windows cleanup uses `taskkill /T`. The Apple comparison runner executes
the bundled binary directly so it owns the app, rather than only a detached
LaunchServices launcher.

Mobile comparison sessions are an explicitly authorized exception to the skill’s
one-shot phone-update rule: on a commit they rebuild, update in place and relaunch
the native app on the same selected serial/UDID. A host session keeps an emulator
or simulator it started alive across revisions, then stops it on final exit. App
identity and data are retained; in-memory UI state can reset. `all` delegates to
the same supervised platform commands. See [the comparison workflow](simple-frontends.md)
for SDK prerequisites, target selection and `simple-reload-test` device acceptance.
`dev-stop`, `dev-clean`, `dev-seed`, builds, generation, tests, checks, installations,
releases and deployments remain one-shot.

This focused update preserves existing public command names and scopes. It adds
commit supervision and automatic preparation for scoped web servers; it does not
claim to complete a repository-wide recipe naming migration or CI audit.

Run the harmless process witnesses with
`python3 -m unittest discover -s scripts/tests -p test_dev_supervisor.py -v`.
They use disposable Git repositories, never commits in this checkout. They cover
the restart trigger, startup races, failed revisions, process-tree ordering,
arguments/environment, linked-worktree isolation and Ctrl-C. Host execution is
separate from GUI, Linux and Windows acceptance; run the actual development
command on each supported host before claiming those platform checks passed.

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
