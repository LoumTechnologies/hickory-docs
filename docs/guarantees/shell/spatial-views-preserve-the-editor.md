# Spatial Views Preserve The Editor

Given an open folder, the workspace places two independent Files buffers at
its outer left and right edges. Each remains the ordinary CodeMirror tree
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

Given a side view, pinning or unpinning preserves its exact current rectangle
and the other visible view rectangles. A pin freezes presentation until the
person drags, resizes, minimizes or centers that view, or changes the available
viewport or column widths. Pin state does not grant permission to edit bytes.
Dragging previews a tentative rectangle and dropping commits that rectangle.
Dragging or resizing an unpinned view pins it. Both resize edges are available;
growth uses free space, then shifts the adjacent pin into available space,
then reduces that neighbor only when the next pin or viewport blocks a shift.

Given related content in the center, side views follow its real provenance
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
placement uses the final clear slot directly. On narrow screens side views
occupy a separate region below the center. The workspace itself does not add
an outer scrollbar to its native editor scrollports.

Given a folder collapsed in a Files buffer, unpinned descendant side views on
that side disappear. A provenance endpoint may terminate at the visible
collapsed ancestor folder. This is a navigation address; it neither invents
nor changes the recorded provenance. Pins remain available. Filenames in the
extended headers use paths relative to the open folder; unnamed and tool views
use their ordinary titles.

---

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
