# Undo Only Undoes What This Client Typed

Given a document open in the collaborative editor, when the user presses undo
repeatedly — past the start of their own edits — then no content arrives at or
disappears from the document that this client did not type: not the initial
sync, not a collaborator's edits, not the mock-mode seed. Undo still fully
undoes this client's own typing.

This failed in the most destructive way available. The CRDT room's first sync
arrives as an ordinary CodeMirror document change, so CodeMirror's own history
recorded "the document appeared" as an undoable local edit. A handful of Ctrl+Z
presses emptied the entire document — and because the editor is collaborative,
that deletion was a legitimate CRDT operation which persisted and propagated to
every other client. The same applied to any edit a collaborator made while the
document was open.

The distinguishing fact is that everything the CRDT applies is dispatched
without a user event, while typing always carries one. Changes without a user
event are kept out of the history.

---

Last LLM verification:
- Date: 2026-08-06
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence: `apps/web/src/editor/DocumentEditor.tsx` — an
  `EditorState.transactionExtender` annotates every document-changing
  transaction that carries no `Transaction.userEvent` with
  `Transaction.addToHistory.of(false)`, which covers remote updates, the
  initial sync, and the mock-mode seed. Found by driving the running app, not
  by reading the code.
- Test coverage: `apps/web/src/editor/DocumentEditor.test.tsx`
  (`undo safety > never erases content this client did not type`) — ten Ctrl+Z
  keystrokes are dispatched at the real key binding (not the command import,
  so the test fails if the keymap is ever wired back to CodeMirror's history);
  the document must be unchanged. The same test then types and undoes, proving
  the fix did not trade a destructive undo for a useless one.
