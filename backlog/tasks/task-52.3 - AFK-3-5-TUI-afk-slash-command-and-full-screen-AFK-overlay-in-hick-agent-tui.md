---
id: TASK-52.3
title: 'AFK 3/5: TUI /afk slash command and full-screen AFK overlay in hick-agent-tui'
status: To Do
assignee: []
created_date: '2026-05-20 12:35'
updated_date: '2026-05-20 12:43'
labels:
  - afk
  - tui
  - ratatui
  - ux
dependencies:
  - TASK-52.2
  - TASK-54
parent_task_id: TASK-52
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Wire AFK mode into `hick-agent-tui`.\n\n**`/afk` command**: Register `/afk` as a slash command in the TUI input handler. Typing `/afk` sends `UiCommand::AfkEnter`. The TUI already dispatches slash commands; add this to that list alongside `/clear`, `/model`, etc.\n\n**AFK overlay**: When `UiEvent::AfkEntered` is received, push an `AfkOverlay` onto the TUI layer stack (similar to how the MCP-credential prompt and network-approval dialogs work as modal overlays today). The overlay:\n- Covers the full terminal with a dark background (use `ratatui::widgets::Clear` + styled block)\n- Top: large `[AFK MODE]` banner with entry timestamp\n- Middle: scrollable activity log — one line per agent turn that occurred since entry (assistant response summary, tool calls, model used)\n- Bottom panel: live token/cost table — input tokens, cached tokens, output tokens, cost (USD), which models were used; updates on every `UiEvent::AfkUpdate` tick\n- Footer: `Press Enter or Esc to exit AFK mode`\n\n**Exit**: Enter or Esc in the overlay sends `UiCommand::AfkDismiss`, which pops the overlay and resumes the normal TUI.\n\n**Auto-entry**: `UiEvent::AfkEntered` from the state machine (screen lock path) also pushes the overlay — no separate code path needed.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 /afk typed in the input bar enters AFK mode and shows the overlay immediately
- [ ] #2 Overlay renders correctly at minimum 80x24 terminal size
- [ ] #3 Activity log scrolls if content exceeds screen height (arrow keys)
- [ ] #4 Token/cost table updates live as UiEvent::AfkUpdate ticks arrive (roughly every 2 s)
- [ ] #5 Enter and Esc both dismiss the overlay and exit AFK mode
- [ ] #6 Normal TUI resumes immediately after dismissal with no visual artifacts
- [ ] #7 Overlay is also shown when AfkEntered arrives from the automatic screen-lock path
- [ ] #8 Security events (blocked network/tool, approval prompts, approval responses) are shown in the AFK activity feed using the same visual language as TASK-54 conversation entries
- [ ] #9 Pending approval cards are rendered as interactive elements within the AFK overlay — user can approve/deny without dismissing AFK (see TASK-52.6 for full spec)
<!-- AC:END -->
