# A Local Session's Durable State Is The Working Tree

Given the desktop app open on a document, when anyone editing it makes a
change — the person typing in the app's editor, or `hick up` carrying an edit
back out of a generated file — then the change lands in the `.hick` **file on
that machine's disk**, within the persist debounce, and nowhere else is
authoritative.

There is no database. The file is the state, which is what makes the product
worth having: the user's other editor, `git diff`, `hick test`, and their
coding agent all see a change made in the app as an ordinary change to an
ordinary file, with no export step and no sync service in between.

Three properties hold that up:

1. **The room machinery is `hickory-collab`**, not a second implementation
   inside the app. Its [`DocStore`] is the seam: the file plus a
   `.hick-cache/crdt/` sidecar. Two implementations of a Yjs room is how the
   49 MB document-doubling incident behind migration 0003 happens twice.
2. **Writes are atomic.** The document is written through a sibling temp file
   and renamed, so an editor watching the file never observes a half-written
   document and a crash mid-save cannot truncate the user's work.
3. **The last socket flushes before the room can be dropped.** An edit the
   user watched happen must not be lost because they closed the window first.

The CRDT sidecar is a cache: losing it costs a re-seed, never data, so a
failure to write it is logged and never fails the document write.

## Why a CRDT with one user

Concurrency needs two writers, not two people, and this product has two by
design: the app's editor buffer, and the file on disk that the user's other
editor, a formatter, or `hick up` may rewrite underneath it. That is the same
merge problem collaboration had, with the same correct answer — which is why
`hickory-collab` survives the removal of sharing. See
`docs/specs/freeform/local-only.md`.

`hick up` on its own runs no rooms and writes the file directly, because with
no window open there is no second writer to merge against.

## Boundary

The session lives and dies with the process, and it is reachable only from
`127.0.0.1`. There is no sharing, no capability link, and no relay — not
disabled, not deferred; the product does not have them.

---

Last LLM verification:
- Date: 2026-08-12
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-collab/src/lib.rs` owns `Room`, `RoomRegistry`, the
  sync protocol, and the debounced persist; `DocStore` is the only seam.
  `crates/hickory-cli/src/serve/store.rs::FileDocStore` implements it against
  the filesystem, with `write_atomic` (temp file + rename) for both the
  document and the sidecar. `crates/hickory-cli/src/serve/socket.rs` calls
  `persist_now` before `drop_if_empty` on socket close, and no longer consults
  a token: `serve::serve` binds `Ipv4Addr::LOCALHOST` unconditionally and
  `serve::router` mounts no authorization layer.
- Test coverage: `crates/hickory-cli/tests/serve_local.rs` —
  `an_edit_in_one_editor_reaches_the_other_and_the_file_on_disk` drives two
  real WebSocket clients through the Yjs handshake, edits from one, asserts the
  other receives it, then polls the file on disk until the edit appears.
  `the_ribbons_have_their_data_without_a_database` covers the lineage path.
  `crates/hickory-collab` unit tests cover resume-not-reseed, external-source
  reconciliation, and the client-id derivation.
- Caveat requiring review: the interaction between a live room and `hick up`'s
  reverse edits is designed but not built — today `hick up` and the desktop app
  cannot hold the same directory at once, which is what the directory lock
  enforces. See `docs/guarantees/authoring/one-loop-owns-a-directory.md`.
