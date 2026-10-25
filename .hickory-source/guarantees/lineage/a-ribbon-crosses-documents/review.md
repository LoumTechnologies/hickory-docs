<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Trace a ribbon to the visible part of its source: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/shell/Ribbons.tsx](../../../shared/files/apps/web/src/shell/Ribbons.tsx.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)

## Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../shared/files/apps/web/e2e/literate-editor.spec.ts.md)
- [apps/web/src/lib/ribbons.test.ts](files/apps/web/src/lib/ribbons.test.ts.md)
- [apps/web/src/lib/uiState.test.ts](../../../shared/files/apps/web/src/lib/uiState.test.ts.md)
- [apps/web/src/lib/visibleElementRect.test.ts](files/apps/web/src/lib/visibleElementRect.test.ts.md)
- [apps/web/src/shell/ribbonNavigation.test.ts](files/apps/web/src/shell/ribbonNavigation.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
