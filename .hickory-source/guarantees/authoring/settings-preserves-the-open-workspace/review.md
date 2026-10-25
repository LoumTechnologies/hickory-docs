<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Settings preserves the open workspace: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/App.tsx](../../../shared/files/apps/web/src/App.tsx.md)
- [apps/web/src/lib/workspaceVisibility.ts](../../../support/files/apps/web/src/lib/workspaceVisibility.ts.md)
- [apps/web/src/shell/CommandBar.tsx](../the-editor-keeps-its-space/files/apps/web/src/shell/CommandBar.tsx.md)
- [apps/web/src/shell/ShellView.tsx](../../../support/files/apps/web/src/shell/ShellView.tsx.md)
- [apps/web/src/views/SettingsView.tsx](../../../shared/files/apps/web/src/views/SettingsView.tsx.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [apps/web/src/views/useWorkspaceAppearance.ts](../../../support/files/apps/web/src/views/useWorkspaceAppearance.ts.md)

## Tests and fixtures

- [apps/web/e2e/workflow-polish.spec.ts](../../../support/files/apps/web/e2e/workflow-polish.spec.ts.md)
- [apps/web/src/App.test.tsx](../../../shared/files/apps/web/src/App.test.tsx.md)
- [apps/web/src/views/useWorkspaceAppearance.test.tsx](../../../support/files/apps/web/src/views/useWorkspaceAppearance.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
