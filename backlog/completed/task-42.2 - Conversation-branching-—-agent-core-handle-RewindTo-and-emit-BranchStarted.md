---
id: TASK-42.2
title: 'Conversation branching — agent core: handle RewindTo and emit BranchStarted'
status: Done
assignee: []
created_date: '2026-05-19 18:26'
updated_date: '2026-05-19 22:24'
labels:
  - backend
  - agent-core
dependencies:
  - TASK-42.1
parent_task_id: TASK-42
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement the server-side handling of `UiCommand::RewindTo`. When this command arrives, the agent truncates its LLM context to the messages up to and including the target node, emits `UiEvent::BranchStarted`, and re-enters the input-waiting loop so the user's next message begins the forked thread.

## Where to implement

The command dispatch lives in `hick-agent/crates/hick-agent-script-first/src/agent_ui_bridge.rs` (the bridge between UiCommands and the agent loop). The LLM context is built from `ConversationTree` in `hick-agent-sdk`.

## Behaviour spec

1. On receiving `RewindTo { node_id }`:
   - Walk the `ConversationTree` to find the node. If not found, emit `UiEvent::Error` and ignore.
   - Collect the ancestor path from the session root to `node_id` (inclusive) as the new LLM message list.
   - Emit `UiEvent::BranchStarted { fork_node_id: node_id, label: "Branch from #<node_id>" }` to all UI subscribers.
   - Discard any in-flight LLM request (cancel the current turn if one is running).
   - Reset the agent loop's context to the collected ancestor path and re-enter the waiting-for-input state.

2. Events emitted after the fork point on the *original* thread remain in the event log as-is (they are part of the session XML) but the agent no longer appends to them.

3. The new branch is a logical continuation: subsequent `UserTurn`, `Thinking`, `Token`, etc. events are emitted normally and the UI's `treeModel` will attach them under `BranchStarted` using `fork_node_id`.

## Key constraint

Do not modify the session XML retroactively. The branching is purely additive — new events appended, old events untouched.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 RewindTo with a valid node_id causes BranchStarted to be emitted, followed by the agent entering the waiting-for-input state
- [ ] #2 RewindTo with an unknown node_id emits a UiEvent::Error and leaves the agent state unchanged
- [ ] #3 A message sent after a successful RewindTo is processed with LLM context truncated to the fork point (no messages after fork_node_id appear in the API request)
- [ ] #4 An in-flight LLM request is cancelled cleanly when RewindTo arrives mid-turn
- [ ] #5 The session XML after branching contains both the original post-fork events and the new branch events (additive only, nothing removed)
- [ ] #6 cargo test --workspace passes
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented the agent-core side of conversation branching:

- Added `AgentEvent::UserMessage(String)` — emitted at the start of `ReactLoop::run()` so UIs can create `UserTurn` nodes in live mode (previously only replay emitted these).
- Added `AgentCommand::RewindTo { fork_node_id, user_assistant_history }` — carries the truncated message history to restore at the fork point.
- Rewrote `AgentUiBridge` to maintain a parallel `turn_history: Vec<Message>` and `node_turn_checkpoint: HashMap<NodeId, usize>` so any node_id can be mapped back to its LLM history position.
- `run_ui_event_fanout_task` now owns the `ui_cmd_rx`, `agent_cmd_tx`, and `interrupt` — it handles `Cancel`, `RewindTo` (emits `BranchStarted`, truncates bridge history, forwards `AgentCommand::RewindTo`), and all other `UiCommand`s.
- Added `ReactLoop::history()` and `ReactLoop::set_history()` public wrappers.
- Updated `copper-ledge` and `iron-peak` `agent_task` functions to handle `AgentCommand::RewindTo`: preserve system messages from current history, prepend them to the forked user/assistant turns, call `set_history`, and emit an Info event confirming the rewind.
<!-- SECTION:FINAL_SUMMARY:END -->
