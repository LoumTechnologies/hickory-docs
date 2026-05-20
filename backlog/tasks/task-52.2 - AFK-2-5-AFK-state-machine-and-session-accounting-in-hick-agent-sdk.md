---
id: TASK-52.2
title: 'AFK 2/5: AFK state machine and session accounting in hick-agent-sdk'
status: To Do
assignee: []
created_date: '2026-05-20 12:35'
labels:
  - afk
  - sdk
  - tokens
  - state-machine
dependencies:
  - TASK-52.1
  - TASK-26
parent_task_id: TASK-52
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement `AfkManager` in `hick-agent-sdk`: the central state machine that owns AFK entry/exit logic, enforces the cloud token budget, and accumulates per-session accounting.\n\n**Entry triggers** (handled by `AfkManager::enter`):\n- Manual: `/afk` command (UiCommand variant)\n- Automatic: `UiEvent::ScreenLocked` (only if `afk.auto_enter_on_lock = true`)\n\n**While in AFK mode:**\n- All LLM requests are routed to the configured local model (`afk.local_model`). If none is configured, cloud calls are suspended and the agent logs a warning.\n- Incoming cloud API calls that slip through (e.g. tool callbacks) are counted against `afk.cloud_token_budget`. Once the budget is exhausted, further cloud calls return `Err(AfkBudgetExceeded)` immediately.\n- Token/cost counters accumulate from entry time: input tokens, cached tokens, output tokens, cost (using the same pricing table as TASK-51), which models were used.\n\n**Exit triggers** (handled by `AfkManager::exit`):\n- Modal dismissed (`UiCommand::AfkDismiss`)\n- Automatic: `UiEvent::ScreenUnlocked` if `afk.exit_on_unlock = true` (default: same as `auto_enter_on_lock`)\n\n**On exit**: emit a `UiEvent::AfkSummary { duration, tokens, cost_usd, models }` for the TUI/browser to display briefly, and write an `<afk-session>` XML element to the session log.\n\n`AfkManager` exposes a simple `Arc<RwLock<AfkState>>` handle that the TUI and relay bridge can read without holding a lock for long.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 AfkManager struct lives in hick-agent-sdk with enter() / exit() / is_active() API
- [ ] #2 UiCommand::AfkEnter and UiCommand::AfkDismiss variants added
- [ ] #3 UiEvent::AfkEntered, UiEvent::AfkUpdate (periodic tick with current stats), UiEvent::AfkExited variants added
- [ ] #4 Cloud token budget enforced: calls beyond budget return Err, never silently succeed
- [ ] #5 AfkState snapshot (entry_time, tokens, cost_usd, models) accessible without async
- [ ] #6 Session log receives <afk-session> element with full accounting on exit
- [ ] #7 Unit tests cover: manual entry, budget enforcement, exit-on-unlock flag, summary event emission
<!-- AC:END -->
