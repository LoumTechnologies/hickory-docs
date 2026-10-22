# Spatial Views Preserve The Editor

Given a workspace, Appearance settings independently enable the left and right
drawers; left defaults on and right defaults off. An enabled drawer appears only
when it has content: a populated Files browser, a relationship to the focused
view, a pending review, or an explicitly pinned view. Unrelated open tabs alone
do not create drawer content. Empty drawers have no handle and reserve no space.
Populated drawers start collapsed and reserve no space. Opening overlays the
editor below its tabs; pinning reserves space beside it. Overlay dismissal by
Escape, close, or a click outside preserves editor buffers and selections.
Drawer pins persist per folder; transient open state does not.

The workspace retains two independently mounted Files buffers even when their
drawers are absent or closed. File → Show Files can explicitly open an empty
folder's browser. Opening a file from Files centers it and dismisses an unpinned
drawer. Center tabs keep ordinary files, documents and conversations available
independently of the drawers. Each Files buffer remains the ordinary CodeMirror tree
buffer, including its caret, selections, folding, keyboard editing, Dry Run,
Apply and filesystem operations. A preview of a listed file uses the filename
line in that buffer as its vertical tab, with contextual color for flowing and
pinned views. Its name is selectable and editable text. A small editor gutter
grip carries drag and pin actions, leaving text selection alone. Line decorations place the actual filename text beside its preview independently
of its logical tree order. Native block decorations reserve neighboring ordinary
rows and enough editor height for the moved text. They never rewrite tree bytes
to record a pin or move. Views without a visible filename
row retain a title header.

Dragging the Files toolbar moves the Files list and its preview lane as one
surface, clamped within the workspace. Pins travel with that surface. Position
persists per folder. Canceling the gesture restores position and view pins.
Dragging a preview snaps to measured Files rows and the editor's line spacing;
alignment places its real filename row beside the window. Pinned obstacles constrain placement, and a drag does not rename or move a
filesystem entry.

Given an open document, agent conversation, generated file, or other tool
view, Center makes that view the central reading surface. Restore returns to
the previous center. Changing presentation keeps each tab's editor mounted,
with its buffer, selection, editor tools, native gutters and line addresses.
The Agent pane uses the same placement mechanism as other views; its own
session and permission controls remain its document's responsibility.

Each center tab has its own close icon and uses the existing unsaved-change
choice. Closing an inactive tab leaves focus on the current pane. Pin actions
use pushpin icons with accessible names and pressed state, including drawer
pins. Following a ribbon expands its destination in the side it points toward;
an existing center tile stays central, and a disabled side falls back to the
available side or the center when both sides are disabled.

Dragging a window header or center tab previews its destination. Dropping in
the middle of the center selects that view; dropping at a tile's left, right,
top or bottom edge splits that tile, allowing nested arrangements. Closing or
moving a tile reclaims its old space. The arrangement persists per folder.
Moving between drawers and center tiles keeps the same mounted editor.

Given a side view, a view pin freezes its placement within the drawer and pins
that drawer. The center's **Pin view in drawer** action moves the view beside a
different open tab without cloning its editor. A pin freezes presentation until the
person drags, resizes, minimizes or centers that view, or changes the available
viewport, drawer geometry or column widths. Pin state does not grant permission to edit bytes.
Scrolling a source, changing contextual content, and aligning Files rows never
change an existing pin's rectangle. Filename alignment follows the window's
screen rectangle and never pulls a flowing window back to its original text row.
A flowing filename can visually pass a pinned filename without changing their
logical tree order. Initial pin placement belongs
to the explicit pin action; later paints cannot write a Files row's displacement
back into an existing pin.
Dragging previews a tentative rectangle and dropping commits that rectangle.
Dragging or resizing an unpinned view pins it. Both resize edges are available;
growth uses free space, then shifts the adjacent pin into available space,
then reduces that neighbor only when the next pin or viewport blocks a shift.

Given related content in a center tile, visible drawer views follow its real
provenance anchors with parallax. The alignment band and speed use that source
tile's screen position and viewport height, including a second center tile.
Moving Files sideways does not replace the source coordinate with a drawer
coordinate. In the central alignment band they use the source's
vertical coordinate without a speed offset. A default side view is collapsed
to its filename; explicit opening/reveal requests can expand it. Optional
automatic accordion behavior expands views away from the viewport edges.
Expanded context excerpts hide lines with up/down controls over the unchanged
buffer, independently of language folding. Center shows the full buffer. A
pinned view also shows the full editable buffer, never an excerpt: the excerpt
is a reading window for a flowing preview, and a person pinning a document is
asking to read and edit it in place. Expanded and revealed side views stay
inside their lane — an opening/reveal request, a pin, or an automatic accordion
expansion is answered where it can be seen, never beyond the lane's scroll
area. Collapsed filename views may still flow off the lane with their source.

Given pinned obstacles, placement reserves their rectangles. Once a moving
view crosses an obstacle's edge, a permitted bypass animation runs to completion
even if scrolling stops. Its source and destination slots remain reserved while
it runs. When a clear bypass is unavailable or reduced motion is requested,
placement uses the final clear slot directly. Narrow overlays fit within the
center; drawer pins keep a nonzero center width. The workspace itself does not add
an outer scrollbar to its native editor scrollports.

Given a folder collapsed in a Files buffer, unpinned descendant side views on
that side disappear. A provenance endpoint may terminate at the visible
collapsed ancestor folder. This is a navigation address; it neither invents
nor changes the recorded provenance. Pins remain available. Filenames in the
extended headers use paths relative to the open folder; unnamed and tool views
use their ordinary titles.

---

Drawer continuation (2026-10-12): `spatialDrawers.ts` derives contextual content
and overlay/pinned geometry. Empty and collapsed drawers reserve no width.
`SpatialShellView` retains all editors and differentiates background pending
reviews from explicit requests to open a review. Pins from older layouts migrate
to drawer pins. `just test-spatial` passed type checking and 41 tests. The native
`just test-spatial-ui` recipe passed four browser flows covering overlay and
pinned rectangles, Files drafts, reload, native editor/selection identity,
drag/resize, Center/Restore and narrow layouts. This replaces the pre-drawer
browser witnesses described in the historical review below. Production native
WebView acceptance and arbitrary multi-obstacle animation remain unverified.
The full frontend suite passed 218 files / 2,087 tests; the production build and
the native Python-agent review/recovery/cancellation browser flow also passed.
Final focused and native-drawer checks passed again after the keyboard-focus and
accessibility adjustments. Browser screenshots of collapsed, overlay and pinned
states were inspected; this is implementation review, not recorded user approval.

Appearance continuation (2026-10-12): `lib/spatialSides.ts` stores independent
left/right preferences with left-on/right-off defaults. The shell retains hidden
editors and the controller removes hidden sides from placement and geometry.
Center tabs still reach those views; hidden-side reveal requests center them.
`just test-spatial` passed type checking and 33 tests, including settings
persistence, corrupt storage, desktop/narrow geometry, editor preservation while
toggling sides, and hidden-side reveal. Five link-shape regression tests also
passed. The optional bilateral browser geometry suite now explicitly enables
both sides; it was not rerun for this continuation. Native visual judgment is
not recorded.

Last LLM verification:

- Date: 2026-10-09
- Reviewer: Codex
- Result: partially verified; desktop Chromium exercises real editors and a
  scratch native engine. Native macOS WebView, very large Files buffers and
  arbitrary multi-obstacle animation sequences still need broader coverage.
- Originating requirement: the user's navigation prototype and request to
  apply its layout and animations to Hickory while retaining native Files
  buffers, line numbers and gutters. The prototype was committed as
  `cc1bb06` before integration.
- Evidence: `apps/web/src/shell/SpatialShellView.tsx`, `SpatialController.ts`,
  `spatialLayout.ts`, `spatialRelations.ts`, `spatialTerminals.ts`,
  `apps/web/src/editor/spatialFiles.ts`, `spatialExcerpt.ts`, and the integration
  in `apps/web/src/views/WorkspaceView.tsx`. `Ribbons.tsx` redraws on spatial
  frames and uses visible editors or native terminal geometry.
- Assumptions: the viewport and column width are presentation constraints,
  so changing them can reconcile saved pins into clear available slots.
  Source identity remains the existing path/span evidence, not a new element
  id. Presentation storage is optional local state keyed to the open folder.
  A rendered Markdown widget has its existing cursor mapping; geometry tests
  use an explicit text selection, independently from widget cursor behavior.
- Tests: `spatialLayout.test.ts` checks clear slots, central alignment,
  bidirectional shift-before-shrink, tab identity and corrupt presentation
  storage. `SpatialShellView.test.tsx` checks editor identity across Center and
  Restore. `spatialExcerpt.test.ts` checks expandable context without byte or
  address changes. `e2e/spatialWorkspace.spec.ts` checks real pin/unpin geometry,
  exact drag drop, top and bottom resize, native editor state, two Files buffers,
  viewport overflow and frame-by-frame collision samples. The live Files
  round trip remains `e2e/files-editor.spec.ts`.
- Challenges: browser automation must wait for the actual layout frame before
  using a moved resize handle; otherwise the next synthetic click can land in
  editor content instead of the handle. Tests distinguish React state from
  the measured rectangle. Universal animation non-overlap is not established
  by the finite frame sample alone.
- Dependencies: inspected working tree based on `cc1bb06`; changes to native
  editor lifecycle, Files projection, provenance registration, viewport
  geometry or the collision planner make this review stale.
- Executed checks: `just test-web` passed 211 files / 2,046 tests before the
  final reverse-link regression; `just test-spatial` passed 8 tests, and the
  targeted link/navigation suite passed 14 tests including that regression.
  `just test-spatial-browser` passed all 5 live-engine scenarios;
  `just test-files-browser` passed rename/create/delete against a throwaway
  engine. `just build-web` passed. `just test-browser-embedding` passed all
  33 scenarios across Chromium, Firefox and WebKit. Final commit hooks run
  the repository's affected checks separately.
- Human judgment: the user approved the prototype's direction before this
  integration; production-app visual acceptance has not yet been recorded.

Verification update (2026-10-13): a search result opened from Agent now follows
the focused tab's identity as well as its pane. Restore previous center also
restores layout focus, so command actions target that view. The real-engine
flow in `e2e/workflow-polish.spec.ts` reproduces search → centered note → Restore
→ Agent and verifies that document Test is unavailable while Agent is focused.
The existing spatial suite passed after the change (5 files / 42 tests including
the Settings preferences). Editor mounts and source bytes remain unchanged.


Tab and docking continuation (2026-10-14):

- Originating requirement: the user requested working tab close icons, ribbon
  destinations opening where their connections point (usually the parallax
  area), pushpin icons instead of Pin/Unpin text, and dragging windows/tabs
  between side areas and central IDE-style tiles.
- Interpretation and assumptions: placement is presentation state, independent
  of editor ownership. A center drop selects the view; an edge drop splits the
  destination into equal tiles. Nested splits are supported. The primary tab
  occupies a stable slot while neighboring tiles retain their own editors.
  Disabled side settings remain authoritative. No new file writes are needed.
- LLM implementation review: inspected `SpatialShellView` close, ribbon request,
  fresh-tab, focus and dock paths; `SpatialController.begin/move/finish/paint`;
  `spatialTiles` pruning, splitting and rectangle calculations;
  `requestSpatialRibbonOpen`; and `layout.closeTab`. Implementation supports
  the stated behaviors. Broader native-WebView acceptance remains unverified.
- Independent witnesses: `SpatialShellView.test.tsx` checks inactive-tab closing,
  existing and newly opened ribbon targets, icons, and disabled-side fallback.
  `spatialTiles.test.ts` checks explicit nested rectangles and moving/removing
  tiles; `layout.test.ts` checks focus after another pane closes. The native
  engine browser flow in `e2e/spatialWorkspace.spec.ts` closes an inactive tab,
  drags a side window into the center, splits by dragging a tab, pins a tile
  back into a drawer, and checks editor instance, text and selection identity.
- Challenges: the first browser close witness exposed `closeTab` selecting
  Files after removing an inactive pane; focus now stays on the surviving
  focused pane. Full-suite startup checks exposed duplicate center close
  controls; only the tab's X remains for the center. Final review found that
  tile pins needed to remove the tile from the center before placing it in a
  drawer; the final browser round trip exercises that correction.
- Executed checks: `just test-web` passed 222 files / 2,106 tests after the
  close-control correction. After the final side-fallback and tile-pin changes,
  `just test-spatial` passed type checking and 58 tests and
  `just test-spatial-ui` passed all six real-engine browser flows.
  `just check-file-length` and `git diff --check` passed. The center-tile
  screenshot was inspected. Actual SVG-ribbon clicks across every provenance
  family are not covered by the new browser flows; ribbon placement is covered
  through the request/reveal integration tests. Arbitrary deep/narrow tiling,
  cross-window dragging and native macOS WebView were not exercised.
- Human judgment: no new user acceptance is recorded. Tests and screenshot
  inspection are implementation evidence, not user approval.
- Evidence dependencies (SHA-256; changing these invalidates this review):
  `SpatialShellView.tsx` = `8f7b95a286d5a579500a050453a7bdf6b4f81921f8288d5e67cde19c84ac87bf`;
  `SpatialController.ts` = `1d2909bad8ae3c80a39707b17e81fd4a9315bf20a85814b68529a1e3086627fd`;
  `spatialTiles.ts` = `b3606e6d07b6323f5fc727dc52fa1e4090d1059e83f5c08ce4967e8f6b37042e`;
  `spatialNavigation.ts` = `aad7a5ae9fff5d4fee3a43a1c1006e80bba208026d56798ade5afa77ff043b0e`;
  `layout.ts` = `d0a254f01ee8228955f5734e2e409abd204ad5fd2e7c979daf0d319510e23039`.
  These implementation files are under `apps/web/src/shell/`.


## Files text-row continuation (2026-10-14)

Originating requirement: the user's request for a draggable Files surface that
combines the editable dired list, ribbon previews and vertical tabs; filename
headers should be ordinary contextual text, and pinned previews should snap to
text lines. Interpretation: use the existing filename bytes as the tab, decorate
them and place them with CodeMirror block spacing, rather than making another
filename button. The gutter grip separates dragging from normal selection.

LLM implementation review: `editor/spatialFiles.ts` adds line decorations,
gutter grips and measured snapping; `SpatialController.paintHeaders` aligns the
actual text row in the same paint before exposing the final rectangle.
`beginFiles`/`move`/`finish` translate the list, preview lane and pins together
and restore them on pointer cancellation. `spatialDrawers.workspaceRegions`
clamps the whole surface; `spatialLayout.readSpatialState` validates persisted
coordinates. `SpatialShellView` leaves text pointer events to CodeMirror and
routes grip gestures to the placement controller. Context discovery includes
all center tiles and `related` reads the actual source tile's geometry.

Executed checks: `just test-spatial` passed type checking and 61 tests;
`just test-spatial-ui` passed all seven native-engine browser flows, including
filename editing/undo. `just test-files-previews` passed the final focused checks
(23 tests) for the Files editor, placement, screen-edge cancellation and themed
tooltip convention. `just check-file-length` and `git diff --check` passed.
The broader `just test-web` run passed 2,109 tests and failed the themed-tooltip
source scan because the new grip initially used `title`. That was replaced with
`dataset.tip`; the exact failed test then passed in `test-files-previews`. The
whole broader suite was not repeated after that correction. The first native
browser runs exposed a paint-timing race: a pin could be visible before its
filename row settled, so a drag could start at the old position. Alignment now
measures the updated row in the same paint and records that pin position.
Earlier passing browser evidence alone did not rule out this race.

Witnesses: `e2e/spatialWorkspace.spec.ts` checks a real native engine, decorated
filename bytes, hidden duplicate strip, movement of the whole Files surface,
actual filename/preview alignment after a grip drag, filename editing and undo,
editor instance/selection preservation and restored position after reload.
`SpatialController.test.ts` checks a receipt originating in a lower center tile,
then moves Files and the source anchor independently; `spatialDrawers.test.ts`
checks offscreen clamping, stored offsets and second-tile context discovery.

Assumptions and coverage gaps: Files row order remains the person's filesystem
reading; this change does not reorder directories or invent path identities.
Where no filename row is visible (unnamed/tool views, unlisted paths and folded
ancestors), the existing title/ancestor endpoint remains. The browser witness
covers one pinned file; many simultaneous previews, very large virtualized Files
buffers, touch gestures and visual taste still need broader evidence. Floating
a pinned drawer keeps its reserved center width until it is unpinned.

Human judgment: no review of the final visual treatment recorded. Implementation
and executed checks do not establish that its feel matches the user's prototype.

Evidence dependencies (SHA-256 of the inspected working-tree bytes; edits to
these files make this review stale):

- `apps/web/src/editor/spatialFiles.ts`: `266e213e1337152f853df9b3526951e4cafb879fe8fc6ccbc7a7b4efaf703fd6`.
- `apps/web/src/shell/SpatialController.ts`: `bc3ca6157b1b58553589c530fbad31233dbdec9f4b15f7c7dc92b2d3ba084c31`.
- `apps/web/src/shell/SpatialShellView.tsx`: `aa1be7bc5d1586caa8a25a811969b4c0050c4499b7cd3b8f149a59cd8d29dc71`.
- `apps/web/src/shell/spatialDrawers.ts`: `e015e08abddcb86e24c71fb2e8c96239613c90bfe20c94f0909ca942bf6eea79`.
- `apps/web/src/shell/spatialLayout.ts`: `ed9b6af4ff975a29d331981f67faf12ec11e787850c03fa666c368c9b358d7cd`.
- `apps/web/src/shell/spatialShell.css`: `90883a47ef4c8a63523ca4ffb91750b44c1df36fe430ca6ae7053f789dba7310`.
- `apps/web/src/shell/SpatialController.test.ts`: `17dd75ff7642959a390a397c8d702c081169d3b8d09d925ab2faeeeba7e389c6`.
- `apps/web/src/shell/spatialDrawers.test.ts`: `17fabcc1860785c7cf23dcfc08accfd76bdcff4eb9c0c6ad6c76f1e3f95d530b`.
- `apps/web/e2e/spatialWorkspace.spec.ts`: `3c8ca7a72fb6deace52927c6ff9b0aa2152a15514c752af7a585e10ec4bc8255`.


## Pin stability correction (2026-10-14)

Originating requirement: the user observed that scrolling an unpinned preview
around a pinned window pushed the pin down. Existing pin placement must be an
obstacle to the flowing preview, not an output of its filename spacing.

Correction to the preceding implementation review: it covered a single pin but
missed feedback from a preceding flowing filename. `paintHeaders` had written
the displaced pinned filename's coordinate into the saved pin. Even when the
window placements cleared each other, the Files block gaps could grow before
that filename. Earlier passing single-pin browser checks did not exercise this.

LLM implementation review: `alignSpatialFileRows` now computes undecorated row
coordinates before choosing gaps and caps a flowing gap by the available space
before the next pinned filename. `SpatialController.paintHeaders` preserves
existing pin rectangles; only a one-paint `alignNewPin` request from an explicit
pin action can settle a newly pinned view. The geometry-change signature uses
actual lane rectangles, not the changing contextual-item inventory.

Executed checks: `just test-spatial` passed type checking and 62 tests;
`just test-spatial-ui` passed all eight native-engine browser flows. The new
“scrolling an earlier Files preview around a pin never moves the pin” witness
opens two real files, pins the later Files entry, reveals the earlier one and
scrolls a third editor repeatedly. It asserts the pin's full rectangle stays
identical, the flowing window changes vertical position and its rectangle stays
clear of the pin. The existing drag, snap, resize, floating Files, edit/undo and
reload checks also passed. `git diff --check` and `just check-file-length` passed.

Coverage limits: the new browser witness drives an explicitly revealed preview
using source-scroll placement; source-ribbon geometry is independently covered
by `SpatialController.test.ts`. It does not establish every arrangement of many
pins or large virtualized trees. Human judgment: no subsequent visual review
recorded. The previous hashes remain historical; the changed dependencies for
this correction are:

- `apps/web/src/editor/spatialFiles.ts`: `43908ae7d05f3dfc02523b41d6c22b911e48dcc6e3154a6f104afd5365592159`.
- `apps/web/src/shell/SpatialController.ts`: `630c47ec4cd6b1ef382a5284309596808a92d2493e7e17d49601c716eb7ccfce`.
- `apps/web/src/shell/SpatialShellView.tsx`: `2717df7f064ff3517ba25373af8d9e837c5d84d475c2d12a0aa094a0e02b0cc7`.
- `apps/web/e2e/spatialWorkspace.spec.ts`: `2954f93ad00d63aab367dfce48d62539e18bdbe2c8eb4207f7a0cc63daecd779`.


## Bypass text/window correction (2026-10-14)

Originating requirement: the user reported that a bypass began but the filename
line stayed still and the window failed to reach its destination, specifically
in `/Users/loumtech/Documents/Hickory walkthrough wbt9rl_0`.

Retrospective correction: the preceding pin-stability fix capped positive gaps
before a pin, which cannot place an earlier filename after a later filename.
It also left flowing windows subject to their constrained row coordinate during
header painting. The earlier regression checked pin stability and changing
window coordinates, but did not check the filename or animation completion.
The historical gap-only evidence above is superseded on this mechanism.

LLM implementation review: `alignSpatialFileRows` now gives the actual filename
line a visual translation independent of tree order. It subtracts the previous
translation and block spacing before computing new placements, reserves normal
rows around preview names, and reserves editor height for translated final
rows. Gutter grips follow the same translation. `previewMouseSelection` uses
browser caret hit-testing so a click/edit targets the displayed filename rather
than the old height-map coordinate. The existing Alt/column-selection provider
continues to own its gesture.

`SpatialController.paintHeaders` no longer derives a flowing window rectangle
from Files text. During a bypass it samples the browser's animated window each
frame and passes that coordinate to the filename decoration. When the route
finishes, both retain its target. Flowing windows with visible Files rows stay
within the editor's readable vertical area. A new pin leaves room for a collapsed
preview above it when available; existing pins are unchanged. Listed filenames
never regain an overlapping duplicate header just because their row is clipped.

Executed checks: `just test-spatial` passed type checking and 62 tests;
`just test-files-previews` passed 23 focused tests; `just test-spatial-ui` passed
all eight native-browser flows after strengthening the scrolling case to check
both filename and window positions and the editing case to check the exact
filename edited. `just test-spatial-walkthrough` runs the native engine against
a disposable copy of the user's walkthrough. Its test explicitly observes a
bypass, samples matching text/window coordinates during that animation, checks
movement over 100 pixels, checks the destination after scrolling stops, keeps
the pin's complete rectangle fixed, and edits/undoes the moved `app.py` filename.
Final walkthrough result: one browser flow passed, including animation-frame
alignment and exact moved-filename editing. `just check-file-length` and
`git diff --check` also passed. The original walkthrough is read
as a fixture; modifications and generated evidence are confined to its copy.

The added editing witness initially failed because CodeMirror's default pointer
hit-test used logical row height. The browser caret mapping fixed that failure;
window/filename animation checks alone had not established click correctness.
A smaller fixture also exposed insufficient room above a new pin, covered by
the initial-placement adjustment and the strengthened ordinary scrolling test.

Coverage gaps: one pinned window with repeated bypasses in a native walkthrough
and one smaller fixture are exercised. Large virtualized trees, many simultaneous
routes, touch selection, and rectangular selection across visually reordered
rows remain unverified. Human judgment: no additional review recorded.

Current dependencies for this correction (SHA-256; earlier hashes above remain
historical):

- `apps/web/src/editor/spatialFiles.ts`: `8c21184b4d3a83b72827f6afe02fa0b3206c82ad566b97c2bb7499973ede2939`.
- `apps/web/src/shell/SpatialController.ts`: `0949900fba11075339e497c751e380fa0820a9621032e5a985d4cf139206faa5`.
- `apps/web/e2e/spatialWorkspace.spec.ts`: `b8c3bf22c8849478fddde369962752230e22f449ce319f035a068661edf1e54f`.
- `apps/web/e2e/spatialWalkthrough.spec.ts`: `0dbdad775fca3107c343a205bb8b005202bbd42b97a6ae4c4b2f7239dce31132`.
- `scripts/test-spatial-ui.sh`: `349c0895b8720dfe1aaed087db5d67a15e6cc9f8518d7b15b8c94bac5bd1eb91`.

Issue 72 adapter extension (2026-10-19):

- Spatial layout state is explicitly paged. Center/restore, view pin/expansion,
  dock/resize/close, drawer visibility/pins, accordion and search call the existing
  layout/controller handlers. A close request may open the existing dirty-work
  prompt; dispatch never claims that the tab closed or that drafts were discarded.
- Hidden/collapsed body controllers retire while their editors remain mounted.
  A focused real CodeMirror test confirms agent center/restore preserves the same
  editor, draft and selection, exposes only the central test surface, and refuses
  a stale layout handle. It and the affected existing preservation test passed.
- Transient hover previews and remaining pointer/minimap gestures still require
  adapters and native visual evidence. These tests do not establish complete layout
  parity or native geometry/accessibility correctness.


## 2026-10-22 focused issue 72 review

Spatial excerpt controls expose actual extent and shared edge handlers without changing source or selection; stale edges refuse.

LLM review: partial. See the 2026-10-22 excerpt/test-launch checkpoint in
`docs/developers/handoffs/2026-10-21-issue-72.md` for executed focused evidence,
exact dependency hashes and boundaries. No human judgment supplied.
