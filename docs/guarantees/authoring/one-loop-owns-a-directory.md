# One loop owns a directory and never reacts to itself

Given an engine watching a folder, its own output writes are not treated as user
edits. App and CLI clients share the engine for that folder; the directory lock
prevents a second writer loop from starting.

An event is an echo when the file still holds the bytes last written by the loop.
Writes that already match are skipped, and filesystem events are debounced before
classification, so partial editor saves are not mistaken for completed edits.

The process holds an OS advisory lock for its lifetime. A Git checkout stores
`up.lock` in its private Git directory, including a linked worktree's own Git
directory. Other folders use personal workspace storage. Opening ordinary source
therefore adds no cache directory to the team's working tree. Killing the writer
releases the advisory lock without requiring deletion of a stale lockfile.

## Boundary

The lock covers one watched root. It does not prevent two overlapping roots or
coordinate unrelated external editors. Byte comparisons and the write gate
handle external changes; the lock is not a filesystem transaction.

---

Last LLM verification:

- Date: 2026-10-03
- Reviewer: Codex
- Result: partially verified
- Evidence: `up/mod.rs::DirectoryLock`, `up/state.rs::WovenState::is_echo`,
  `engine/mod.rs`, and the shared engine write gate.
- Test coverage: `tests/engine_lifecycle.rs` exercises shared clients, restart,
  reverse editing and source-backed view publication through the real engine.
  `tests/literate_views.rs` exercises independent linked inspection worktrees.
- Limits: the older `tests/up_loop.rs` suite still uses retired `.hick` fixtures;
  it has not been migrated as part of this change. Overlapping roots remain
  outside the guarantee.
