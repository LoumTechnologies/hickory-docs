---
id: TASK-42.3
title: 'Conversation branching — TUI: message cursor and branch keybinding'
status: To Do
assignee: []
created_date: '2026-05-19 18:26'
labels:
  - tui
  - frontend
dependencies:
  - TASK-42.1
parent_task_id: TASK-42
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add keyboard-driven message selection and a branch gesture to the Ratatui TUI.

## Current TUI state

The TUI (`hick-agent/crates/hick-agent-tui/src/run.rs`) renders a scrollable list of `StyledLine` items derived from `ConversationTree`. Navigation is scroll-only; there is no per-message cursor.

## Changes required

### Message cursor
- Add a `selected_node: Option<NodeId>` field to app state.
- Up/down arrow keys (when not typing in the input bar) move the cursor between `UserTurn` and `AssistantDone` nodes in the rendered list.
- The selected message is highlighted (e.g. `>` gutter marker or reversed style on the first line).
- Pressing `Escape` or entering the input bar clears the selection.

### Branch keybinding
- When a message is selected, pressing `b` sends `UiCommand::RewindTo { node_id: selected_node }` via the command channel.
- The status bar (or a brief flash message) shows `[b] branch from here` as a hint while a message is selected.
- In read-only replay mode (`config.replay_label.is_some()`), `b` is disabled and the hint is not shown.

### Post-branch feedback
- When `UiEvent::BranchStarted` arrives, show a separator line in the output: `── Branch from #<fork_node_id> ──`.
- Clear the message cursor after issuing `b`.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Up/down arrows navigate between UserTurn and AssistantDone messages when the input bar is not focused
- [ ] #2 The selected message is visually distinguished from surrounding lines
- [ ] #3 Pressing b with a message selected sends UiCommand::RewindTo for that message's node_id
- [ ] #4 The status bar shows a branch hint while a message is selected and hides it otherwise
- [ ] #5 Pressing b in replay mode has no effect
- [ ] #6 A BranchStarted event causes a clearly labelled separator line to appear in the output
- [ ] #7 cargo test --workspace passes
<!-- AC:END -->
