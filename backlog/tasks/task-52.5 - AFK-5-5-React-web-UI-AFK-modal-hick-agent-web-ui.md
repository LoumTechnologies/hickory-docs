---
id: TASK-52.5
title: 'AFK 5/5: React web-UI AFK modal (hick-agent/web-ui)'
status: To Do
assignee: []
created_date: '2026-05-20 12:36'
updated_date: '2026-05-20 12:43'
labels:
  - afk
  - web-ui
  - react
  - ux
dependencies:
  - TASK-52.4
  - TASK-54
parent_task_id: TASK-52
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add an AFK modal to the React web-UI (`hick-agent/web-ui/`). The modal must match the information density of the TUI overlay and be dismissable by the user.\n\n**Trigger**: `useAgentSocket` already parses incoming frames by type; add handlers for `afk_entered`, `afk_update`, and `afk_exited` that update a `useAfk` hook (or context).\n\n**Modal design** (full-screen overlay, `position: fixed`, high `z-index`, semi-transparent dark backdrop):\n- Header bar: `AFK MODE` label + entry timestamp + duration counter (client-side tick every second)\n- Stats panel: token table — Input / Cached / Output / Cost (USD) — mirrors TUI bottom panel; updates on every `afk_update`\n- Activity feed: scrollable list of agent turns since AFK entry; each entry shows turn number, one-line summary, model name\n- Cloud budget indicator: progress bar showing tokens used vs budget; turns amber at 80%, red at 100%\n- Dismiss button (`Resume`) sends `{ \"type\": \"afk_dismiss\" }` via WebSocket and closes the modal\n\n**Accessibility**: modal traps focus; Escape also dismisses. After dismissal the modal unmounts, no residual UI.\n\n**No new dependencies**: implement using existing project CSS/styling. Do not introduce a modal library if the project doesn't already use one.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Modal appears within one render cycle of afk_entered arriving
- [ ] #2 Duration counter ticks every second client-side (no WebSocket required)
- [ ] #3 Stats table updates on every afk_update without full re-mount
- [ ] #4 Activity feed is scrollable; new turns are appended and auto-scroll to bottom
- [ ] #5 Cloud budget bar is visible; correct amber/red thresholds
- [ ] #6 Resume button and Escape both send afk_dismiss and unmount the modal
- [ ] #7 Modal is fully keyboard-navigable and focus-trapped (Tab cycles within modal)
- [ ] #8 afk_exited from agent also closes the modal (screen-unlock path)
- [ ] #9 Blocked network/tool events appear in the activity feed as red [BLOCKED] entries
- [ ] #10 Pending approval requests render as interactive Allow/Deny cards within the modal (not as separate browser dialogs); see TASK-52.6 for full spec
- [ ] #11 User decisions on approval cards are shown as resolved entries after interaction
<!-- AC:END -->
