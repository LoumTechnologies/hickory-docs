---
id: TASK-42.5
title: Conversation branching — /btw slash command in TUI and web UI input bars
status: To Do
assignee: []
created_date: '2026-05-20 12:33'
labels:
  - tui
  - web
  - frontend
  - slash-command
dependencies:
  - TASK-42.1
  - TASK-42.2
parent_task_id: TASK-42
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add a `/btw` slash command to both the TUI and web UI input bars as a text-driven alternative to the keyboard (`b`) and hover-button branch gestures.

## Behaviour

- **`/btw`** (bare) — branches from the most recent `UserTurn` or `AssistantDone` node in the active branch column.
- **`/btw <node_id>`** — branches from the node with the given numeric ID.
- Both forms send the existing `UiCommand::RewindTo { node_id }` — no new protocol is required.
- If `<node_id>` is not a valid integer, show an inline error in the input bar ("invalid node id").
- If the resolved node does not exist in the tree, the agent core will emit `UiEvent::Error`; the UI should surface that naturally (same path as any other error event).
- The command is a no-op (and the hint is hidden) in read-only replay mode — same gate used by the `b` keybinding (TASK-42.3) and the hover button (TASK-42.4).

## TUI (`hick-agent/crates/hick-agent-tui/src/run.rs`)

- Intercept `/btw` and `/btw <n>` in the submit handler before the text is sent as a `UserTurn`.
- Resolve the target node from `ConversationTree`: for bare `/btw`, walk the tree to find the last `UserTurn` or `AssistantDone` node in the current branch.
- Send `UiCommand::RewindTo { node_id }` via the existing command channel.
- Clear the input bar on success; leave it populated on parse error with a status-bar error message.

## Web UI (`hick-agent/web-ui/src/`)

- In the submit handler (`InputBar.tsx` or equivalent), detect input starting with `/btw` before sending a `user_turn` message.
- Same resolution logic as TUI: bare `/btw` → last node in active branch; `/btw <n>` → explicit node.
- Send `{ type: 'rewind_to', node_id }` over the WebSocket via the existing `send` prop.
- Show an inline error (below the input bar) for bad node IDs; clear on next keystroke.
- Hidden / disabled when `connected` is false or in replay mode.

## Note on sequencing

TASK-42.2 must land before this task is testable end-to-end (the agent will not yet handle `RewindTo` until then). The slash command parsing and `RewindTo` dispatch can be implemented and reviewed independently of 42.2.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 /btw with no argument sends RewindTo for the most recent UserTurn or AssistantDone node in the active branch, in both TUI and web UI
- [ ] #2 /btw <node_id> sends RewindTo for the specified node in both TUI and web UI
- [ ] #3 A non-integer argument shows an inline error and does not send any command
- [ ] #4 The command is ignored in replay mode (TUI: config.replay_label.is_some(); web UI: connected is false)
- [ ] #5 The input bar is cleared after a successful /btw dispatch
- [ ] #6 No new UiCommand or UiEvent variants are introduced — RewindTo from TASK-42.1 is reused
- [ ] #7 cargo test --workspace passes
- [ ] #8 npm run build in hick-agent/web-ui passes with no TypeScript errors
<!-- AC:END -->
