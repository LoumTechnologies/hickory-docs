# Hickory Docs — the desktop app (Tauri v2)

The window around the editor. The UI is `apps/web`'s default build
(`index.html` → `apps/web/dist`), which Tauri bundles; the marketing site is a
separate build from the same source tree (`site.html` → `dist-site`) and is
never part of this bundle.

- Bundle id: `com.loumtechnologies.hickorydocs`
- Rust crate: `src-tauri/` — deliberately **outside** the root Cargo workspace
  (empty `[workspace]` in its `Cargo.toml`) so the Tauri build's requirements
  never break `cargo check --workspace` at the repo root.
- Capabilities: `src-tauri/capabilities/default.json` (core defaults only).

## What it hosts

The desktop app runs the local document server **in-process**:
`hickory_cli::serve` on `127.0.0.1`, answering the same routes the editor
already calls. That makes the app and `hick up` two front doors onto one
engine rather than two implementations — see
`docs/specs/freeform/local-only.md`.

It takes the same directory lock `hick up` takes, so opening a folder in the
app while `hick up` is watching it is refused rather than producing two
processes writing the same files.

The window loads `http://127.0.0.1:<port>` rather than Tauri's bundled
`tauri://` origin, and the server serves the UI as well as the API. One origin
means relative `fetch` and `location.host` WebSocket URLs work exactly as they
do in a browser — no CORS policy, and no need to tell the frontend which port
the server landed on. The UI is compiled into the binary with `rust-embed`, so
a downloaded app needs no `dist/` beside it.

The static-file serving lives in this crate, not in `hickory-cli`: the CLI has
no UI and never serves HTML. `serve::prepare` hands back a `Router`, and this
crate layers the assets onto it.

## Which folder it opens

1. `HICKORY_PROJECT_DIR`, then the first command-line argument. Either one is
   a deliberate act, so it is used as given and **not** second-guessed: if it
   cannot be opened the app says so rather than falling back to a picker and
   hiding the typo.
2. The folder opened last time, if it still exists.
3. Otherwise a native folder picker.

The working directory is never consulted. An app launched from a dock, Finder,
or a Start menu inherits `/`, and opening `/` because nobody said otherwise is
not a sensible default.

A folder with no `.hick` documents in it, or one another Hickory Docs process
already holds, is not fatal: the app says which and asks again. Cancelling the
picker with nothing open exits quietly — that is a decision not to use the app
right now, not an error.

## Prerequisites

```sh
cargo install tauri-cli --version '^2'   # provides `cargo tauri`
cd apps/web && npm install               # frontend deps
```

## Install this checkout on an Apple Silicon Mac

Run `just local-install` from the repository. It builds the release app with
its embedded UI, installs `/Applications/Hickory Docs.app`, and registers it
with macOS. Re-run the command to replace the installed app with a fresh build.

Launch **Hickory Docs** from Applications or Spotlight. The welcome pane opens
by default, with your restored tabs still available. Uncheck “Show this page
when a folder opens” on the welcome pane to skip it on later launches.
The app runs independently of this checkout and needs no development server.

## Desktop only

iOS and Android are not targets. A tool whose job is editing files in a git
repository and running code on your machine has no phone story yet; if that
changes, Tauri v2 supports both from this same shell.
