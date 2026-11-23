<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Spatial Views Preserve The Editor

For: Authors arranging document windows while preserving their editing position.

Given a workspace, Appearance settings independently enable the left and right
drawers; left defaults on and right defaults off. An enabled drawer appears only
when it has content: a populated Files browser, a relationship to the focused
view, a pending review, or an explicitly pinned view. Unrelated open tabs alone
do not create drawer content. Empty drawers have no handle and reserve no space.
Opening a folder always reveals Files in a pinned left drawer, even for an empty
folder or a restored layout that hid Files. This opening action enables the left
drawer and resets its dragged offset. Other populated drawers start collapsed and
reserve no space. Opening overlays the
editor below its tabs; pinning reserves space beside it. Overlay dismissal by
Escape, close, or a click outside preserves editor buffers and selections.
Drawer pins persist per folder; reopening a folder pins Files on the left again.
Transient open state does not persist.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/shell/spatial-views-preserve-the-editor.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

## Implementation chapters

These chapters keep this behavior's larger implementation readable without duplicating shared code.

- [`apps/web/src/editor/spatialExcerpt.controls.test.tsx`](implementation/apps/web/src/editor/spatialExcerpt.controls.test.tsx.md#output-c25f2be0aa63)
- [`apps/web/src/editor/spatialExcerpt.test.ts`](implementation/apps/web/src/editor/spatialExcerpt.test.ts.md#output-a83150248eb7)
- [`apps/web/src/editor/spatialFiles.ts`](implementation/apps/web/src/editor/spatialFiles.ts.md#output-08858fd1c537)
- [`apps/web/src/shell/SpatialController.test.ts`](implementation/apps/web/src/shell/SpatialController.test.ts.md#output-934bc82e5e77)
- [`apps/web/src/shell/SpatialController.ts`](implementation/apps/web/src/shell/SpatialController.ts.md#output-92c0691b2070)
- [`apps/web/src/shell/spatialDrawers.test.ts`](implementation/apps/web/src/shell/spatialDrawers.test.ts.md#output-97ae5caf5e38)
- [`apps/web/src/shell/spatialDrawers.ts`](implementation/apps/web/src/shell/spatialDrawers.ts.md#output-989018a9b5f9)
- [`apps/web/src/shell/spatialLayout.test.ts`](implementation/apps/web/src/shell/spatialLayout.test.ts.md#output-b5e8db4d0a48)
- [`apps/web/src/shell/spatialLayout.ts`](implementation/apps/web/src/shell/spatialLayout.ts.md#output-8fe693011dbb)
- [`apps/web/src/shell/spatialTiles.test.ts`](implementation/apps/web/src/shell/spatialTiles.test.ts.md#output-ba8df0717d23)
<storycode:include file="implementation/apps/web/src/editor/spatialExcerpt.controls.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/editor/spatialExcerpt.test.ts.md" />
<storycode:include file="implementation/apps/web/src/editor/spatialFiles.ts.md" />
<storycode:include file="implementation/apps/web/src/shell/SpatialController.test.ts.md" />
<storycode:include file="implementation/apps/web/src/shell/SpatialController.ts.md" />
<storycode:include file="implementation/apps/web/src/shell/spatialDrawers.test.ts.md" />
<storycode:include file="implementation/apps/web/src/shell/spatialDrawers.ts.md" />
<storycode:include file="implementation/apps/web/src/shell/spatialLayout.test.ts.md" />
<storycode:include file="implementation/apps/web/src/shell/spatialLayout.ts.md" />
<storycode:include file="implementation/apps/web/src/shell/spatialTiles.test.ts.md" />

## Implementation review

### Implementation

- [apps/web/src/editor/spatialExcerpt.ts](../../../mechanisms/apps/web/src/editor/spatialExcerpt.ts.md#output-6c4319624c14)
- [apps/web/src/editor/spatialFiles.ts](implementation/apps/web/src/editor/spatialFiles.ts.md#output-08858fd1c537)
- [apps/web/src/lib/spatialSides.ts](../../../mechanisms/apps/web/src/lib/spatialSides.ts.md#output-562ade054815)
- [apps/web/src/shell/Ribbons.tsx](../../../mechanisms/apps/web/src/shell/Ribbons.tsx.md#output-b916e85d3b0b)
- [apps/web/src/shell/SpatialController.ts](implementation/apps/web/src/shell/SpatialController.ts.md#output-92c0691b2070)
- apps/web/src/shell/SpatialShellView.tsx
- [apps/web/src/shell/spatialDrawers.ts](implementation/apps/web/src/shell/spatialDrawers.ts.md#output-989018a9b5f9)
- [apps/web/src/shell/spatialLayout.ts](implementation/apps/web/src/shell/spatialLayout.ts.md#output-8fe693011dbb)
- apps/web/src/shell/spatialNavigation.ts
- [apps/web/src/shell/spatialRelations.ts](../../../mechanisms/apps/web/src/shell/spatialRelations.ts.md#output-a9a4742c7a2b)
- apps/web/src/shell/spatialShell.css
- [apps/web/src/shell/spatialTerminals.ts](../../../mechanisms/apps/web/src/shell/spatialTerminals.ts.md#output-f32af4e498bd)
- [apps/web/src/shell/spatialTiles.ts](../../../mechanisms/apps/web/src/shell/spatialTiles.ts.md#output-cdea607538fb)
- apps/web/src/views/WorkspaceView.tsx
- scripts/test-spatial-ui.sh

### Tests and fixtures

- apps/web/e2e/files-editor.spec.ts
- apps/web/e2e/spatialWalkthrough.spec.ts
- apps/web/e2e/spatialWorkspace.spec.ts
- apps/web/e2e/workflow-polish.spec.ts
- [apps/web/src/editor/spatialExcerpt.controls.test.tsx](implementation/apps/web/src/editor/spatialExcerpt.controls.test.tsx.md#output-c25f2be0aa63)
- [apps/web/src/editor/spatialExcerpt.test.ts](implementation/apps/web/src/editor/spatialExcerpt.test.ts.md#output-a83150248eb7)
- [apps/web/src/shell/SpatialController.test.ts](implementation/apps/web/src/shell/SpatialController.test.ts.md#output-934bc82e5e77)
- apps/web/src/shell/SpatialShellView.test.tsx
- [apps/web/src/shell/layout.test.ts](../../../mechanisms/apps/web/src/shell/layout.test.ts.md#output-64b3aead7936)
- [apps/web/src/shell/linkShapes.test.ts](../../../mechanisms/apps/web/src/shell/linkShapes.ts.md#output-04fb400a1dba)
- [apps/web/src/shell/spatialDrawers.test.ts](implementation/apps/web/src/shell/spatialDrawers.test.ts.md#output-97ae5caf5e38)
- [apps/web/src/shell/spatialLayout.test.ts](implementation/apps/web/src/shell/spatialLayout.test.ts.md#output-b5e8db4d0a48)
- [apps/web/src/shell/spatialTiles.test.ts](implementation/apps/web/src/shell/spatialTiles.test.ts.md#output-ba8df0717d23)
- apps/web/src/shell/useShellKeys.test.tsx

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
