# The Agent pane is a live document

Given an agent conversation, the Agent pane uses `DocumentEditor`, the same
literate editor as an ordinary note. Recorded turns on the selected branch and
the running turn appear in one buffer, followed by **Your response**, an editable
region. Recorded and streaming content is protected against keyboard edits,
paste, undo, and programmatic widget edits. Only the conversation owner can
replace that prefix. A streaming update preserves the user's response draft,
including its line breaks.

Enter inserts a line. Send or Cmd/Ctrl+Enter submits the response. During a turn,
Stop remains available and cancels the turn through the existing agent stop
route. Stopped turns keep their recorded work. Reasoning remains separate and
foldable. Bookkeeping is expandable, rather than exposed as long JSON lines.

The editor retains the ordinary literate formatting, braces, folds, and line
rails. Receipts from the session are remapped into the selected branch and
registered with the existing ribbon overlay. Hidden work has no visible ribbon
unless the user enables collapsed lineage. The live preview and the response
draft receive no invented receipts. This reading does not save synthetic session
bytes or the draft into the recorded session; the harness remains its writer.

---

Last LLM verification:

- Date: 2026-10-04
- Reviewer: Codex
- Result: partially verified
- Evidence: `ConversationEditor`, `conversationReading`, `protectedPrefix`,
  `sessionBookkeeping`, `DocumentEditor`, `ChatDock`, and `registerLens`.
- Tests: `ConversationEditor.test.tsx` drives actual CodeMirror editors through
  protected edits, streaming updates, multiline drafts, undo, submission,
  selected-branch reads, Unicode offsets, and receipt registration.
  `ChatDock.test.tsx` checks model selection, requests, and stopping;
  the ACP browser flow in `e2e/literate-editor.spec.ts` covers real tools and UI.
- Limits: the browser ACP peer is deterministic; authenticated Codex integration
  was verified for the underlying tools in the earlier review-policy change.
  Desktop-native ribbon geometry has not been inspected for this change.
