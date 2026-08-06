# A Collaborative Session Never Duplicates the Document

Given a document open in the collaborative editor, when the socket drops and
reconnects, when the last editor leaves and the room is re-created, or when
the server restarts, then the CRDT text still holds exactly one copy of the
document — the server resumes the document's stored operation history rather
than minting a rival copy of `docs.source`, and what is persisted back to
Postgres and git is the same single copy.

---

Last LLM verification:
- Date: 2026-08-06
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence: `apps/server/src/ws.rs::build_room_doc` loads `docs.crdt_state`
  (migration `0003_doc_crdt_state.sql`) and applies it, so a re-created room
  continues the operation history connected clients already hold; the Y.Doc
  is created with `Doc::with_client_id(stable_client_id(doc.id))`, so even
  without stored state two independent seeds of the same source are the same
  operation and dedupe instead of concatenating. Out-of-band changes to
  `docs.source` (REST save, lineage edit, run) are reconciled as a minimal
  prefix/suffix-preserving replace (`text_delta`), never by re-seeding.
  `persist_now` stores text and encoded state under one lock.
  Before this, each room re-creation inserted `docs.source` into a brand-new
  `Doc`; the merge with a returning client's copy concatenated the two. In
  the dev stack one 15 KB document had reached 49 MB — 3268 copies — which
  is what made `GET /api/docs/:id/render` take 7.8 s per call.
- Test coverage: `apps/server/tests/integration.rs` —
  `reconnecting_does_not_duplicate_the_document` drives four real
  connect/sync/disconnect cycles with one long-lived client `Y.Doc` and
  asserts both the client text and the persisted `docs.source` stay exactly
  one copy. Reverting `build_room_doc` to a fresh `Doc::new()` makes it fail
  on round 1 with 2x the bytes.
- Caveats: documents that already contain duplicated text from before this
  fix are not repaired automatically; their source has to be truncated by
  hand.
