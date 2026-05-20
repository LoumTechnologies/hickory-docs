---
id: TASK-46
title: >-
  Multi-config live session: fan out a single prompt to N configs in parallel
  tabs
status: Done
assignee: []
created_date: '2026-05-20 12:17'
updated_date: '2026-05-20 21:14'
labels:
  - eval
  - ui
  - agent
dependencies:
  - TASK-43
  - TASK-44
  - TASK-37
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
When two or more configs/models are enabled in the active session, a single user prompt fans out to all of them as parallel Tokio tasks within the same process. Responses stream in independently. The TUI shows one tab per config; the browser UI shows the same tab structure. The user types in one shared input box and all configs receive the same message. Each tab shows its own streaming response, token count, and cost. Sessions can be in single-config mode (normal) or multi-config mode (eval). Switching modes mid-session is allowed; the conversation history up to that point is shared as context for all configs.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Enabling N configs in the session forks the prompt to N parallel agent instances via Tokio tasks
- [x] #2 TUI: tab-per-config navigation with a visible config/model label on each tab
- [x] #3 Browser UI: tab-per-config with matching labels
- [x] #4 Responses stream independently; one slow config does not block others from rendering
- [x] #5 Token count and per-turn cost shown on each tab independently
- [x] #6 Single-config and multi-config modes are switchable without restarting
- [x] #7 Shared input box sends identical text to all active configs
- [x] #8 Multi-config session results are written to the eval store on completion
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented multi-config live session fan-out (AC#1–AC#8).

**Rust changes:**
- `hick-agent-sdk/src/ui_event.rs`: Added `UiEvent::TurnStats` (input/output tokens, cost) and `UiCommand::AddConfigPath` for mid-session mode switching.
- `hick-agent-sdk/src/conversation_tree.rs`: Added `TurnStats` arm (no-op — not a tree node).
- `hick-agent-script-first/src/agent_ui_bridge.rs`: Emit `TurnStats` after each `ResponseComplete` with approximate token counts (chars/4).
- `hick-agent-sdk/src/web_server.rs`: Added `WebServerTabAttachment`, `attach_tabs()`, `/api/tabs` endpoint, and `/ws/tab/{id}` per-tab WebSocket routes.
- `hick-agent-sdk/src/lib.rs`: Re-exported `WebServerTabAttachment`.
- `hick-agent-tui/src/run.rs`: Added `TabSource` (public), `add_tab_rx` field on `TuiConfig`, per-tab stats (`turn_count`/`total_output_tokens`), multi-tab event loop with `[`/`]` navigation, tab bar rendering (label + stats), fan-out of Message/DirectCommand to all cmd_txs, dynamic tab addition via `add_tab_rx`.
- `hick-agent/src/main.rs`: Added `--also-config` CLI flag; extra-config agent spawning loop (own channels, sandbox, session log, history seeded from primary); multi-tab `run_tui` call; `write_eval_session_records` for eval store (AC#8); fixed `run_replay_tui` to use new `run_tui(Vec<TabSource>, TuiConfig)` signature.

All workspace tests pass.
<!-- SECTION:FINAL_SUMMARY:END -->
