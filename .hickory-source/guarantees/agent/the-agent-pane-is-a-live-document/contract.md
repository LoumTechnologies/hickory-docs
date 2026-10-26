<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Agent pane is a live document

For: Authors using an agent to change documents while retaining review control.

Given an agent conversation, the Agent pane uses `DocumentEditor`, the same
literate editor as an ordinary note. Recorded turns on the selected branch and
the running turn appear in one buffer, followed by **Your response**, an editable
region. Recorded and streaming content is protected against keyboard edits,
paste, undo, and programmatic widget edits. Only the conversation owner can
replace that prefix. A streaming update preserves the user's response draft,
including its line breaks.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/the-agent-pane-is-a-live-document.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

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
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
