---
id: TASK-47
title: 'Retroactive A/B comparison: replay any history turn with alternate configs'
status: Done
assignee: []
created_date: '2026-05-20 12:17'
updated_date: '2026-05-20 21:39'
labels:
  - eval
  - ui
dependencies:
  - TASK-37
  - TASK-44
  - TASK-46
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
From the conversation history view (TUI or browser UI), the user can select any past turn and replay it with one or more alternate configs. The original response is preserved and shown alongside the new response(s) in the same tab structure used by live multi-config mode. The history up to (but not including) the selected turn is used as context for the replay. This allows retroactive evaluation: you used the agent normally, then later decide to see how a different model or extension combination would have responded. Results are written to the eval store.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 In history view, a keybinding (TUI) and button (browser) triggers replay-with-alternate-config on the selected turn
- [ ] #2 The user can select one or more alternate configs to replay against
- [ ] #3 Original response is preserved and shown in its own tab labeled with the original config
- [ ] #4 Replay uses conversation history up to the selected turn as context
- [ ] #5 Replay results written to eval store with a reference to the original turn
- [ ] #6 Works in both TUI and browser UI with feature parity
- [ ] #7 Replaying a turn does not modify the original conversation history
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented retroactive A/B comparison (replay any history turn with alternate configs).

**Rust changes:**
- `hick-eval-store/src/record.rs`: Added `original_turn_id: Option<String>` to `EvalResultRecord`; fixed all construction sites.
- `hick-agent-sdk/src/ui_event.rs`: Added `UiCommand::ReplayTurn { turn_node_id, config_paths }` variant.
- `hick-agent-script-first/src/agent_ui_bridge.rs`: Added `ReplayRequest` struct, `handle_replay_turn` function, and `replay_tx` parameter to `run_ui_event_fanout_task`.
- `hick-agent-script-first/src/lib.rs`: Re-exported `ReplayRequest`.
- `hick-agent-tui/src/run.rs`: Added `PendingReplay` state, `r` keybinding on `UserTurn` nodes, config path input flow, and updated hint text.
- `hick-agent/src/main.rs`: Added replay channel, `handle_replay_requests` task, and `write_replay_eval_record` function; replay agents spawn new tabs via existing `add_tab_tx` machinery (TASK-46 reuse).

**Web UI changes:**
- `web-ui/src/types.ts`: Added `replay_turn` to `UiCommand`.
- `web-ui/src/nodes/UserTurnNode.tsx`: Added `onReplay` prop with hover button (↺).
- `web-ui/src/canvas/BranchColumn.tsx`: Wired `onReplay` callback using `window.prompt` for config path input.

All tests pass (`cargo test --workspace -- --test-threads=1`).
<!-- SECTION:FINAL_SUMMARY:END -->
