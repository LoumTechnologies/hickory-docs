---
id: TASK-42
title: 'Conversation branching: rewind to any message and fork'
status: Done
assignee: []
created_date: '2026-05-19 18:26'
updated_date: '2026-05-19 22:25'
labels:
  - feature
  - agent-ui
  - sdk
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Enable users to select any past message in the conversation and branch from it — rewinding the agent's LLM context to that point and starting a fresh thread while keeping the original thread visible.

## Background

The agent currently supports only a linear conversation. Users frequently want to go back to an earlier message, try a different prompt, or explore an alternative approach without losing the original thread. This feature adds that capability end-to-end.

## Architecture

Four layers must change in sequence:

1. **SDK protocol** — new `UiCommand::RewindTo { node_id }` and `UiEvent::BranchStarted { fork_node_id, label }`
2. **Agent core** — handle `RewindTo` by truncating the LLM context at the fork node and resuming the loop
3. **TUI** — keyboard gesture (message cursor + `b`) to send `RewindTo`
4. **Web UI** — hover button on bubbles to send `RewindTo`; new branch column rendered on `BranchStarted`

Subtasks cover each layer in dependency order.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A user can select any UserTurn or AssistantDone message and issue a rewind-to-branch command from both the TUI and the web UI
- [ ] #2 The agent resumes with the LLM context truncated at the selected node; events after the fork point in the original thread are preserved in the event stream but no longer continued
- [ ] #3 The branched thread appears as a new column in the web UI canvas with a Prezi transition to it
- [ ] #4 The branched thread is clearly distinguished in the TUI (header label or indicator)
- [ ] #5 cargo test --workspace passes throughout
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
All sub-tasks complete. Conversation branching (rewind to any message and fork) is fully implemented across the stack:

- **42.1** (SDK primitives): `UiCommand::RewindTo`, `UiEvent::BranchStarted`, `AgentRuntime::set_history` — already done.
- **42.2** (agent core): `AgentEvent::UserMessage` emission, `AgentCommand::RewindTo`, `AgentUiBridge` turn-history checkpointing, `run_ui_event_fanout_task` RewindTo handling, `ReactLoop::history()`/`set_history()`, copper-ledge + iron-peak wiring.
- **42.3** (TUI): `BranchStarted` separator, `bump_branchable_focus`, Up/Down node navigation, `b` key → `UiCommand::RewindTo`, input bar hint, updated header.
- **42.4** (web UI): branch buttons on bubbles, `BranchPointNode`, new branch column on `branch_started`, auto-navigation — already implemented.

All tests pass (`cargo test --workspace -- --test-threads=1`).
<!-- SECTION:FINAL_SUMMARY:END -->
