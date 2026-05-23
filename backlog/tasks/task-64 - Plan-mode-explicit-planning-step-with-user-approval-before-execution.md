---
id: TASK-64
title: 'Plan mode: explicit planning step with user approval before execution'
status: To Do
assignee: []
created_date: '2026-05-23 02:28'
labels: []
milestone: m-7
dependencies: []
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add a plan mode to the agent where, before executing any tools or writing any code, the agent produces a structured plan and waits for user approval. This prevents the agent from doing 10+ minutes of work in the wrong direction — the user can redirect at the plan stage for free.

Plan mode behaviour:
1. User turn arrives (or `/plan` slash command is used to force plan mode for the next request)
2. Agent makes a `plan` role model call (see model routing task) with a planning system prompt — no tools available during this call
3. Agent emits the plan as a `UiEvent::AdaptiveCard` (or structured info node) with Approve / Edit / Reject actions
4. If approved: execution loop proceeds normally
5. If edited: user's edits replace the plan text; execution proceeds with the edited plan prepended to context
6. If rejected: agent discards the plan and asks the user to clarify

Plan output format: a hick XML doc (`<hick:plan>`) with `<hick:step>` children, rendered in the TUI as a collapsible tree.

`agent.toml`: `[plan] auto = false` (default off; user opts in per-turn with `/plan` or turns on auto for all turns)
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 agent.toml [plan] section accepts auto (bool, default false) and enabled (bool, default true)
- [ ] #2 /plan slash command in TUI forces plan mode for the next user turn regardless of auto setting
- [ ] #3 During the plan call no tools are dispatched; the model is constrained to produce a <hick:plan> XML response
- [ ] #4 Plan is rendered in the TUI as a structured node (collapsible steps) with Approve / Edit / Reject affordances
- [ ] #5 Approving the plan starts the normal execution loop with the plan prepended to context
- [ ] #6 Editing the plan replaces plan text; execution proceeds with the edited version
- [ ] #7 Rejecting the plan prompts the user for clarification without executing anything
- [ ] #8 With auto = true every user turn goes through plan mode before execution
- [ ] #9 Plan mode uses the plan model role (falls back to default if not configured)
- [ ] #10 Integration test: plan produced → approved → execution loop runs; plan produced → rejected → no tools called
<!-- AC:END -->
