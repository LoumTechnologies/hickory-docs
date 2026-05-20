---
id: TASK-52
title: >-
  AFK Mode: budget-aware standby with screen-lock auto-entry and dismissable
  modal
status: To Do
assignee: []
created_date: '2026-05-20 12:35'
labels:
  - ux
  - afk
  - tokens
  - tui
  - web-ui
dependencies:
  - TASK-26
  - TASK-51
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add an AFK (away-from-keyboard) operating mode. The user can enter it manually with `/afk`; optionally the agent enters it automatically when the OS screen lock activates (configurable, off by default). While in AFK mode the agent switches to a configured local model (no cloud token budget) and enforces a configurable cloud token cap. Both the TUI and the browser web-UI display a prominent modal overlay summarising what has happened since AFK entry and showing live token/cost accounting; dismissing that modal exits AFK mode. Screen lock detection is implemented in the Rust terminal process (Linux: D-Bus `org.freedesktop.ScreenSaver`; macOS: IOKit `com.apple.screenIsLocked` Darwin notification) — no Tauri or desktop-app dependency is required for this feature. Tauri / desktop IDE work is planned separately in TASK-53.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Typing /afk in the TUI immediately enters AFK mode
- [ ] #2 Screen-lock auto-entry is gated by a config key (afk.auto_enter_on_lock, default false)
- [ ] #3 AFK mode switches the active model to the configured local model (or suspends cloud calls if none configured)
- [ ] #4 A configurable cloud token budget (afk.cloud_token_budget) is enforced; agent emits a warning and refuses further cloud calls if exceeded
- [ ] #5 TUI shows a full-screen overlay while in AFK mode (see TASK-52.3)
- [ ] #6 Browser web-UI shows a matching full-screen modal while in AFK mode (see TASK-52.5)
- [ ] #7 Dismissing the modal (Enter/Esc in TUI; close button in browser) exits AFK mode and resumes normal operation
- [ ] #8 Exiting AFK mode logs a summary (duration, total tokens, cost, models used) to the XML session log
- [ ] #9 All AFK config keys are documented in the agent TOML schema
<!-- AC:END -->
