---
id: TASK-37.5
title: Web UI — conversation tree visualization with collapsible nodes
status: To Do
assignee: []
created_date: '2026-05-19 11:55'
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
Replace the raw JSON event dump (task-37.4) with a full conversation tree visualization in the web UI. This is the primary feature users see: a git-log-style tree showing the full conversation with branches for sub-agents and parallel tasks, with script source/output collapsed by default.

## What to build

### ConversationTree React component

A vertical tree view rendered in the center panel. Nodes render top-to-bottom in insertion order. Branches render as parallel columns with connecting lines (similar to `git log --graph`), using SVG or CSS-based connectors.

### Node types and visual treatment

| UiEvent type | Visual | Collapsed by default |
|---|---|---|
| UserTurn | Cyan bullet + text | No |
| Thinking | Gray spinning indicator | No |
| Token / AssistantDone | White text | No |
| ActionStart (script) | Code icon + language badge | **Yes** |
| ActionEnd (output) | Terminal icon + exit code badge | **Yes** |
| McpCall | Tool icon + server name | **Yes** |
| McpResponse | Check icon + server name | **Yes** |
| SubAgentSpawned | Branch indicator + label | No |
| SecurityViolation | Red warning icon + text | No |
| AdaptiveCard | Card icon (rendered in task-37.6) | No |
| Info | Gray text | No |
| Error | Red text | No |
| Done | Green checkmark | No |

### Collapsed node rendering

A collapsed node shows a single-line summary: icon + type label + key metadata (e.g., `[python] print(1)...` or `exit 0`). A `▶` chevron indicates it is expandable. Clicking anywhere on the node expands it.

### Expanded script source rendering

When ActionStart is expanded, display the code in a syntax-highlighted code block (use `highlight.js` or `prism-react-renderer` — pick whichever is smaller). The language is provided by the `lang` field on the event.

### Branch rendering

When a `SubAgentSpawned` event is received, the corresponding sub-tree renders as a branch column to the right of the main thread. All events with `parent_id` matching the sub-agent's `node_id` appear in that branch. When `SubAgentDone` is received, the branch merges back. Multiple simultaneous branches render as parallel columns.

### Collapse state

The browser manages its own collapse state (a `Map<NodeId, boolean>`). The initial state is determined by `collapsed_by_default` on each event. The user can override per-node. A "Collapse all / Expand all" toggle button appears in the toolbar.

### Auto-scroll

The view auto-scrolls to the bottom as new events arrive, unless the user has scrolled up (in which case a "Jump to bottom" button appears).

## Constraints

- Branch column rendering must degrade gracefully when there are more than 4 simultaneous branches (collapse to a simplified view rather than overflowing)
- Syntax highlighting is optional — if the library adds too much weight, fall back to a monospace `pre` block
- The component must handle 10,000+ nodes without freezing (use windowing/virtualization if needed)
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 All UiEvent types render with distinct visual treatment per the node type table
- [ ] #2 ActionStart and ActionEnd nodes are collapsed by default; clicking expands them
- [ ] #3 McpCall and McpResponse nodes are collapsed by default; clicking expands them
- [ ] #4 Expanded script source displays the code in a monospace block (syntax highlighting preferred)
- [ ] #5 Sub-agent branches render as parallel columns with connecting lines; merge when SubAgentDone is received
- [ ] #6 Collapse All / Expand All toolbar toggle works
- [ ] #7 Auto-scroll follows new events unless the user has scrolled up; Jump to bottom button appears when scrolled up
- [ ] #8 Layout does not freeze or scroll-jank with 1,000 nodes in the tree
- [ ] #9 cargo test --workspace passes (Rust side unaffected)
<!-- AC:END -->
