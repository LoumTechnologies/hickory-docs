# A Local Session's Durable State Is The Working Tree

Given `hickory serve` running on a machine, when anyone connected to that
session edits the document — the host in their browser, or someone holding a
share link — then the edit lands in the `.hick` **file on that machine's
disk**, within the persist debounce, and nowhere else is authoritative.

There is no database in a local session. The file is the state, which is what
makes the mode worth having: the host's editor, `git diff`, `hickory test`, and
their coding agent all see a collaborator's keystrokes as ordinary changes to
ordinary files, with no export step and no sync service in between.

Three properties hold that up:

1. **The room machinery is the hosted server's**, not a second implementation
   (`crates/hickory-collab`). The local session differs only in its
   [`DocStore`] — `docs.source` + `docs.crdt_state` in Postgres there, the file
   plus a `.hick-cache/crdt/` sidecar here. Two implementations of a Yjs room
   is how the 49 MB document-doubling incident behind migration 0003 happens
   twice.
2. **Writes are atomic.** The document is written through a sibling temp file
   and renamed, so an editor watching the file never observes a half-written
   document and a crash mid-save cannot truncate the host's work.
3. **The last socket flushes before the room can be dropped.** An edit a
   collaborator watched happen must not be lost because they closed the tab
   first.

The CRDT sidecar is a cache: losing it costs a re-seed, never data, so a
failure to write it is logged and never fails the document write.

## Boundary

The session lives and dies with the process. When the host closes their laptop,
the URL stops working and a guest's unsaved local CRDT state has nowhere to go
— see the open question in `docs/specs/freeform/local-collaboration.md`. That
is the honest limit of putting the server on a laptop, and the thing the hosted
product sells against.

---

Last LLM verification:
- Date: 2026-08-11
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-collab/src/lib.rs` owns `Room`, `RoomRegistry`, the
  sync protocol, and the debounced persist; `DocStore` is the only seam.
  `crates/hickory-cli/src/serve/store.rs::FileDocStore` implements it against
  the filesystem, with `write_atomic` (temp file + rename) for both the
  document and the sidecar. `crates/hickory-cli/src/serve/socket.rs` calls
  `persist_now` before `drop_if_empty` on socket close.
  `apps/server/src/doc_store.rs` is the hosted implementation of the same
  trait, and the hosted integration suite (including
  `reconnecting_does_not_duplicate_the_document`) passes unchanged after the
  extraction.
- Test coverage: `crates/hickory-cli/tests/serve_local.rs` —
  `a_collaborators_edit_reaches_the_other_client_and_the_hosts_file` drives two
  real WebSocket clients through the Yjs handshake, edits from one, asserts the
  other receives it, then polls the file on disk until the edit appears.
  `the_ribbons_have_their_data_without_a_database` covers the lineage path.
  `crates/hickory-collab` unit tests cover resume-not-reseed, external-source
  reconciliation, and the client-id derivation.
