<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The App Starts As A Lightweight Editor: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/desktop/src-tauri/src/lib.rs](../../../shared/files/apps/desktop/src-tauri/src/lib.rs.md)
- [apps/web/src/App.tsx](../../../shared/files/apps/web/src/App.tsx.md)
- [apps/web/src/lib/focusWhenShown.ts](../../../support/files/apps/web/src/lib/focusWhenShown.ts.md)
- [apps/web/src/lib/newDoc.ts](../../../support/files/apps/web/src/lib/newDoc.ts.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [apps/web/src/views/workspaceState.ts](../../../shared/files/apps/web/src/views/workspaceState.ts.md)
- [apps/web/src/views/workspaceTabs.tsx](../../../shared/files/apps/web/src/views/workspaceTabs.tsx.md)

## Tests and fixtures

- [apps/desktop/src-tauri/tests/serves_one_origin.rs](../../../shared/files/apps/desktop/src-tauri/tests/serves_one_origin.rs.md)
- [apps/web/e2e/startup.spec.ts](../../../shared/files/apps/web/e2e/startup.spec.ts.md)
- [apps/web/src/App.test.tsx](../../../shared/files/apps/web/src/App.test.tsx.md)
- [apps/web/src/views/workspaceState.test.ts](../../../shared/files/apps/web/src/views/workspaceState.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
