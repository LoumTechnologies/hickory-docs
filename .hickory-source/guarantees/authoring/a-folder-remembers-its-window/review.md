<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Folder Remembers Its Window: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/desktop/src-tauri/src/folder_session.rs](../../../shared/files/apps/desktop/src-tauri/src/folder_session.rs.md)
- [apps/web/src/lib/uiState.ts](../../../shared/files/apps/web/src/lib/uiState.ts.md)
- [apps/web/src/shell/SpatialShellView.tsx](../../../shared/files/apps/web/src/shell/SpatialShellView.tsx.md)
- [apps/web/src/shell/layout.ts](files/apps/web/src/shell/layout.ts.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [apps/web/src/views/restoreFolderLayout.ts](files/apps/web/src/views/restoreFolderLayout.ts.md)
- [apps/web/src/views/useUnsavedLifecycle.ts](../../../shared/files/apps/web/src/views/useUnsavedLifecycle.ts.md)
- [apps/web/src/views/useUntitledSave.ts](../../../shared/files/apps/web/src/views/useUntitledSave.ts.md)
- [apps/web/src/views/useWorkspaceUi.ts](files/apps/web/src/views/useWorkspaceUi.ts.md)
- [apps/web/src/views/workspaceTabs.tsx](../../../shared/files/apps/web/src/views/workspaceTabs.tsx.md)
- [crates/hickory-workspace/src/lib.rs](../../../shared/files/crates/hickory-workspace/src/lib.rs.md)

## Tests and fixtures

- [apps/desktop/src-tauri/tests/serves_one_origin.rs](../../../shared/files/apps/desktop/src-tauri/tests/serves_one_origin.rs.md)
- [apps/web/src/App.test.tsx](../../../shared/files/apps/web/src/App.test.tsx.md)
- [apps/web/src/shell/SpatialShellView.test.tsx](../../../shared/files/apps/web/src/shell/SpatialShellView.test.tsx.md)
- [apps/web/src/views/folderPane.test.tsx](../../../support/files/apps/web/src/views/folderPane.test.tsx.md)
- [apps/web/src/views/restoreFolderLayout.test.ts](files/apps/web/src/views/restoreFolderLayout.test.ts.md)
- [apps/web/src/views/useUnsavedLifecycle.test.tsx](../../../shared/files/apps/web/src/views/useUnsavedLifecycle.test.tsx.md)
- [apps/web/src/views/useUntitledSave.test.tsx](../../../shared/files/apps/web/src/views/useUntitledSave.test.tsx.md)
- [apps/web/src/views/useWorkspaceUi.test.tsx](files/apps/web/src/views/useWorkspaceUi.test.tsx.md)
- [apps/web/src/views/workspaceTabs.test.tsx](../../../shared/files/apps/web/src/views/workspaceTabs.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
