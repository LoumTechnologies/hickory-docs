<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/agent/the-agent-pane-is-a-live-document.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `8e15f39fa0d1c845e95c26bbd073be5882a0a7cddf82d580b3aa876d2a5e9053`.

<book:entry path="docs/guarantees/agent/the-agent-pane-is-a-live-document.md">
# The Agent pane is a live document

Given an agent conversation, the Agent pane uses `DocumentEditor`, the same
literate editor as an ordinary note. Recorded turns on the selected branch and
the running turn appear in one buffer, followed by **Your response**, an editable
region. Recorded and streaming content is protected against keyboard edits,
paste, undo, and programmatic widget edits. Only the conversation owner can
replace that prefix. A streaming update preserves the user's response draft,
including its line breaks.

On first opening a visible conversation editor, the caret starts at the editable
response. The footer's Write a response action returns to the end of the draft
without changing session bytes. Streaming updates do not repeatedly steal focus
or reset the reader's selection.

Both speakers are left-aligned. Enter inserts a line. Send sits on its own line after the
editable draft and is disabled for an empty draft or while a turn starts/runs;
it is a view widget and contributes no document bytes. While a turn runs,
Send is replaced by the persistent Stop control. Send or Cmd/Ctrl+Enter
submits the response. Sending resumes following the response as it grows in the
conversation's single scrollbar. Scrolling upward pauses following; returning
to the bottom resumes it. During a turn,
Stop remains available and cancels the turn through the existing agent stop
route. Stopped turns keep their recorded work. Reasoning remains separate and
foldable. Context, reasoning, tool calls and results collapse through the same
native CodeMirror fold state, gutter and keyboard commands as ordinary notes.
Single-line ACP context folds its body while leaving both tags visible. Unfolded
context remains selectable, searchable editor text. Newly arriving work folds
once; streaming and draft updates do not re-close work the reader opened.

The editor retains the ordinary literate formatting, braces, folds, and line
rails. Receipts from the session are remapped into the selected branch and
registered with the existing ribbon overlay. Hidden work has no visible ribbon
unless the user enables collapsed lineage. The live preview and the response
draft receive no invented receipts. This reading does not save synthetic session
bytes or the draft into the recorded session; the harness remains its writer.

---

Last LLM verification:

- Date: 2026-10-06
- Reviewer: Codex
- Result: verified for the changed conversation controls and formatting
- Evidence: `ConversationEditor`, `conversationReading`, `protectedPrefix`,
  `sessionBookkeeping`, `DocumentEditor`, `ChatDock`, and `registerLens`.
- Tests: `ConversationEditor.test.tsx` drives actual CodeMirror editors through
  protected edits, streaming updates, multiline drafts, undo, submission,
  selected-branch reads, Unicode offsets, receipt registration, inline Send,
  and pausing/resuming scroll following.
  `ChatDock.test.tsx` checks model selection, requests, and stopping;
  the ACP browser flow in `e2e/literate-editor.spec.ts` covers real tools and UI.
- Limits: the browser ACP peer is deterministic; authenticated Codex integration
  was verified for the underlying tools in the earlier review-policy change.
  The Chromium ACP flow verifies left alignment and inline Send in a real
  browser; `/tmp/hickory-agent-inline-send.png` was visually inspected.
  Desktop-native ribbon geometry has not been inspected for this change.

Verification update (2026-10-07): `editor/sessionChrome.ts` adds newly arriving
work to native folds, and `editor/folding.ts` covers single-line ACP records.
`ConversationEditor` refreshes lens receipts after fold-state changes and anchors
opted-in collapsed receipts on their opening line. Tests in
`ConversationEditor.test.tsx` and `folding.test.ts` protect these behaviors;
`e2e/literate-editor.spec.ts` unfolds recorded context in a real Chromium browser.

The pane does not duplicate live ACP tool reports in HTML disclosures; their
recorded context blocks appear in the conversation editor. Permission decisions
remain explicit controls. The change-review strip links directly to the existing
literate comparison editor without a disclosure that hides its navigation.

Verification update (2026-10-13): `ConversationEditor.test.tsx` verifies initial
draft focus and the return-to-response action without source changes.
`e2e/workflow-polish.spec.ts` types into the actual editor immediately after
opening Agent and appends to the draft after using Write a response. The rebuilt
macOS app was also driven through the command palette and its AX state showed
the conversation editor focused. Live provider quality is not claimed.

Verification update (2026-10-17): `responseSend` uses a block widget after
the draft without inserting a newline into session bytes. `ConversationEditor.test.tsx`
checks that Send is outside the draft line and preserves multiline text.
Proposal comparisons now render inside the conversation scroll area through
`AgentChanges` and `DocumentReading`; opening a separate review is optional.

2026-10-17 recovery extension: response text now uses personal workspace
persistence, outside recorded session bytes. See
[Unsent responses survive restart](unsent-responses-survive-restart.md) for
identity, rewind sharing, discard, send acceptance and interruption boundaries.
</book:entry>

</book:doc>
