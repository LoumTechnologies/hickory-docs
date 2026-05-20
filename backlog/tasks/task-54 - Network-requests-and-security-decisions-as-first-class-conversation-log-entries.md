---
id: TASK-54
title: >-
  Network requests and security decisions as first-class conversation log
  entries
status: To Do
assignee: []
created_date: '2026-05-20 12:43'
labels:
  - ux
  - security
  - tui
  - web-ui
  - conversation-log
dependencies:
  - TASK-37
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Network requests (outbound HTTP, WebSocket upgrades, DNS) and the security decisions made about them (allowed, blocked, pending approval, user response) should appear inline in the conversation timeline, alongside tool calls, thought bubbles, and LLM/user messages. Currently these events exist only as ephemeral dialogs that disappear without a trace.\n\n## New conversation node kinds (hick-agent-sdk ConversationNodeKind)\n\n```rust\nNetworkRequest { url: String, method: String, verdict: NetworkVerdict, initiated_at: DateTime },\nNetworkApprovalPrompt { request_id: Uuid, url: String, method: String, expires_at: Option<DateTime> },\nNetworkApprovalResponse { request_id: Uuid, verdict: ApprovalResponse, decided_by: DecisionSource },\nToolCallSecurity { tool: String, args_summary: String, verdict: NetworkVerdict },\nToolApprovalPrompt { request_id: Uuid, tool: String, args_summary: String },\nToolApprovalResponse { request_id: Uuid, verdict: ApprovalResponse, decided_by: DecisionSource },\n```\n\n`DecisionSource` = `User | PolicyDefault | Timeout`.\n\n`NetworkVerdict` reuses the existing type from `hick-agent-sdk::network_filter`.\n\n## TUI rendering\n\nAdd rendering arms in `hick-agent-tui/src/run.rs` for the new node kinds:\n- `NetworkRequest (allowed)`: dim green `→ net  GET api.example.com/path` (collapsible; collapsed by default after 3 per turn)\n- `NetworkRequest (blocked)`: bold red `✗ net  POST api.evil.com — blocked`\n- `NetworkApprovalPrompt`: yellow `? net  GET … (awaiting your decision)` — becomes historical once resolved\n- `NetworkApprovalResponse (user/allow)`: green `✓ net  allowed by user`\n- `NetworkApprovalResponse (user/deny)`: red `✗ net  denied by user`\n- `NetworkApprovalResponse (timeout)`: dim `✗ net  denied — approval timeout`\n- Tool call variants use `→ tool` / `✗ tool` / `? tool` prefixes with the same colour scheme\n\n## Web-UI rendering\n\nIn `hick-agent/web-ui/`, add card components for each new node kind:\n- Network allowed: small pill `GET api.example.com — allowed` with expand-to-full-URL affordance\n- Network blocked: red card with URL, method, and reason\n- Approval prompt (historical): inline card showing what was asked and how it was resolved, with timestamp\n- Approval prompt (live): same as the existing NetworkApprovalPrompt dialog, but rendered inline in the conversation thread so it is part of the permanent record\n- User responses styled as user-action bubbles (distinct from LLM messages)\n\n## Session log\n\nThe XML session log (`XmlSessionLog`) must write a `<network-request>` or `<tool-security>` element for every event, including the verdict and decision source, so the full audit trail is preserved across restarts.\n\n## No duplicate UI\n\nOnce this task is done, the existing ephemeral network-approval dialog in the TUI should be removed or refactored to drive the new conversation-node path. The approval prompt node in the conversation tree IS the dialog — it just happens to be rendered with interactive controls when pending.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 New ConversationNodeKind variants exist in hick-agent-sdk for all six event types above
- [ ] #2 TUI renders all six variants with correct colours/prefixes in the conversation tree
- [ ] #3 Allowed network requests are collapsed by default after 3 per turn (expand with arrow key)
- [ ] #4 Blocked requests and denied approvals use red styling; allowed/approved use green
- [ ] #5 Approval prompts rendered inline are interactive (same keyboard controls as current dialog) while pending
- [ ] #6 Once resolved, approval prompt node updates in place to show the decision and who made it
- [ ] #7 Web-UI renders all variants as inline conversation cards with the same information hierarchy
- [ ] #8 User approval/denial is shown as a user-action styled bubble in the web-UI
- [ ] #9 XML session log receives an element for every security event including verdict and decision source
- [ ] #10 Existing ephemeral approval dialog is removed; all approval UX flows through the conversation node
<!-- AC:END -->
