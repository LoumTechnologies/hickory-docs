<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Close or recover a document without losing the wrong draft: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/App.tsx](../../../shared/files/apps/web/src/App.tsx.md)
- [apps/web/src/components/PlainFilePane.tsx](../../../shared/files/apps/web/src/components/PlainFilePane.tsx.md)
- [apps/web/src/components/PromptPanel.tsx](files/apps/web/src/components/PromptPanel.tsx.md)
- [apps/web/src/lib/drafts.ts](files/apps/web/src/lib/drafts.ts.md)
- [apps/web/src/lib/outputSave.ts](../../../shared/files/apps/web/src/lib/outputSave.ts.md)
- [apps/web/src/lib/viewBuffers.ts](../../../shared/files/apps/web/src/lib/viewBuffers.ts.md)
- [apps/web/src/lib/windowClose.ts](files/apps/web/src/lib/windowClose.ts.md)
- [apps/web/src/shell/SpatialShellView.tsx](../../../shared/files/apps/web/src/shell/SpatialShellView.tsx.md)
- [apps/web/src/shell/views.tsx](../../../shared/files/apps/web/src/shell/views.tsx.md)
- [apps/web/src/views/RepresentationPane.tsx](../../../shared/files/apps/web/src/views/RepresentationPane.tsx.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [apps/web/src/views/useUnsavedLifecycle.ts](../../../shared/files/apps/web/src/views/useUnsavedLifecycle.ts.md)
- [apps/web/src/views/useUntitledRecovery.ts](files/apps/web/src/views/useUntitledRecovery.ts.md)
- [crates/hickory-cli/src/serve/shell.rs](../../../shared/files/crates/hickory-cli/src/serve/shell.rs.md)

## Tests and fixtures

- [apps/web/src/App.test.tsx](../../../shared/files/apps/web/src/App.test.tsx.md)
- [apps/web/src/components/PlainFilePane.test.tsx](../../../shared/files/apps/web/src/components/PlainFilePane.test.tsx.md)
- [apps/web/src/components/PromptPanel.test.tsx](../../../shared/files/apps/web/src/components/PromptPanel.test.tsx.md)
- [apps/web/src/lib/drafts.lifecycle.test.tsx](files/apps/web/src/lib/drafts.lifecycle.test.tsx.md)
- [apps/web/src/lib/outputSave.test.ts](../../../shared/files/apps/web/src/lib/outputSave.test.ts.md)
- [apps/web/src/lib/windowClose.test.ts](files/apps/web/src/lib/windowClose.test.ts.md)
- [apps/web/src/shell/FolderTreePane.test.tsx](../../../shared/files/apps/web/src/shell/FolderTreePane.test.tsx.md)
- [apps/web/src/shell/ShellView.test.tsx](files/apps/web/src/shell/ShellView.test.tsx.md)
- [apps/web/src/shell/SpatialShellView.test.tsx](../../../shared/files/apps/web/src/shell/SpatialShellView.test.tsx.md)
- [apps/web/src/views/SettingsView.test.tsx](../../../shared/files/apps/web/src/views/SettingsView.test.tsx.md)
- [apps/web/src/views/useUnsavedLifecycle.test.tsx](../../../shared/files/apps/web/src/views/useUnsavedLifecycle.test.tsx.md)
- [apps/web/src/views/useUntitledRecovery.test.tsx](files/apps/web/src/views/useUntitledRecovery.test.tsx.md)
- [crates/hickory-cli/tests/serve_ui_settings.rs](../../../shared/files/crates/hickory-cli/tests/serve_ui_settings.rs.md)
- [crates/hickory-cli/tests/serve_workspace.rs](files/crates/hickory-cli/tests/serve_workspace.rs.md)

## Focused witnesses

- `waits for an unfinished recovery write and never rewrites explicitly discarded work` in `apps/web/src/lib/drafts.lifecycle.test.tsx`.
- `keeps an inactive view open when its attempted Save fails` in `apps/web/src/views/useUnsavedLifecycle.test.tsx`.

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
