---
id: TASK-42.2
title: 'Conversation branching — agent core: handle RewindTo and emit BranchStarted'
status: To Do
assignee: []
created_date: '2026-05-19 18:26'
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
