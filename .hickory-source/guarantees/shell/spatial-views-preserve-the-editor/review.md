<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Spatial Views Preserve The Editor: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/editor/spatialExcerpt.ts](../../../support/files/apps/web/src/editor/spatialExcerpt.ts.md)
- [apps/web/src/editor/spatialFiles.ts](files/apps/web/src/editor/spatialFiles.ts.md)
- [apps/web/src/lib/spatialSides.ts](../../../support/files/apps/web/src/lib/spatialSides.ts.md)
- [apps/web/src/shell/Ribbons.tsx](../../../shared/files/apps/web/src/shell/Ribbons.tsx.md)
- [apps/web/src/shell/SpatialController.ts](files/apps/web/src/shell/SpatialController.ts.md)
- [apps/web/src/shell/SpatialShellView.tsx](../../../shared/files/apps/web/src/shell/SpatialShellView.tsx.md)
- [apps/web/src/shell/spatialDrawers.ts](files/apps/web/src/shell/spatialDrawers.ts.md)
- [apps/web/src/shell/spatialLayout.ts](files/apps/web/src/shell/spatialLayout.ts.md)
- [apps/web/src/shell/spatialNavigation.ts](../../../support/files/apps/web/src/shell/spatialNavigation.ts.md)
- [apps/web/src/shell/spatialRelations.ts](../../../support/files/apps/web/src/shell/spatialRelations.ts.md)
- [apps/web/src/shell/spatialShell.css](files/apps/web/src/shell/spatialShell.css.md)
- [apps/web/src/shell/spatialTerminals.ts](../../../support/files/apps/web/src/shell/spatialTerminals.ts.md)
- [apps/web/src/shell/spatialTiles.ts](../../../support/files/apps/web/src/shell/spatialTiles.ts.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [scripts/test-spatial-ui.sh](files/scripts/test-spatial-ui.sh.md)

## Tests and fixtures

- [apps/web/e2e/files-editor.spec.ts](../../authoring/the-workspace-tree-edits-like-an-editor/files/apps/web/e2e/files-editor.spec.ts.md)
- [apps/web/e2e/spatialWalkthrough.spec.ts](files/apps/web/e2e/spatialWalkthrough.spec.ts.md)
- [apps/web/e2e/spatialWorkspace.spec.ts](files/apps/web/e2e/spatialWorkspace.spec.ts.md)
- [apps/web/e2e/workflow-polish.spec.ts](../../../support/files/apps/web/e2e/workflow-polish.spec.ts.md)
- [apps/web/src/editor/spatialExcerpt.controls.test.tsx](files/apps/web/src/editor/spatialExcerpt.controls.test.tsx.md)
- [apps/web/src/editor/spatialExcerpt.test.ts](files/apps/web/src/editor/spatialExcerpt.test.ts.md)
- [apps/web/src/shell/SpatialController.test.ts](files/apps/web/src/shell/SpatialController.test.ts.md)
- [apps/web/src/shell/SpatialShellView.test.tsx](../../../shared/files/apps/web/src/shell/SpatialShellView.test.tsx.md)
- [apps/web/src/shell/layout.test.ts](../../../support/files/apps/web/src/shell/layout.test.ts.md)
- [apps/web/src/shell/linkShapes.test.ts](../../../shared/files/apps/web/src/shell/linkShapes.test.ts.md)
- [apps/web/src/shell/spatialDrawers.test.ts](files/apps/web/src/shell/spatialDrawers.test.ts.md)
- [apps/web/src/shell/spatialLayout.test.ts](files/apps/web/src/shell/spatialLayout.test.ts.md)
- [apps/web/src/shell/spatialTiles.test.ts](files/apps/web/src/shell/spatialTiles.test.ts.md)
- [apps/web/src/shell/useShellKeys.test.tsx](files/apps/web/src/shell/useShellKeys.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
