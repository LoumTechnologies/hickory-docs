---
id: TASK-46
title: >-
  Multi-config live session: fan out a single prompt to N configs in parallel
  tabs
status: To Do
assignee: []
created_date: '2026-05-20 12:17'
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
- [ ] #1 Enabling N configs in the session forks the prompt to N parallel agent instances via Tokio tasks
- [ ] #2 TUI: tab-per-config navigation with a visible config/model label on each tab
- [ ] #3 Browser UI: tab-per-config with matching labels
- [ ] #4 Responses stream independently; one slow config does not block others from rendering
- [ ] #5 Token count and per-turn cost shown on each tab independently
- [ ] #6 Single-config and multi-config modes are switchable without restarting
- [ ] #7 Shared input box sends identical text to all active configs
- [ ] #8 Multi-config session results are written to the eval store on completion
<!-- AC:END -->
