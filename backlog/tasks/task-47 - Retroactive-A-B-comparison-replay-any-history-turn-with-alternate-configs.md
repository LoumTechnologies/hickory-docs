---
id: TASK-47
title: 'Retroactive A/B comparison: replay any history turn with alternate configs'
status: To Do
assignee: []
created_date: '2026-05-20 12:17'
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
