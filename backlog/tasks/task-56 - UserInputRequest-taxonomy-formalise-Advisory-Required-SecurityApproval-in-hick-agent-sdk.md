---
id: TASK-56
title: >-
  UserInputRequest taxonomy: formalise Advisory / Required / SecurityApproval in
  hick-agent-sdk
status: To Do
assignee: []
created_date: '2026-05-20 13:09'
updated_date: '2026-05-20 13:14'
labels:
  - sdk
  - ux
  - architecture
dependencies:
  - TASK-37
references:
  - docs/specs/user-input.md
  - docs/specs/security-prompts.md
  - hick-agent/crates/hick-agent-tui/src/run.rs
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace the current ad-hoc collection of user-prompt types (`PendingApproval`, `PendingMcpCredential`, `UiNetworkApproval`) with a single principled `UserInputRequest` enum in `hick-agent-sdk`, as specified in `docs/specs/user-input.md`.\n\nThe three variants and their key behavioural differences:\n\n| | Advisory | Required | SecurityApproval |\n|---|---|---|---|\n| Agent blocked? | No — proceeds with default after timeout | Yes — no fallback | Yes — pending authorisation |\n| Timeout | MUST have one; proceeds with stated default | MUST NOT have one | Governed by security-prompts spec (always deny) |\n| AFK | Self-resolves; logs to activity feed | High-priority \"waiting\" card; agent halted | Approve/deny card with countdown |\n\n## Migration\n\n- `PendingMcpCredential` → `Required { kind: Secret }` with `prompt` derived from the current credential prompt text\n- `UiNetworkApproval` / `PendingApproval` → `SecurityApproval { action: NetworkRequest { .. }, timeout }`\n- Tool-call approval (currently ad-hoc) → `SecurityApproval { action: ToolCall { .. }, timeout }`\n\n## UiEvent changes\n\nReplace the separate `UiEvent` variants for each old type with:\n```rust\nUiEvent::UserInputRequested { id: Uuid, request: UserInputRequest }\nUiEvent::UserInputResolved { id: Uuid, response: UserInputResponse }\n```\n\n`UserInputResponse` carries the chosen value (or `Auto(default)` for advisory timeouts, or `Timeout(Deny)` for security timeouts).\n\n## TUI and web-UI\n\nBoth frontends switch from handling three separate event types to a single `UserInputRequested` event, dispatching to the appropriate renderer based on the variant. This simplifies the rendering logic and ensures consistent treatment across the codebase.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 UserInputRequest enum with three variants exists in hick-agent-sdk
- [ ] #2 UserInputResponse enum covers: UserProvided(value), Auto(default), Timeout(Deny)
- [ ] #3 PendingMcpCredential fully replaced by Required { kind: Secret } with no behaviour change
- [ ] #4 UiNetworkApproval / PendingApproval replaced by SecurityApproval with no behaviour change
- [ ] #5 UiEvent::UserInputRequested and UserInputResolved replace all legacy prompt event variants
- [ ] #6 TUI dispatches on UserInputRequested variant; all three render paths exist and pass existing behaviour tests
- [ ] #7 Advisory variant: timeout fires, agent proceeds with default, event logged
- [ ] #8 Required variant: no timeout; agent stays blocked across full session until response
- [ ] #9 SecurityApproval: timeout always results in Deny per security-prompts spec
- [ ] #10 SecurityApproval variant enforces the 5-second activation delay per security-prompts spec before the approve action becomes available
<!-- AC:END -->
