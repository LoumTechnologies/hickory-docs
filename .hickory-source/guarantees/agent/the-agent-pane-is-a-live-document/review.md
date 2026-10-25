<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Agent pane is a live document: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/editor/folding.ts](../../../shared/files/apps/web/src/editor/folding.ts.md)
- [apps/web/src/editor/sessionChrome.ts](../../../support/files/apps/web/src/editor/sessionChrome.ts.md)

## Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../shared/files/apps/web/e2e/literate-editor.spec.ts.md)
- [apps/web/e2e/workflow-polish.spec.ts](../../../support/files/apps/web/e2e/workflow-polish.spec.ts.md)
- [apps/web/src/components/ChatDock.test.tsx](../../../shared/files/apps/web/src/components/ChatDock.test.tsx.md)
- [apps/web/src/editor/folding.test.ts](../../../shared/files/apps/web/src/editor/folding.test.ts.md)
- [apps/web/src/views/ConversationEditor.test.tsx](../../../support/files/apps/web/src/views/ConversationEditor.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</book:doc>
