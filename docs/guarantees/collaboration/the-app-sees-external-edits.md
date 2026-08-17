# The App Runs The Up-Loop: External Edits Are Seen, Never Clobbered

Given the desktop app open on a folder, when any other program — vim, a
formatter, a coding agent, `git checkout` — changes a `.hick` document on
disk, then the live editor buffer follows the file (the change reconciles
into the document's room); when such a program saves an edit in a generated
output file, then the edit is carried back into the document it came from,
exactly as headless `hick up` would carry it; and when a new `.hick` file
appears, then it is indexed, woven, and openable without restarting the app.

This is `local-only.md`'s two-writer design made true: the CRDT exists
because the editor buffer and the file on disk are both writers, and a
watcher that isn't running means the room's next debounced persist silently
overwrites whatever the other writer did. For a tool pitched at the agentic
era, an agent editing the repo while the window is open is the ordinary
case, not the edge.

Two properties keep the loop honest:

1. **The rooms' own persists are echoes, not edits.** The store remembers
   the exact text of its last persist per document; a watcher event whose
   file still holds it is skipped. Without this, reconciling the persist
   back into the room would revert keystrokes typed since — the loop
   fighting the person typing.
2. **The marks come off however the loop ends.** Read-only marking of
   fully-generated files is a live signal; shutdown clears it, and
   `write_outputs` clears it defensively for the path where the process was
   killed first.

## Boundary

The in-app loop weaves on change and never executes cells — running is the
Run button's deliberate act, matching headless `hick up` without `--run`.
The loop takes no directory lock of its own: the desktop shell already holds
the folder's lock, which is also what keeps `hick up` and the app mutually
exclusive on one folder.

A folder with no documents is a valid session (the app's first-run state):
the server starts with an empty index and the loop waits.

---

**Verification notes (2026-08-16).** Implementation:
`crates/hickory-cli/src/serve/watch.rs` (`spawn`, `run_loop`,
`reconcile_rooms`), reusing `crate::up`'s `handle_batch`/`weave_document`/
`WovenState`; echo test `FileDocStore::was_own_write` recorded in
`FileDocStore::save` (`crates/hickory-cli/src/serve/store.rs`); empty-folder
sessions via `contains_hick` in `DocIndex::scan` (same file); the desktop
shell starts the loop in `apps/desktop/src-tauri/src/server.rs::start` and
holds its `WatchGuard` in `Session`. Test coverage:
`crates/hickory-cli/tests/serve_watch.rs` (empty folder + late document,
generated-file edit round trip, external doc edit reaching a live room, echo
recognition) and the updated `a_session_with_no_documents_starts_empty` in
`tests/serve_local.rs`. Caveats: shutdown-clears-marks is exercised via
`WatchGuard::shutdown` in tests, but the app-quit path (drop without await)
is best-effort by design; reconciliation while a user is mid-keystroke
relies on the echo test plus `apply_external_source`'s no-op-on-equal — a
true simultaneous external edit and unsaved buffer merge through the CRDT,
which is LLM-reviewed rather than tested here.
