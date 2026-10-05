# Offline edits keep their sync base

Given an initialized local-first workspace, saving edits commits the local
files and their pending publication together. Disconnection leaves those files
editable and saved locally. The original remote revisions and base bytes stay
in a durable sync journal until an explicit sync succeeds. A conflict retains
both the local files and the remote files; it never retries as a blind overwrite.
An edit saved during a slow sync remains pending after that older upload finishes.
Local-save and remote-sync status are separate. Sync is explicit and has no
unbounded polling or background retry loop.

---

Last LLM verification:

- Date: 2026-10-04
- Reviewer: Codex
- Result: partially verified
- Evidence: `apps/web/src/embed/syncedStorage.ts` atomically stores mutations and
  the outbox, freezes one local snapshot, uses recorded remote revisions, then
  removes only paths whose local revision still equals that frozen snapshot.
- Test coverage: `syncedStorage.test.ts` covers offline reopening/base retention,
  conflicts, later edits during a held sync, deletion and local transaction
  failure. Browser acceptance exercises IndexedDB and two independent local
  workspaces against the S3 HTTP contract fixture.
- Caveat: first initialization requires a reachable remote snapshot. Browser
  durability and quotas apply. Conflicts offer explicit version selection or host-provided manually merged
  bytes, guarded by the reviewed local and remote revisions; there is no portable automatic merge or background pull.
  Closing the wrapper does not close the separately owned local/remote adapters.
