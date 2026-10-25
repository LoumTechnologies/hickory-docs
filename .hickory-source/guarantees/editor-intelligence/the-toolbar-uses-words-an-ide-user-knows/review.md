<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Run and test from the pane menu without adding a toolbar row: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/ProblemsPanel.tsx](files/apps/web/src/components/ProblemsPanel.tsx.md)
- [apps/web/src/components/RefactorBadge.tsx](../../../shared/files/apps/web/src/components/RefactorBadge.tsx.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [apps/web/src/views/workspaceTabs.tsx](../../../shared/files/apps/web/src/views/workspaceTabs.tsx.md)

## Tests and fixtures

- [apps/web/src/components/ProblemsPanel.test.tsx](files/apps/web/src/components/ProblemsPanel.test.tsx.md)
- [apps/web/src/views/workspaceTabs.test.tsx](../../../shared/files/apps/web/src/views/workspaceTabs.test.tsx.md)

## Focused witnesses

- `disables both buttons while the document's run is in flight` in `apps/web/src/views/workspaceTabs.test.tsx`.

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
