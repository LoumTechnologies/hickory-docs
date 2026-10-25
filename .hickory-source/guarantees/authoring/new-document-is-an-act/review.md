<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# New Document Creates Another Unnamed Buffer: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/lib/editorBufferTargets.ts](../../../support/files/apps/web/src/lib/editorBufferTargets.ts.md)
- [apps/web/src/views/WorkspaceChat.tsx](../../agent/unsent-responses-survive-restart/files/apps/web/src/views/WorkspaceChat.tsx.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [apps/web/src/views/uiWorkspaceMenuActions.ts](../../../shared/files/apps/web/src/views/uiWorkspaceMenuActions.ts.md)
- [apps/web/src/views/useNewDocument.ts](../../../support/files/apps/web/src/views/useNewDocument.ts.md)
- [apps/web/src/views/useUnsavedLifecycle.ts](../../../shared/files/apps/web/src/views/useUnsavedLifecycle.ts.md)
- [apps/web/src/views/useUntitledRecovery.ts](../unsaved-work-survives-closing-the-app/files/apps/web/src/views/useUntitledRecovery.ts.md)
- [apps/web/src/views/useUntitledSave.ts](../../../shared/files/apps/web/src/views/useUntitledSave.ts.md)
- [apps/web/src/views/workspaceMenuActions.ts](../../../shared/files/apps/web/src/views/workspaceMenuActions.ts.md)
- [apps/web/src/views/workspaceState.ts](../../../shared/files/apps/web/src/views/workspaceState.ts.md)
- [crates/hickory-cli/src/serve/api.rs](../../../shared/files/crates/hickory-cli/src/serve/api.rs.md)
- [crates/hickory-cli/src/serve/api/create_new.rs](files/crates/hickory-cli/src/serve/api/create_new.rs.md)

## Tests and fixtures

- [apps/web/src/lib/newDoc.test.ts](files/apps/web/src/lib/newDoc.test.ts.md)
- [apps/web/src/router.test.ts](../../../shared/files/apps/web/src/router.test.ts.md)
- [apps/web/src/views/uiWorkspaceMenuActions.test.tsx](../../../shared/files/apps/web/src/views/uiWorkspaceMenuActions.test.tsx.md)
- [apps/web/src/views/useUnsavedLifecycle.test.tsx](../../../shared/files/apps/web/src/views/useUnsavedLifecycle.test.tsx.md)
- [apps/web/src/views/useUntitledSave.test.tsx](../../../shared/files/apps/web/src/views/useUntitledSave.test.tsx.md)
- [apps/web/src/views/workspaceMenuActions.test.ts](../../../shared/files/apps/web/src/views/workspaceMenuActions.test.ts.md)
- [apps/web/src/views/workspaceTabs.test.tsx](../../../shared/files/apps/web/src/views/workspaceTabs.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
