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
Apply and filesystem operations. Header extensions reserve space through
native block decorations; they never rewrite the filesystem projection to
record a pin or move.

Given an open document, agent conversation, generated file, or other tool
view, Center makes that view the central reading surface. Restore returns to
the previous center. Changing presentation keeps each tab's editor mounted,
with its buffer, selection, editor tools, native gutters and line addresses.
The Agent pane uses the same placement mechanism as other views; its own
session and permission controls remain its document's responsibility.

Given a side view, a view pin freezes its placement within the drawer and pins
that drawer. The center's **Pin view in drawer** action moves the view beside a
different open tab without cloning its editor. A pin freezes presentation until the
person drags, resizes, minimizes or centers that view, or changes the available
viewport, drawer geometry or column widths. Pin state does not grant permission to edit bytes.
Dragging previews a tentative rectangle and dropping commits that rectangle.
Dragging or resizing an unpinned view pins it. Both resize edges are available;
growth uses free space, then shifts the adjacent pin into available space,
then reduces that neighbor only when the next pin or viewport blocks a shift.

Given related content in the center, visible drawer views follow its real provenance
anchors with parallax. In the central alignment band they use the source's
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
