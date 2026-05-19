---
id: TASK-42.4
title: >-
  Conversation branching — web UI: branch button on bubbles and new branch
  column
status: To Do
assignee: []
created_date: '2026-05-19 18:27'
labels:
  - web
  - frontend
  - react
dependencies:
  - TASK-42.1
  - TASK-42.2
parent_task_id: TASK-42
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add the branch affordance to the web UI canvas and handle `BranchStarted` events to create new branch columns.

## Current web UI state

The canvas (`hick-agent/web-ui/src/canvas/`) renders conversation nodes as `DisplayItem` rows inside `BranchColumn`. Sub-agent branches already create new columns via `sub_agent_spawned` events. `UiEvent` types are in `src/types.ts`.

## Changes required

### `treeModel.ts` — handle `branch_started`
- Add a new `DisplayItem` kind: `{ kind: 'branch_point'; nodeId: number; forkNodeId: number; label: string }` to mark where a branch diverges in the parent column.
- On `branch_started`: create a new `BranchMeta` (same logic as `sub_agent_spawned`) and push a `branch_point` item to the parent branch at the fork position.
- Subsequent events after `branch_started` go into the new branch column.

### `nodes/BranchPointNode.tsx` — new component
- Render as a centered label similar to `SubAgentNode`: `⎇ Branch from here → <label>` button.
- Clicking navigates to the forked branch column.

### `BranchColumn.tsx` — render `branch_point`
- Add `branch_point` case to `renderItem`.

### Bubble hover affordance — `UserTurnNode.tsx` and `AssistantNode.tsx`
- On hover, show a small `⎇` icon button positioned at the top-right of the bubble.
- Clicking sends `UiCommand::RewindTo { node_id }` via the `send` prop (needs to be threaded down from `BranchColumn` → `renderItem` → node components).
- The button is only shown when `send` is available (i.e. not in read-only replay mode — check `connected` prop or gate on whether `send` is a no-op).

### `ConversationCanvas.tsx`
- Pass `send` down to `renderItem` for `UserTurnNode` / `AssistantNode` (it is already passed for network approval items — extend the same pattern).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Hovering a UserTurn or AssistantDone bubble shows a small ⎇ branch button
- [ ] #2 Clicking the branch button sends {type:'rewind_to', node_id} over the WebSocket
- [ ] #3 The branch button is not shown in replay mode (when send is a no-op or connected is false)
- [ ] #4 A branch_started event creates a new branch column in the canvas at the correct horizontal position
- [ ] #5 A BranchPointNode appears in the parent column at the fork position and navigates to the new column when clicked
- [ ] #6 The Prezi transition fires to center the new branch column when BranchStarted is received
- [ ] #7 cargo test --workspace passes
- [ ] #8 npm run build passes with no TypeScript errors
<!-- AC:END -->
