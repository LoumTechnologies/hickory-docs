# A Document's Text Is Never Duplicated Or Silently Rewritten By Sync

Given a document open in one or more editors, when the server rebuilds its
room, reconciles the CRDT against a source rewritten out of band (an
`/outputs/edit` resolved through provenance), or receives client updates, then
the room's text equals the intended text exactly — never the intended text
concatenated with a rival copy of itself, and never spliced at an offset that
lands mid-character. A document that nevertheless exceeds
`MAX_DOC_BYTES` (4 MB) is refused rather than persisted.

Three distinct defects produced the same user-visible symptom — a document that
doubled on every reconnect until the page went blank and froze the tab:

1. **JS-unsafe client ids.** `stable_client_id` derived a 64-bit id from the
   doc UUID. yrs stores client ids as `u64`, but the browser decodes them into
   JS numbers; above 2^53 Yjs throws "Integer out of Range" and discards the
   whole update, so the client never received the document at all.
2. **Client-side seeding against a server-backed room.** The editor seeded its
   `Y.Doc` from the fetched source. With a server behind it the room is already
   built from `docs.source`, so seeding minted an independent copy of the same
   text under a different client id and the CRDT merged the two by
   concatenation — one doubling per connect.
3. **Byte offsets vs UTF-16 offsets.** The reconcile path computes indices in
   UTF-16 code units (Yjs semantics), but yrs defaults to BYTE offsets. On any
   document containing non-ASCII the removal clipped the wrong span and the
   replacement text was spliced mid-character.

Additionally, `/outputs/edit` wrote `docs.source` without telling the live
room, so the room kept serving pre-edit text and its next debounced persist
wrote that stale text back over the edit.

---

Last LLM verification:
- Date: 2026-08-06
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence: `apps/server/src/ws.rs` — `stable_client_id` truncates to 32 bits
  (matching what Yjs itself mints); `new_room_doc` sets
  `OffsetKind::Utf16` so every index in the room agrees with the browser;
  `RoomRegistry::apply_external_source` pushes out-of-band source rewrites
  into the live room and re-persists; `handle_yjs_payload` closes the socket
  without persisting once the text passes `MAX_DOC_BYTES`.
  `apps/web/src/editor/DocumentEditor.tsx` seeds only when
  `realtime.serverAuthoritative` is false (mock mode).
- Test coverage: `apps/server/src/ws.rs` tests —
  `stable_client_id_stays_javascript_safe` (1000 random UUIDs vs
  `Number.MAX_SAFE_INTEGER`), `stable_client_id_is_stable_and_distinct`,
  `room_doc_indexes_text_in_utf16_units`,
  `text_delta_reconciles_non_ascii_documents_exactly`, and
  `reconciling_identical_text_is_a_no_op` (five reconcile rounds over the real
  grand-tour document, asserting no drift). Removing `OffsetKind::Utf16`
  fails the latter two, so the tests bind the fix.
  Live end-to-end against the running server: a connected WebSocket client
  received an `/outputs/edit` rewrite as a CRDT update, its text matched
  `docs.source` afterwards, and the edit survived the room's next persist.
