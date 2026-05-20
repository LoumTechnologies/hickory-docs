---
id: TASK-50
title: 'Idle local eval: background eval using llama.cpp when hick is left running'
status: To Do
assignee: []
created_date: '2026-05-20 12:17'
labels:
  - eval
  - local-model
dependencies:
  - TASK-44
  - TASK-45
  - TASK-49
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
When the running `hick` process detects no user interaction for a configurable idle timeout (default: 5 minutes), it starts background Tokio tasks that replay past session queries through untested config combinations using the local llama.cpp model. Idle detection is based on last UI event timestamp. The local model is loaded once and kept resident for the duration of the idle period. When user interaction resumes, in-flight eval tasks complete their current turn but do not start new ones. Results from idle eval sessions are written to the eval store as normal. The idle eval respects the machine capability config — it only runs combinations the machine can handle. A status indicator in the TUI/browser UI shows when idle eval is active and how many combinations remain.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Idle timeout is configurable in [eval] section of agent TOML; default 5 minutes
- [ ] #2 Idle eval starts automatically after timeout with no user action required
- [ ] #3 Local model is loaded once per idle session and released when user activity resumes
- [ ] #4 In-flight eval tasks finish their current turn before stopping; no partial results
- [ ] #5 Idle eval results written to eval store indistinguishable in format from manual eval results
- [ ] #6 Only config combinations not yet evaluated (per eval store) are scheduled
- [ ] #7 Pruned configs are excluded from idle eval scheduling
- [ ] #8 TUI status line and browser UI indicator show: idle eval active, combinations remaining, current config under test
- [ ] #9 Idle eval does not start if no local model is configured in [machine]
<!-- AC:END -->
