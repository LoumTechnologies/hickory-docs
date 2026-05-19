---
id: TASK-42.1
title: >-
  Conversation branching — SDK protocol: UiCommand::RewindTo and
  UiEvent::BranchStarted
status: Done
assignee: []
created_date: '2026-05-19 18:26'
updated_date: '2026-05-19 18:32'
labels:
  - sdk
  - protocol
dependencies: []
parent_task_id: TASK-42
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add the protocol primitives that all other branching subtasks depend on.

## Files to change

- `hick-agent/crates/hick-agent-sdk/src/ui_event.rs` — add new enum variants
- `hick-agent/web-ui/src/types.ts` — mirror changes in TypeScript

## Changes

### `UiCommand` (Rust)
Add:
```rust
RewindTo { node_id: NodeId }
```
Wire format: `{"type": "rewind_to", "node_id": <u64>}`.

### `UiEvent` (Rust)
Add:
```rust
BranchStarted {
    node_id: NodeId,
    fork_node_id: NodeId,  // the node branched from
    label: String,         // e.g. "Branch from #42"
}
```
Wire format: `{"type": "branch_started", "node_id": ..., "fork_node_id": ..., "label": ...}`.

Update `ui_event_wire_value` and the `Debug` impl to cover both new variants.

### TypeScript (`types.ts`)
Add `{ type: 'rewind_to'; node_id: number }` to `UiCommand`.
Add `{ type: 'branch_started'; node_id: number; fork_node_id: number; label: string }` to `UiEvent`.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 UiCommand::RewindTo { node_id } is present in ui_event.rs and serializes to {"type":"rewind_to","node_id":N}
- [x] #2 UiEvent::BranchStarted { node_id, fork_node_id, label } is present, serializes correctly, and is covered by the Debug impl
- [x] #3 ui_event_wire_value handles both new variants without a non-exhaustive match warning
- [x] #4 TypeScript UiCommand union includes rewind_to variant
- [x] #5 TypeScript UiEvent union includes branch_started variant
- [x] #6 cargo test --workspace passes
- [x] #7 npm run build in hick-agent/web-ui passes with no TypeScript errors
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added `UiCommand::RewindTo { node_id }` and `UiEvent::BranchStarted { node_id, fork_node_id, label }` to `hick-agent-sdk/src/ui_event.rs`. Updated `ui_event_wire_value` and the `Debug` impl for both new variants. Added a no-op arm for `BranchStarted` to `ConversationTree::apply` (exhaustive match). Added `RewindTo` to the `None` arm of `ui_command_to_agent` in `agent_ui_bridge.rs`. Mirrored both variants in `web-ui/src/types.ts`. All matches remain exhaustive with no wildcards dropped.
<!-- SECTION:FINAL_SUMMARY:END -->
