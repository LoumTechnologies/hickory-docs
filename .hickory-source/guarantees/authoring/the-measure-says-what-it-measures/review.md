<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Measure Says What It Measures: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/editor/EditorRuler.tsx](../../../shared/files/apps/web/src/editor/EditorRuler.tsx.md)
- [apps/web/src/editor/wrapColumn.ts](files/apps/web/src/editor/wrapColumn.ts.md)
- [apps/web/src/styles.css](../../../shared/files/apps/web/src/styles.css.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [apps/web/src/views/workspaceTabs.tsx](../../../shared/files/apps/web/src/views/workspaceTabs.tsx.md)

## Tests and fixtures

- [apps/web/e2e/startup.spec.ts](../../../shared/files/apps/web/e2e/startup.spec.ts.md)
- [apps/web/src/App.test.tsx](../../../shared/files/apps/web/src/App.test.tsx.md)
- [apps/web/src/editor/EditorRuler.test.tsx](../../../shared/files/apps/web/src/editor/EditorRuler.test.tsx.md)
- [apps/web/src/views/workspaceTabs.test.tsx](../../../shared/files/apps/web/src/views/workspaceTabs.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
