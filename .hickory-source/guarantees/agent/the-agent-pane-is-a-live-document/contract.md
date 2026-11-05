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
