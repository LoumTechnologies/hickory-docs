<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Shared print requests do not attest physical output: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/lib/printing.ts](files/apps/web/src/lib/printing.ts.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [apps/web/src/views/uiWorkspaceMenuActions.ts](../../../shared/files/apps/web/src/views/uiWorkspaceMenuActions.ts.md)
- [apps/web/src/views/workspaceMenuActions.ts](../../../shared/files/apps/web/src/views/workspaceMenuActions.ts.md)

## Tests and fixtures

- [apps/web/src/lib/printing.test.ts](files/apps/web/src/lib/printing.test.ts.md)
- [apps/web/src/views/workspaceMenuActions.test.ts](../../../shared/files/apps/web/src/views/workspaceMenuActions.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
