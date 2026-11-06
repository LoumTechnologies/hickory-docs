<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Agent pane is a live document

For: Authors using an agent to change documents while retaining review control.

Given an agent conversation, each message on the selected branch appears in its
own card backed by `DocumentEditor`, the same Markdown/literate editor as a note.
Recorded user messages and recorded or streaming AI messages are read-only,
including paste, undo and programmatic widget edits. The latest **Your response**
card is a separate editable buffer with its own undo history. Streaming changes
preserve the draft and its line breaks; response focus and Send target that buffer.

The initial thread invites a request. Assistant messages use a quiet reading
surface, user messages use a compact bubble, and the editable response stays
anchored at the bottom while scrolling. Reasoning and the final project review
start collapsed; opening them retains their native document tools. New thread is
a direct header action and branch navigation lives in Conversation history.

A successful turn carries its final captured literate construction in a
**Review project changes** disclosure immediately after its messages. This document presents the successful steps from beginning to
end with the final design in mind, beside real implementation and observed checks.
It omits exploratory chronology; rejected alternatives appear only when needed
to explain a decision. Native captured-project run/debug tools remain available
without making the AI record editable. Acceptance proves source/output equality,
not explanatory accuracy or runtime correctness.

**Dictate** is a browser speech-to-text input method for the editable response,
available independently of the chosen agent/provider. It starts only on a click,
inserts final speech at the current selection as undoable text, and shows interim
speech in a compact status field. **Stop dictation** waits for final results before
Send becomes available; dictation never sends a message or speaks an answer.
Switching responses, discarding, closing the pane or a stalled Stop aborts the
recognizer and rejects obsolete replies. Existing typed and finalized text stay
editable and follow the ordinary response-recovery path.

Recognition uses the browser's speech service and language where available; this
is not a promise of local/offline recognition. Unsupported browsers, denied
microphone access, missing microphones and service errors offer typing or system
dictation as a fallback. No audio is attached to the agent request and no voice
agent is introduced. Focused lifecycle tests and a browser transport fixture check
insertion, undo, explicit Send and cleanup with deterministic recognition events;
physical microphone and recognition-quality acceptance remain manual.

For Hickory Python, the response accepts selected files, clipboard files/images
and dropped files. Attachments are removable before Send, image thumbnails allow
inspection, and an attachment-only message can be sent. Preparation must finish
before sending; a failed request retains the selection for retry. Selected images
are converted to PNG at a bounded resolution. Each message accepts at most eight
files and 384 KiB of total decoded attachment bytes (512 KiB base64); image
preparation targets at most 128 KiB per image. Oversized selections explain how to
retry with fewer or smaller files.

**Take picture** requests camera access only after a click, shows a floating
preview and captures a photo after **Use photo**. Cancel, capture and unmount stop
the camera tracks; obsolete capture replies cannot attach to another document.
Camera availability and permission failures offer attaching an existing photo.

The host validates bytes independently before starting a turn. Image bytes enter
the existing immutable artifact/reference path; UTF-8 files enter labeled user
context. Other binary files retain their bytes as labeled base64; this does not
promise PDF/Office extraction or archive execution. Attachments currently require
Hickory Python; a selected model/provider must support images to see a photo.
The subscription Codex/Grok adapters still explicitly refuse image context.
Unsent selections are in memory; the accepted turn records its supplied context.

Focused evidence: the attachment component witness checks exact clipboard bytes,
removal and retention on failure; host unit and real HTTP witnesses reject forged
PNG, invalid encoding, oversized names and oversized payloads. The confined-worker
browser flow exercises file selection, clipboard paste, synthetic camera capture
and the accepted turn. Synthetic camera coverage does not certify physical devices
or model vision quality. Broad regressions remain a separate maintainer check.

Requirement decision: this card-based conversation supersedes the historical
single-buffer wording below. Original captures remain history. Focused component
checks protect independent editor identity, immutable speech, streaming draft
retention, native folding and receipt anchors. The quality of an agent’s chosen
explanation still requires human review; instructions cannot guarantee it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/the-agent-pane-is-a-live-document.md.md). The paragraph ledger records their exact ranges and dispositions.

Historical detail below retains the former single-buffer design. The card-based requirement above governs current work.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/editor/folding.ts](../../../mechanisms/apps/web/src/editor/folding.ts.md#output-eee9d1963829)
- [apps/web/src/editor/sessionChrome.ts](../../../mechanisms/apps/web/src/editor/sessionChrome.ts.md#output-e9d398e40d37)

### Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../mechanisms/apps/web/e2e/literate-editor.spec.ts.md#output-648693644a28)
- [apps/web/e2e/workflow-polish.spec.ts](../../../mechanisms/apps/web/e2e/workflow-polish.spec.ts.md#output-2519c064887c)
- [apps/web/src/components/ChatDock.test.tsx](../../../mechanisms/apps/web/src/components/ChatDock.tsx.md#output-e7bd6d898a44)
- [apps/web/src/editor/folding.test.ts](../../../mechanisms/apps/web/src/editor/folding.ts.md#output-92a9cd526370)
- [apps/web/src/views/ConversationEditor.test.tsx](../../../mechanisms/apps/web/src/views/ConversationEditor.tsx.md#output-edc92cd336d7)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Focused execution (2026-11-04): 59 frontend tests passed across ConversationEditor, ChatDock, ChangesetDocument, linkShapes and coalesceLinks. The final scroll-observation change passed the eight ConversationEditor tests again. Type checking passed; three Rust changeset safety tests and eight Python local-workflow tests passed.
- Browser execution: `scripts/test-literate-editor.sh --grep 'an ACP final claim about the startup Untitled note'` passed against a real isolated engine and ACP fixture. It checks separate read-only cards, the editable Markdown response and the answer’s context ribbon with explicitly selected context and Files windows enabled. No model tokens are spent by this fixture.
- Focused execution (2026-11-05): 41 ConversationEditor and ChatDock tests passed,
  including read-only messages, independent draft undo, streaming, send and stop.
  TypeScript checking passed. The isolated ACP browser flow passed with the
  simplified presentation, retained draft, visible composer and context ribbon.
  Project review disclosures are checked closed by default in the component test.
  Visual review used `/tmp/hickory-agent-pane.png`; aesthetic acceptance remains
  the user's judgment. Node 26 required `NODE_OPTIONS=--no-experimental-webstorage`
  for this repository's jsdom storage setup.
- Evidence limits: final explanation quality remains a human judgment. Broad regressions and post-commit status monitoring remain with the maintainer.

</storycode:doc>
