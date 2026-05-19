---
id: TASK-37.5
title: Web UI — conversation tree visualization with collapsible nodes
status: Done
assignee: []
created_date: '2026-05-19 11:55'
updated_date: '2026-05-19 16:28'
labels:
  - web
  - frontend
  - react
dependencies:
  - TASK-37.4
parent_task_id: TASK-37
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace the raw JSON event dump (task-37.4) with a full conversation tree visualization in the web UI. This is the primary feature users see: a spatial 2D canvas combining iMessage-style chat bubbles, thought bubbles, timeline chips, git-log-style branch connectors, and Prezi-style animated navigation between branches.

## Spatial canvas model

The conversation is laid out on a 2D canvas where:
- **Vertical axis = time** — all threads run top-to-bottom
- **Horizontal axis = branches** — the active thread is centered, sibling branches fan out to the left and right in parallel columns

At any moment the user is "in" one thread. That thread fills the center of the viewport and scrolls vertically like a normal chat. Adjacent branches are visible as narrower, slightly faded columns to the left and right. Branch points show connecting indicators where the threads diverged.

The technical foundation is **React Flow** (`@xyflow/react`): conversation nodes become React Flow nodes, branch connectors become React Flow edges, and the library's built-in pan/zoom and `fitView({ nodes })` API powers the Prezi navigation. Each conversation node renders its own bubble/chip/label content inside the React Flow node wrapper.

A **minimap** (React Flow's built-in component) appears in a corner, showing the full canvas topology. Clicking the minimap triggers Prezi navigation to that location.

## Prezi-style branch navigation

Clicking a branch indicator (a subtle arrow or label on the left or right edge of the viewport) triggers a three-phase animated transition:

**Phase 1 — Zoom out (~200ms)**
The canvas scales down to ~25–30% so all branches are visible simultaneously. A CSS 3D perspective tilt engages: `perspective(1200px) rotateX(6deg) rotateY(±4deg)`, tilting toward the destination branch (left-tilt for left branch, right-tilt for right branch). This gives the sensation of physically pulling back to see the whole structure — the "rotation" that distinguishes this from a plain pan/zoom.

**Phase 2 — Pan (~200ms)**
The canvas translates horizontally to center the target branch. The tilt gradually shifts. At this scale the full conversation topology is readable — branch lengths, fork points, the relationship between threads.

**Phase 3 — Zoom in (~200ms)**
Scale returns to 100% centered on the target branch, the 3D tilt flattens to neutral. The user lands at the vertical position of the branch point and can scroll from there.

Total transition: ~600ms, easing `cubic-bezier(0.4, 0, 0.2, 1)`. The transition must be interruptible — clicking another branch mid-transition navigates to the new target.

Navigation modes:
- **Within-branch**: normal vertical scroll, no transition
- **Cross-branch**: full Prezi transition
- **Back to parent**: Prezi transition back up the hierarchy
- **Jump to now**: button that Prezi-transitions to the bottom of the active branch

## Visual system

### Layout structure

A vertical spine line runs at roughly the 25% horizontal mark within each branch column. It is the structural anchor for all event types. Agent content hangs to the left of the spine; user content floats to the right margin of the column. This asymmetry gives agent messages (typically longer) more width.

Turn separators are subtle horizontal rules with a centered timestamp label:
```
── 10:42:31 ──────────────────────────────────────────
```
These appear between user/agent exchange pairs, not between every event.

### Event type visual treatments

**UserTurn — right-aligned speech bubble**
- Floats to the right margin of the branch column, away from the spine
- Fill: cyan/blue tint (matches TUI user color)
- Shape: rounded rectangle (border-radius ~18px), iMessage proportions
- Tail: a curvy SVG bezier tail at the bottom-right corner — a curved flare pointing down-right, matching iMessage's characteristic look
- Never collapsed

**AssistantDone / streaming Token — left-aligned speech bubble**
- Attaches to the left side of the spine
- Fill: neutral dark surface
- Tail: curvy SVG bezier tail at the bottom-left corner, pointing toward the spine
- Live streaming tokens append to the bubble in-place; the bubble grows downward
- Never collapsed

**Thinking — left-aligned thought bubble**
- Same left-side position as agent speech bubbles, different shape and tail
- Outline: scalloped/cloud border (CSS pseudo-elements or SVG clip-path)
- Tail: three ascending circles of increasing size (classic thought bubble trail)
- **Collapsed by default**: animated pulsing dots (●●●) inside a small cloud bubble with ascending-circle tail
- **Expanded**: cloud bubble expands to show full thinking text

**ActionStart / ActionEnd pair — timeline chip (notebook cell)**
- A horizontal chip crossing the spine; collapsed by default
- **Single click expands both ActionStart and ActionEnd together** — do not require two expand actions
- Expanded state: Jupyter notebook cell model — syntax-highlighted code on top, monospace output below, separated by a thin divider. Exit code badge in the header (green for 0, red otherwise)
- ActionStart and ActionEnd chips that belong to the same execution are paired with a faint vertical connector line when both are visible in collapsed state

**McpCall / McpResponse pair — timeline chip (DevTools network row)**
- A horizontal chip crossing the spine; collapsed by default
- Single click expands both together
- Expanded state: request params (key fields only, heuristic extraction) on top, response summary below. A "View full JSON" toggle at the bottom expands to raw JSON (scrollable, fixed max-height)

**Info / Error — spine labels**
- Tiny inline text on the spine line; no bubble, no chip
- Info: gray label with ℹ icon. Error: red label with ✕ icon

**AdaptiveCard — left-aligned card**
- Card-style border, no speech bubble tail
- Never collapsed

### Branch rendering

Branch column positions are computed from the tree topology. Fork points on the parent column show labeled branch indicators on the left/right edges of the viewport (not on the canvas itself — they're viewport-relative overlays that stay visible during vertical scroll). Clicking an indicator triggers the Prezi transition.

Adjacent branch columns are rendered at reduced opacity (~40%) and slightly narrowed so the user can see the topology without reading individual nodes. Hovering a faded column highlights it and shows a "Switch to this branch" affordance.

When there are more than 3 simultaneous branches, overflow branches are shown as compact labels only in the zoomed-out view; a `+N more` indicator appears in the viewport-edge overlay.

### Collapse state

Browser-local `Map<NodeId, boolean>`. Initial state from `collapsed_by_default` on each event. Collapse All / Expand All in the toolbar affects chips and thought bubbles, not speech bubbles.

### Auto-scroll

Auto-scrolls to the bottom of the active branch as new events arrive, unless the user has scrolled up. "Jump to now" button appears when scrolled up; it also Prezi-transitions if the latest event is in a different branch than the one currently viewed.

### Performance

Per-column virtualization for long branches using `@tanstack/react-virtual` (one virtualizer instance per branch column). At zoom-out scale, virtualization is suspended — at 25% scale, fixed-height placeholder nodes are rendered instead of full bubble content. The SVG/React Flow edge layer redraws only on topology changes.

## Implementation notes

The SVG tail for speech bubbles is an inline `<svg>` absolutely positioned at the bubble corner. Sample bezier path for the right-side (user) tail:
```svg
<path d="M 0 0 C 4 0, 8 4, 6 12 C 10 8, 14 4, 18 0 Z" />
```

The 3D tilt during Prezi transitions uses CSS `perspective` on the React Flow container's parent and `rotateX`/`rotateY` on the container itself, animated via `framer-motion` or a CSS transition on `transform`. Keep tilt angles small (≤8°) — the effect should feel like depth, not vertigo.

## Refinement notes (post-first-visual-iteration)

Intentionally unspecified — require visual feedback to tune:
- Whether long output clips at N lines with "N more" or scrolls inline
- Tool call parameter extraction heuristics (which keys to surface in collapsed view)
- Exact tilt angles and easing curves for the Prezi transition
- Whether adjacent branch columns show node outlines or just the spine line at ambient opacity
- Column width at zoom-out scale — enough to see structure, not enough to read text
- Exact padding, font sizes, and border radius within expanded chip blocks
- Whether ActionStart/ActionEnd far apart in the stream still pair visually or only when adjacent
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The conversation is rendered on a 2D React Flow canvas; branch columns are positioned horizontally, threads scroll vertically
- [x] #2 Clicking a branch indicator triggers a three-phase Prezi transition: zoom out to ~25% with 3D perspective tilt, pan to target branch, zoom in to 100% with tilt returning to flat
- [x] #3 The transition direction of the 3D tilt corresponds to the navigation direction (left/right)
- [x] #4 Transitions are ~600ms total and interruptible — clicking mid-transition navigates to the new target
- [x] #5 The minimap shows full canvas topology and is clickable to trigger Prezi navigation
- [x] #6 Within-branch scrolling is normal vertical scroll with no transition
- [x] #7 Adjacent branch columns are visible at ~40% opacity during within-branch scroll; clicking triggers Prezi transition
- [x] #8 UserTurn events render as right-aligned cyan speech bubbles with a curvy SVG bezier tail at the bottom-right corner
- [x] #9 AssistantDone/Token events render as left-aligned dark speech bubbles with a curvy SVG bezier tail at the bottom-left corner
- [x] #10 Thinking events render as left-aligned thought bubbles with scalloped border and ascending-circle tail; collapsed by default showing animated pulsing dots; expandable
- [x] #11 ActionStart and ActionEnd render as paired timeline chips; a single click expands both together in a notebook-cell layout (code on top, output below)
- [x] #12 McpCall and McpResponse render as paired timeline chips; a single click expands both in a DevTools-row layout (params on top, result below, View full JSON toggle)
- [x] #13 Info and Error events render as small spine labels with no bubble or chip
- [x] #14 Collapse All / Expand All affects chips and thought bubbles but not speech bubbles
- [x] #15 Jump to now button appears when scrolled up; Prezi-transitions if the latest event is in a different branch
- [x] #16 Per-column virtualization is used for long branches; at zoom-out scale, placeholder nodes replace full bubble content
- [x] #17 cargo test --workspace passes (Rust side unaffected)
- [x] #18 Off-screen blocking prompts (network approval, credentials, security) show a pulsing viewport-edge indicator with type icon and label; clicking triggers a Prezi transition to the node
- [x] #19 On-screen blocking prompts render as interactive canvas nodes with a pulsing ring; the edge indicator is absent
- [x] #20 While any blocking prompt is pending, the input bar is locked and a persistent banner is visible regardless of canvas position
- [x] #21 Ctrl+click a branch indicator or node enters crosscut mode, showing multiple panels at 100% scale separated by a ⚡ discontinuity separator
- [x] #22 Each crosscut panel supports independent Prezi branch navigation; panels close individually; last panel dissolves back to single view via Prezi transition
- [x] #23 Free pan (drag, scroll) and free zoom (scroll, pinch, double-click) are disabled; all navigation is via Prezi transitions, minimap, or notification indicators
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
## Script output and tool calls — expanded state

Both are input/output pairs. The visual treatment should make the pairing obvious and let the user treat each pair as one unit.

### Script execution expanded state (ActionStart + ActionEnd pair)

Mental model: a Jupyter notebook cell. Input (code) on top, output below, grouped in one container. A single click on either chip expands both together; clicking again collapses both. Do not require two separate expand actions.

```
┌─ [python] ──────────────────────────────── exit 0 ─┐
│  import pathlib                                     │  ← syntax highlighted
│  for f in pathlib.Path('.').rglob('*.rs'): print(f) │
├─────────────────────────────────────────────────────┤
│  src/main.rs                                        │  ← monospace
│  src/lib.rs                                         │
└─────────────────────────────────────────────────────┘
```

The exit code badge appears in the ActionEnd chip header (green for 0, red otherwise) and in the expanded block header.

### Tool call expanded state (McpCall + McpResponse pair)

Mental model: a browser DevTools network row. Compact — shows what was called and what came back. Key parameters only in the request view; a result summary in the response view.

```
┌─ → github / search_code ───────────────────── 200 ─┐
│  q: "SessionLog"  repo: "hickory-docs"              │  ← key params
├─────────────────────────────────────────────────────┤
│  3 results: src/session_log.rs, ...                 │  ← result summary
└─────────────────────────────────────────────────────┘
```

A "View full JSON" toggle at the bottom of the response section expands to the raw JSON (scrollable, fixed max-height).

### Refinement notes (post-first-visual-iteration)

These details are intentionally unspecified — they require visual feedback to tune correctly:
- Whether output clips at N lines with a "... N more" button, or scrolls within the expanded block (depends on typical output length)
- Whether key parameter extraction for tool calls is heuristic (first N keys) or structured (schema-aware) — start heuristic
- Exact padding, font sizes, and border radius within the expanded block
- Whether ActionStart and ActionEnd that are far apart in the event stream still pair visually, or only when adjacent

## Off-screen notification indicators

Blocking prompts (network approval, credentials, security violations) render as interactive nodes at their location on the canvas. When off-screen, a viewport-fixed HUD indicator appears on the nearest screen edge:

- Position: computed by projecting the node's canvas coords to viewport space via React Flow's `project()`, then finding the intersection of the viewport center → node line with the viewport boundary
- Appearance: pulsing (not subtle — these are blocking), type icon (⚠ security, 🌐 network, 🔒 credentials), short label
- Click: triggers Prezi transition to the node
- When on-screen: the node renders inline with a pulsing ring; the edge indicator disappears

While any blocking prompt is pending, the input bar is locked and a persistent banner communicates this regardless of where the user is in the canvas. The user must resolve the prompt to continue.

## Crosscut multi-panel view

Ctrl+click a branch indicator or node → adds it to the crosscut selection. Entering crosscut mode zooms out slightly (Prezi-style) to reveal the panel layout. Each subsequent Ctrl+click slides a new panel in from an edge.

Panels are separated by a discontinuity separator: a dashed vertical line with ⚡ centered, or a styled torn-edge `· · ·` pattern. The separator communicates that the panels are shown together but are NOT adjacent on the canvas — it is purely informational, not interactive.

Each panel is a 100%-scale viewport into its area of the canvas. Prezi branch navigation still works within each panel. Panels close individually with ×; when one panel remains, the crosscut dissolves back to single view via Prezi transition.

Primary use case: monitoring parallel sub-agents side-by-side without leaving the main thread.

## No free pan/zoom (first cut constraint)

Free pan and zoom are explicitly disabled in the first cut. React Flow config:

```tsx
<ReactFlow
  panOnDrag={false}
  panOnScroll={false}
  zoomOnScroll={false}
  zoomOnPinch={false}
  zoomOnDoubleClick={false}
  preventScrolling={true}
/>
```

Within-branch vertical scroll is handled by node content scroll, not React Flow pan. The minimap triggers Prezi transitions on click, not free pan. All navigation has defined semantics: branch indicators, minimap, notification indicators. Nothing is reachable except via these intentional triggers — unreachability is a signal that the topology needs a navigation affordance added.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented in commit 7a879ff. 13 new source files under `src/canvas/` and `src/nodes/`. Packages added: `@xyflow/react`, `@tanstack/react-virtual`, `framer-motion`. One intentional simplification: crosscut panels render item kind labels rather than full bubble components to avoid duplicating the virtualizer render path; the structural shell (panels, ⚡ separator, close, exit) is fully functional.
<!-- SECTION:FINAL_SUMMARY:END -->
