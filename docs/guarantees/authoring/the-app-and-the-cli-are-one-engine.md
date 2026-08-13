# The Desktop App And The CLI Are One Engine

Given the desktop app open on a folder, when it weaves a document, computes
lineage, or carries an edit from a generated file back into its source, then it
runs the same code `hick up` runs — not a second implementation of it — and it
holds the same directory lock, so one folder is never open in two writers at
once.

The app links `hickory-cli` as a library and runs `serve::prepare` in its own
process. There is no subprocess, no IPC protocol, and no duplicated weave. A
change to how an output edit maps back into a document changes both front doors
at once, because there is only one of them.

Two properties hold that up:

1. **The UI and the API answer on one origin.** The app's window loads
   `http://127.0.0.1:<port>` — the server's own address — rather than Tauri's
   bundled `tauri://` origin. The frontend uses relative `fetch` paths and
   derives its WebSocket URL from `location.host`, so a split origin would
   break every request the editor makes, and would break it *only* in the
   packaged app, where it is hardest to notice. One origin also means no CORS
   policy and no mechanism for telling the frontend which port the server
   landed on.
2. **The lock is the same lock.** `up::DirectoryLock` is public precisely so
   the app can take it. Two processes weaving one directory is the same bug
   whichever front door they arrived through, so it has one answer.

## Where the static files live

In the **desktop crate**, not in `hickory-cli`. The CLI has no UI and never
serves HTML; `serve::prepare` returns a `Router` carrying only `/api` routes,
and the desktop crate layers the embedded UI onto it as a fallback. That
composition is why the CLI can stay headless while the app has a window.

The UI is compiled into the desktop binary with `rust-embed`. A downloaded app
has no checkout to find `apps/web/dist` in, and a loose directory beside the
executable is a thing a user can separate from it by moving one and not the
other.

## Boundary

An unknown path returns the shell rather than a 404, because client-side
routing depends on it.

The app resolves its folder from `HICKORY_PROJECT_DIR` or its first argument —
used as given, because both are deliberate acts and a fallback would hide a
typo — then the folder opened last time, then a native picker. The working
directory is never consulted: launched from a dock it is `/`.

A folder with no documents, or one another Hickory Docs process holds, is
recoverable rather than fatal: the app says which and asks again.

---

Last LLM verification:
- Date: 2026-08-12
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `apps/desktop/src-tauri/Cargo.toml` depends on `hickory-cli` by
  path. `apps/desktop/src-tauri/src/server.rs::start` acquires
  `hickory_cli::up::DirectoryLock`, calls `hickory_cli::serve::prepare`, layers
  `ui_handler` on as `Router::fallback`, binds `Ipv4Addr::LOCALHOST` port 0,
  and returns the bound address. `src/lib.rs::run` starts that before the
  window exists and opens the window with `WebviewUrl::External` at the
  returned URL, so page and API share an origin by construction.
  `crates/hickory-cli/src/serve/mod.rs::router` mounts no static-file service.
- Test coverage: `apps/desktop/src-tauri/tests/serves_one_origin.rs` —
  `the_ui_and_the_api_answer_on_one_origin` starts the real engine in a temp
  folder and asserts `GET /` returns the shell and `GET /api/health` and
  `GET /api/projects` succeed against the same base URL;
  `an_unknown_path_returns_the_shell` covers the routing fallback;
  `a_second_session_on_the_same_folder_is_refused` covers the shared lock;
  `a_remembered_folder_is_reopened`,
  `a_remembered_folder_that_is_gone_is_forgotten`, and
  `remembering_into_an_unwritable_place_does_not_panic` cover folder
  resolution and its failure modes.
- Caveat requiring review: nothing drives the actual Tauri window or the
  native folder dialog in a test. The window is created from the same URL the
  tests exercise and the picker's *result* feeds the same `server::start` they
  call, but that the webview loads the page and that the dialog appears are
  verified by hand, not by CI. The launch sequence's threading — dialogs off
  the main thread, engine started before the window exists — is likewise
  reasoned rather than tested.
