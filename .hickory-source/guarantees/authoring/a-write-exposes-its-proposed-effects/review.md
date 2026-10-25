<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A write exposes its proposed effects: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/ChangePreview.tsx](files/apps/web/src/components/ChangePreview.tsx.md)
- [apps/web/src/components/InsertMenu.tsx](../../../shared/files/apps/web/src/components/InsertMenu.tsx.md)
- [apps/web/src/components/NewProjectDialog.tsx](../../../shared/files/apps/web/src/components/NewProjectDialog.tsx.md)
- [apps/web/src/environments/EnvironmentPanel.tsx](../../../shared/files/apps/web/src/environments/EnvironmentPanel.tsx.md)
- [apps/web/src/shell/FilesystemTreeEditor.tsx](../../../shared/files/apps/web/src/shell/FilesystemTreeEditor.tsx.md)
- [apps/web/src/views/DocumentReading.tsx](../../../shared/files/apps/web/src/views/DocumentReading.tsx.md)
- [apps/web/src/views/RepresentationPane.tsx](../../../shared/files/apps/web/src/views/RepresentationPane.tsx.md)

## Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../shared/files/apps/web/e2e/literate-editor.spec.ts.md)
- [apps/web/src/components/InsertMenu.test.tsx](../../../shared/files/apps/web/src/components/InsertMenu.test.tsx.md)
- [apps/web/src/components/NewProjectDialog.test.tsx](../../../shared/files/apps/web/src/components/NewProjectDialog.test.tsx.md)
- [apps/web/src/environments/EnvironmentPanel.test.tsx](../../../shared/files/apps/web/src/environments/EnvironmentPanel.test.tsx.md)
- [apps/web/src/views/workspaceTabs.test.tsx](../../../shared/files/apps/web/src/views/workspaceTabs.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
