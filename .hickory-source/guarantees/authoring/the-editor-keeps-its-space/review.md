<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The editor keeps its space: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/PlainFilePane.tsx](../../../shared/files/apps/web/src/components/PlainFilePane.tsx.md)
- [apps/web/src/components/SurfaceInfo.tsx](../../../shared/files/apps/web/src/components/SurfaceInfo.tsx.md)
- [apps/web/src/editor/CardRail.tsx](../../../shared/files/apps/web/src/editor/CardRail.tsx.md)
- [apps/web/src/editor/chrome.ts](files/apps/web/src/editor/chrome.ts.md)
- [apps/web/src/editor/density.css](files/apps/web/src/editor/density.css.md)
- [apps/web/src/lib/cardRail.ts](../../../shared/files/apps/web/src/lib/cardRail.ts.md)
- [apps/web/src/lib/editorAppearance.ts](files/apps/web/src/lib/editorAppearance.ts.md)
- [apps/web/src/lib/keymap.ts](../../../shared/files/apps/web/src/lib/keymap.ts.md)
- [apps/web/src/shell/CommandBar.tsx](files/apps/web/src/shell/CommandBar.tsx.md)
- [apps/web/src/shell/PaneHeader.tsx](files/apps/web/src/shell/PaneHeader.tsx.md)
- [apps/web/src/shell/SpatialShellView.tsx](../../../shared/files/apps/web/src/shell/SpatialShellView.tsx.md)
- [apps/web/src/shell/commandPalette.css](files/apps/web/src/shell/commandPalette.css.md)
- [apps/web/src/shell/paneHeader.css](files/apps/web/src/shell/paneHeader.css.md)
- [apps/web/src/styles.css](../../../shared/files/apps/web/src/styles.css.md)
- [apps/web/src/views/SettingsView.tsx](../../../shared/files/apps/web/src/views/SettingsView.tsx.md)
- [apps/web/src/views/workspaceTabs.tsx](../../../shared/files/apps/web/src/views/workspaceTabs.tsx.md)

## Tests and fixtures

- [apps/web/e2e/editor-density.spec.ts](files/apps/web/e2e/editor-density.spec.ts.md)
- [apps/web/src/editor/DocumentEditor.test.tsx](../../../shared/files/apps/web/src/editor/DocumentEditor.test.tsx.md)
- [apps/web/src/shell/CommandBar.test.tsx](../../../shared/files/apps/web/src/shell/CommandBar.test.tsx.md)
- [apps/web/src/shell/PaneHeader.test.tsx](files/apps/web/src/shell/PaneHeader.test.tsx.md)
- [apps/web/src/shell/views.test.tsx](../../../shared/files/apps/web/src/shell/views.test.tsx.md)
- [apps/web/src/views/SettingsView.test.tsx](../../../shared/files/apps/web/src/views/SettingsView.test.tsx.md)
- [apps/web/src/views/useFileTabMenu.test.tsx](files/apps/web/src/views/useFileTabMenu.test.tsx.md)
- [apps/web/src/views/workspaceTabs.test.tsx](../../../shared/files/apps/web/src/views/workspaceTabs.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
