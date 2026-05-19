---
id: TASK-37.5
title: Web UI — conversation tree visualization with collapsible nodes
status: To Do
assignee: []
created_date: '2026-05-19 11:55'
updated_date: '2026-05-19 12:03'
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
Replace the raw JSON event dump (task-37.4) with a full conversation tree visualization in the web UI. This is the primary feature users see: a hybrid of iMessage-style chat bubbles, thought bubbles, timeline chips, and git-log-style branch connectors — all unified by a vertical spine line.

## Visual system

### Layout structure

A vertical spine line runs at roughly the 25% horizontal mark of the panel (not centered). It is the structural anchor for all event types. Agent content hangs to the left of the spine; user content floats to the right margin. This asymmetry gives agent messages (which are typically longer) more panel width.

Turn separators are subtle horizontal rules with a centered timestamp label:
```
── 10:42:31 ──────────────────────────────────────────
```
These are the "vertical timeline with a line between events" — they appear between user/agent exchange pairs, not between every event.

### Event type visual treatments

**UserTurn — right-aligned speech bubble**
- Floats to the right margin, away from the spine
- Fill: cyan/blue tint (matches TUI user color)
- Shape: rounded rectangle (border-radius ~18px), iMessage proportions
- Tail: a curvy SVG bezier tail at the bottom-right corner — not a sharp CSS triangle, an actual curved flare pointing down-right (away from the spine). The tail shape matches iMessage's characteristic "curled" look.
- Never collapsed

**AssistantDone / streaming Token — left-aligned speech bubble**
- Attaches to the left side of the spine
- Fill: neutral dark surface (matches TUI white-on-dark)
- Tail: curvy SVG bezier tail at the bottom-left corner, pointing toward the spine
- Live streaming tokens append to the bubble in-place; the bubble grows downward as tokens arrive
- Never collapsed

**Thinking — left-aligned thought bubble**
- Same left-side position as agent speech bubbles, but a different shape and tail
- Outline: scalloped/cloud border — achieved with CSS `border-radius` on overlapping pseudo-elements or an SVG clip-path with bumpy edges (no external library needed)
- Tail: a series of 3 ascending circles of increasing size (the classic thought bubble trail), rather than a pointed or curved tail. The circles connect from the spine up to the bubble body.
- **Collapsed by default**: shows the iMessage typing indicator — three pulsing gray dots (●●●) inside a small cloud bubble, with the ascending-circle tail
- **Expanded**: the cloud bubble expands to show the full thinking text with the same scalloped border
- The thought bubble / speech bubble distinction is the primary visual signal that this is reasoning, not output

**ActionStart / ActionEnd / McpCall / McpResponse — timeline chips**
- A horizontal pill/chip that crosses the spine like a node on a timeline
- The chip sits on the spine rather than hanging off it — reads as "something happened here" not "someone said something"
- Collapsed (default): one line — e.g. `[python] print("hello")...` or `→ github/search_repos` with a `▶` chevron on the right
- Expanded: full content block (syntax-highlighted code or raw output) drops below the chip, indented from the spine
- ActionStart and ActionEnd chips are visually paired — a faint connecting line between them when both are visible

**Info / Error — spine labels**
- Tiny inline text sitting directly on the spine line, no bubble, no chip
- Info: small gray label with an ℹ icon
- Error: small red label with a ✕ icon
- These are diagnostic, not conversational — they should not visually compete with bubbles

**AdaptiveCard — left-aligned card**
- Renders inline like an agent bubble but with a card-style border (not a speech bubble shape)
- No tail — it's content, not dialogue
- Never collapsed (the card itself is the compact representation)

### Branch rendering (SubAgentSpawned / SubAgentDone)

When a branch event is received, the spine forks into N parallel vertical lines using SVG bezier curves at the fork point (git-log style). Each branch:
- Has its own narrower spine
- Renders its own bubbles/chips in the same visual system
- Is slightly desaturated relative to the main thread to indicate subordinate context
- Shows a branch label at the fork: `⎇ sub-agent: "summarize repo"`

At the merge point (SubAgentDone), the SVG paths curve back together into the main spine.

When there are more than 3 simultaneous branches, overflow branches collapse into a `+N more` indicator (click to expand into a scrollable side panel).

The SVG fork/merge connectors should be rendered in a dedicated `<svg>` overlay layer positioned absolutely over the conversation panel, so the bubble layout is pure CSS flexbox and the connectors are pure SVG — no mixed layout hacks.

### Collapse state

The browser manages its own collapse state (`Map<NodeId, boolean>`). Initial state comes from `collapsed_by_default` on each event (true for ActionStart, ActionEnd, McpCall, McpResponse, Thinking). The user overrides per-node by clicking. A "Collapse all / Expand all" toggle in the toolbar affects all action chips and thought bubbles but not speech bubbles.

### Auto-scroll

Auto-scrolls to the bottom as new events arrive, unless the user has scrolled up. A "Jump to bottom" button appears when scrolled up.

### Performance

Must not freeze or scroll-jank with 1,000+ nodes. Use windowing/virtualization (e.g. `react-virtual` or `@tanstack/react-virtual`) for the node list. The SVG branch connector layer redraws only when branch topology changes, not on every event.

## Implementation notes

The SVG tail for speech bubbles can be implemented as an inline `<svg>` absolutely positioned at the bubble corner. A sample bezier path for the right-side (user) tail:

```svg
<path d="M 0 0 C 4 0, 8 4, 6 12 C 10 8, 14 4, 18 0 Z" />
```

Adjust control points to taste. The thought bubble ascending circles can be three `<circle>` elements of radius 3, 5, 7 spaced vertically below the bubble.

The scalloped thought bubble border can be approximated with a CSS approach: a `box-shadow` with many small spread values, or with `border-radius` on a rotated pseudo-element grid. If CSS proves too fragile, fall back to an inline SVG `<path>` for the bubble outline.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 UserTurn events render as right-aligned cyan speech bubbles with a curvy SVG bezier tail at the bottom-right corner
- [ ] #2 AssistantDone/Token events render as left-aligned dark speech bubbles with a curvy SVG bezier tail at the bottom-left corner, attached to the spine
- [ ] #3 Thinking events render as left-aligned thought bubbles with a scalloped/cloud border and ascending-circle tail; collapsed by default showing animated pulsing dots (●●●); expandable to show thinking text
- [ ] #4 ActionStart and ActionEnd events render as horizontal timeline chips crossing the spine; collapsed by default with a ▶ chevron; expanded shows full code/output block
- [ ] #5 McpCall and McpResponse events render as timeline chips; collapsed by default
- [ ] #6 Info and Error events render as small inline spine labels (no bubble, no chip)
- [ ] #7 AdaptiveCard events render as left-aligned cards with a card border (no speech bubble tail)
- [ ] #8 Turn separators render as horizontal rules with a centered timestamp label between exchange pairs
- [ ] #9 The vertical spine line runs at ~25% of the panel width; agent content is left of spine, user content is right-aligned to the margin
- [ ] #10 Sub-agent branches render as parallel spine lines with SVG bezier fork/merge connectors; branch label shown at fork point; >3 simultaneous branches collapse to +N more
- [ ] #11 Collapse All / Expand All toolbar toggle affects thought bubbles and action chips but not speech bubbles
- [ ] #12 Auto-scroll follows new events unless the user has scrolled up; Jump to bottom button appears when scrolled up
- [ ] #13 Layout uses windowing/virtualization for the node list; no freeze or scroll-jank at 1,000 nodes
- [ ] #14 SVG branch connectors are on a separate absolutely-positioned overlay layer; bubble layout is pure CSS flexbox
- [ ] #15 cargo test --workspace passes (Rust side unaffected)
<!-- AC:END -->
