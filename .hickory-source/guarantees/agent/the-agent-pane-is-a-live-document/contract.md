<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Agent pane is a live document

For: Authors using an agent to change documents while retaining review control.

Given an agent conversation, each message on the selected branch appears in its
own card backed by `DocumentEditor`, the same Markdown/literate editor as a note.
Recorded user messages and recorded or streaming AI messages are read-only,
including paste, undo and programmatic widget edits. The latest **Your response**
card is a separate editable buffer with its own undo history. Streaming changes
preserve the draft and its line breaks; response focus and Send target that buffer.

Code and patch fences in both user and assistant messages offer the same
applicable literate tools as open files on each supported frontend. A person
can explicitly launch runnable code, set breakpoints and step in the original
message coordinates. A patch exposes its authored changes and generated
consequences for review before guarded application. Run, debug and review use
the exact message revision and keep recorded messages immutable.

The initial thread invites a request. Assistant messages use a quiet reading
surface, user messages use a compact bubble, and the editable response stays
anchored at the bottom while scrolling. Reasoning and the final project review
start collapsed; opening them retains their native document tools. New thread is
a direct pane action beside the composer. Conversation history opens an
existing tab for an already-open conversation, allocating a new tab otherwise; message branch actions appear below the
message they retain. A tab keeps its draft and selected ancestry while another
conversation runs. Its bottom status strip owns provider, model, effort and
reported usage for that ancestry. Enter adds a newline; Shift+Enter sends.
Ordinary response Markdown needs no document wrapper or empty project element.

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

- apps/web/e2e/literate-editor.spec.ts
- apps/web/e2e/workflow-polish.spec.ts
- [apps/web/src/components/ChatDock.test.tsx](../../../mechanisms/apps/web/src/components/ChatDock.tsx.md#output-e7bd6d898a44)
- apps/web/src/editor/folding.test.ts
- [apps/web/src/views/ConversationEditor.test.tsx](../../../mechanisms/apps/web/src/views/ConversationEditor.tsx.md#output-edc92cd336d7)
- [macOS Agent editor and buffer witnesses](../../../frontends/apps/apple-simple/Tests/ParityTests.swift.md)
- [iOS recorded message journeys](../../../mechanisms/ios-mounted-parity-acceptance.md#follow-an-immutable-agent-message-into-debug-or-patch-review)
- [Android Agent buffer and request witnesses](../../../frontends/apps/android-simple/app/src/test/java/com/hickorydocs/simple/ParityTest.kt.md)

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
- Focused execution (2026-11-26): the Simple browser lifecycle component witness
  passed with parser loading, immutable user and assistant buffers, an advancing
  answer, retained selection, and independent draft undo. Four focused component
  checks also passed for send, explicit recorded-fence Debug and construction
  patch reading; TypeScript checking passed before the later minimal-edit change.
  The real-engine `scripts/test-simple-browser.py --agent-only` flow passed after
  rebuilding `engine_client` and scoping the source-position observation to its
  View menu. It verifies history, read-only messages, draft retention through a
  refused provider request, subscription selection and the exact HTTP payload.
  Visual inspection used `target/simple-agent.png`; this is not native acceptance.
- Recorded-fence browser execution (2026-11-26):
  `scripts/test-simple-browser.py --recorded-fence-debug-only` passed with the
  rebuilt engine and target-local debugpy 1.8.22. Both user and assistant records
  explicitly run the selected fence, stop at a breakpoint in original message
  coordinates, Step, read a Watch and Continue. The witness compares the entire
  session bytes after execution and confirms retained editor identity. The flow
  opens its saved conversation from History; opening Agent alone creates a new one.
- Recorded-patch browser execution (2026-11-26):
  `scripts/test-simple-browser.py --recorded-patch-only` passed on user and
  assistant messages. Review displays authored changes and generated consequences
  before approval. Apply amends drafts, retains editor identity and supports Undo;
  all saved files and the recorded session remain byte-identical throughout.
- Native verification limit (2026-11-26): Apple and Android now route plain and
  provisional answers through their document editors and retain message buffers
  as host text advances. Focused model witnesses were added but not executed:
  this Linux checkout has no Swift toolchain or Gradle 9.3.1. Source/output
  reconstruction and ownership checks passed; those do not prove native behavior.
- Browser repeat (2026-11-27): the isolated engine passed the Agent history and
  Send journey, recorded Run and debugpy 1.8.22 on both message sides, and both
  recorded patch reviews through draft Apply and Undo. The main browser's
  ConversationEditor and ChatDock passed 52 focused component checks.
- macOS (2026-11-27): Xcode 27 built the application. The answer-replacement
  witness and a mounted AppKit Agent pane passed. The native user and assistant
  views refused insertion. Enter inserted a newline and Shift+Enter invoked
  submit. With marked text, AppKit kept control of the key. An advancing
  answer kept the native editor, selection and viewport. The distinct composer
  kept its draft and could still undo the edit.
- iOS (2026-11-27): the mounted UIKit tests opened each saved conversation and
  used the real engine. The composer accepted native newline input. Both
  recorded views refused direct insertion, replacement, deletion and marked
  text. Selected fences on both sides paused and stepped in original message
  coordinates, evaluated a Watch and continued to completion. The guarded
  patch witness reviewed both sides, checked generated consequences, applied
  and undid the drafts, and preserved the saved fixture and session bytes.
  The rendered app also ran both fences from their native gutters and
  preserved each editor's text and frame. UIKit registered separate Return
  and Shift+Return commands. Dispatching Shift+Return through UIKit invoked
  the submit callback and left a marked candidate with the input method.
  The rendered composer accepted a newline and sent the exact prompt to the
  isolated engine, which archived the user card.
- The separate rendered iOS managed-provider journey reached guarded Apply
  for two source drafts. The relay delivered the live editor capture and the
  engine accepted the draft application report. The replay stopped when
  XCTest could not locate **Undo workspace change** in the Edit menu. The
  mounted recorded-patch tests independently verified draft Undo for both
  message sides.
- Android (2026-11-27): Gradle 9.3.1 assembled the debug APK. Five focused
  parity tests passed for retained message state, draft acknowledgment, exact
  recorded Run and Debug addressing, and message buffers. The separate guarded
  patch test verified the user side's draft Apply and refused Apply after the
  assistant side's authority retired. These checks ran with loopback fixtures.
- Rendered Android (2026-11-27): the API 36 app opened both recorded editors
  from History. Native Run actions selected the first fence on each side and
  produced `42` and `53` with exit code 0 while the session stayed byte-identical.
  The rebuilt terminal filled the measured preview viewport. The native
  composer accepted accessibility input and its Send button sent the exact
  prompt to the owned local engine with HTTP 202. A mounted-activity input
  witness checked the exact draft after Enter and Undo. Shift+Enter sent the
  exact draft to the engine. The fixture returned HTTP 409 on that replay.
- Evidence limits: final explanation quality remains a human judgment. Broad regressions and post-commit status monitoring remain with the maintainer.

</storycode:doc>
