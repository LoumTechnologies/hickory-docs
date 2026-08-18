# A Plain File Opens, Edits, And Never Silently Loses A Byte

Given a folder open in the desktop app, when the tree shows a text file that
is neither a `.hick` document nor a woven output of an open document — a
README, a workflow, a justfile — then clicking it opens an editable pane, and
edits save back to the same path on disk; and when the file was rewritten on
disk between the load and the save, then the save is refused and the person
chooses — reload the disk copy or overwrite it — rather than either version
being lost silently.

The folder pane tells the truth about the whole folder, so it must also open
the whole folder. Before this surface existed, every such file rendered as an
inert row; the app could show a repository but not edit it.

Three properties hold it up:

1. **A save rides the hash of what was loaded.** `GET /api/file` answers the
   content and its fingerprint; `PUT /api/file` carries that fingerprint back
   as `base_hash` and is refused with 409 when the disk no longer matches it.
   Saves are strictly serialized client-side, each riding the hash the
   previous exchange established, and a 409 parks the saver until a person
   resolves it — no retry loop, no last-write-wins.
2. **The surface refuses what it cannot honestly edit.** A `.hick` document
   (whose CRDT room owns the file), a path escaping the served root (`..`,
   absolute, or a symlink that resolves outside), non-UTF-8 bytes, and files
   past the size cap are all refused with a message naming the reason and the
   way forward. A read-only file — the up-loop's mark on a fully generated
   output, see [a-generated-file-refuses-an-edit](a-generated-file-refuses-an-edit.md)
   — is refused pointing at the document that produces it.
3. **External edits arrive; they do not clobber.** The pane refetches on
   window focus and on the files-changed announcement, reconciling the disk
   copy into the live buffer with a flash — but never while the pane's own
   edits are unsent, whose save then meets the moved disk as the 409 above.

## Boundary

A writable woven output saved through this surface is not a lineage bypass:
the in-app up-loop sees the write as an external edit and carries it back
into its document, the same path a vim edit takes. Binary files stay inert in
the tree by extension, and the server's UTF-8 check backstops any file the
extension list mislabels.

---

Last LLM verification:
- Date: 2026-08-17
- Reviewer: Claude (Fable 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `crates/hickory-cli/src/serve/plain_file.rs` — `resolve` refuses
  empty/escaping/`.hick` paths, `resolve_existing` canonicalizes against the
  root (symlinks), `read_plain` enforces the 10 MB cap and UTF-8, and
  `write_plain` implements the read-only refusal, the binary refusal, and
  the `base_hash`/`force` contract; routes registered in
  `crates/hickory-cli/src/serve/mod.rs` (`GET`/`PUT /api/file`).
  Client: `apps/web/src/lib/plainFileSave.ts` (serialized saves, hash
  baseline, parked conflict), `apps/web/src/components/PlainFilePane.tsx`
  (editor, conflict banner, focus/files-changed reload guarded by
  `hasPendingEdits`), `apps/web/src/shell/FolderTreePane.tsx::fileAction`
  (tree routing + binary-extension gate),
  `apps/web/src/views/workspaceState.ts::openFileTab` (ensure-open, no
  owning docId).
- Test coverage: `crates/hickory-cli/src/serve/plain_file.rs::tests`
  (round-trip, stale-hash refusal + force, document/escape/binary/missing
  refusals, read-only refusal, symlink escape);
  `apps/web/src/lib/plainFileSave.test.ts` (hash advance, parked 409, both
  resolutions, non-409 re-send); `apps/web/src/shell/FolderTreePane.test.tsx`
  and `apps/web/src/views/workspaceState.test.ts` (tree routing and
  open-adds-never-resets).
- Caveat requiring review: the end-to-end pane behavior (CodeMirror buffer +
  focus-reload + banner) is covered by unit tests of its parts, not by a
  browser-driven system test; and the "two panes on one path are two
  buffers" case relies on the focus-refresh to converge, which no test
  exercises yet.
