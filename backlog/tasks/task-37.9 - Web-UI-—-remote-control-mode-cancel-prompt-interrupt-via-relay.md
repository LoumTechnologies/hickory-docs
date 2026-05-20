---
id: TASK-37.9
title: 'Web UI — remote control mode (cancel, prompt, interrupt via relay)'
status: In Progress
assignee: []
created_date: '2026-05-19 11:57'
updated_date: '2026-05-20 01:17'
labels:
  - web
  - frontend
  - relay
dependencies:
  - TASK-37.8
  - TASK-37.5
parent_task_id: TASK-37
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The web UI served through the relay (task-37.8) must support full remote control: the user can send prompts, cancel the current operation, approve network requests, and do anything the local TUI can do. This task wires up those interactions in the web UI for the remote case, and ensures network approval prompts work across the relay.

## What to build

### Full input parity in the web UI

The input bar (from task-37.4) must work identically whether the user is on localhost or connected through the relay:
- Submit a prompt → `UiCommand::Message`
- Cancel / interrupt → `UiCommand::Cancel`
- `/mcp install`, `/mcp list`, `/mcp remove` slash commands (same as TUI)

### Network approval UI

When the agent sends a `UiEvent::NetworkApprovalRequest`, the web UI displays a modal or inline prompt:

> `Allow connection to api.openai.com?`
> `[Allow once]  [Always allow ▾]  [Block]`

"Always allow" opens a dropdown listing the generated policy suggestions (same as TUI 'a' key flow). The user's choice is sent back as a `UiCommand::NetworkApproval { verdict, always_allow }`.

**Relay note**: The relay forwards this command back to the agent, which holds the `oneshot::Sender<ApprovalResponse>` on the Rust side. The agent's `UiEvent` bridge must translate `UiCommand::NetworkApproval` back into `ApprovalResponse` and resolve the sender. Implement this translation in the bridge layer (task-37.1).

### MCP credential prompt UI

When the agent sends `UiEvent::McpCredentialRequest { server_name, suggested_env_var }`, the web UI shows a form:
- Env var name (pre-filled with `suggested_env_var`)
- Credential value (masked input)
- Submit / Cancel buttons

The response is sent as `UiCommand::McpCredential { env_var, value }` or `UiCommand::McpCredentialSkip`.

### Remote vs. local detection

The `useAgentSocket` hook (task-37.4) detects whether it is connected to the local server or the relay by checking the WebSocket URL. When connected via relay:
- Show a "Remote session" indicator in the sidebar
- Show the session ID for sharing

### Read-only vs. control mode

When multiple browsers connect to the same relay session, only the first connection gets control mode by default. Additional connections are read-only. A "Request control" button is visible to read-only viewers; clicking it sends a `UiCommand::RequestControl` which the relay (or agent) can grant.

## Constraints

- Network approval and credential prompts must be surfaced in the web UI even for remote sessions — these are blocking operations on the agent side
- The `UiCommand::NetworkApproval` and `UiCommand::McpCredential` variants must be added to the command enum (as part of task-37.1 or amended here)
- Read-only / control arbitration can be simple (first-come-first-served, no explicit transfer protocol) for the initial implementation
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Prompt submission, cancel, and /mcp slash commands work identically from the relay-connected web UI as from the local web UI
- [ ] #2 Network approval prompts appear in the web UI as a modal/inline block; the user's verdict is forwarded to the agent and resolves the approval
- [ ] #3 Always Allow dropdown shows generated policy suggestions and sends the selected rule back to the agent
- [ ] #4 MCP credential prompts appear as a form in the web UI; responses are forwarded to the agent
- [ ] #5 Remote session indicator and session ID are shown in the sidebar when connected via relay
- [ ] #6 Second+ browser connections to the same session are read-only by default; a Request Control button is visible
- [ ] #7 cargo test --workspace passes
<!-- AC:END -->
