-- Durable Yjs state per document.
--
-- Collaboration rooms are created on the first socket and dropped when the
-- last one leaves. Re-creating a room used to build a brand-new Y.Doc and
-- insert `docs.source` into it, minting fresh CRDT operations with a fresh
-- client id every time. A returning client still held the *previous*
-- room's operations, so the merge concatenated both copies: every
-- reconnect appended the whole document to itself, and the debounced
-- persist wrote the result back. One document reached 49 MB (3268 copies).
--
-- Keeping the encoded document state means a re-created room resumes the
-- same operation history instead of minting a rival one.

ALTER TABLE docs ADD COLUMN crdt_state BYTEA;
