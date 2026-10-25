<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Open File Tabs Follow Renames: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/shell/FolderTreePane.tsx](../../../shared/files/apps/web/src/shell/FolderTreePane.tsx.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [apps/web/src/views/workspaceState.ts](../../../shared/files/apps/web/src/views/workspaceState.ts.md)

## Tests and fixtures

- [apps/web/src/views/workspaceState.test.ts](../../../shared/files/apps/web/src/views/workspaceState.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</book:doc>
