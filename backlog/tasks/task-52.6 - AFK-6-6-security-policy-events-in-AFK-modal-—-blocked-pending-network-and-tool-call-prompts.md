---
id: TASK-52.6
title: >-
  AFK 6/6: security policy events in AFK modal — blocked/pending network and
  tool-call prompts
status: To Do
assignee: []
created_date: '2026-05-20 12:44'
updated_date: '2026-05-20 13:14'
labels:
  - afk
  - security
  - tui
  - web-ui
dependencies:
  - TASK-52.2
  - TASK-52.3
  - TASK-52.5
  - TASK-54
parent_task_id: TASK-52
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
AFK mode does not bypass any security policy. This task ensures that policy outcomes are correctly surfaced inside the AFK modal on both frontends, and that pending approval requests can be answered without dismissing AFK mode.\n\n## Three event categories\n\n**1. Explicitly allowed** (network request or tool call whitelisted in policy)\nAdd a summary line to the AFK activity feed: `[allowed] GET https://api.example.com/v1/...`. No user interaction required. Grouped by host to keep the feed readable (e.g. `3 requests to api.example.com — all allowed`).\n\n**2. Blocked by policy**\nAdd a prominent activity feed entry: `[BLOCKED] POST https://api.evil.com — security policy`. The agent continues without the result (the SDK already handles the `Err` return). The AFK modal makes it clear what was blocked so the user can review on return.\n\n**3. Requires user approval** (network request or tool call in the `requires_approval` policy tier)\nRender an interactive approval card **inside** the AFK modal — the user does not need to dismiss AFK to respond. The card shows:\n- Request type (network / tool call)\n- Target (URL + method, or tool name + arguments summary)\n- Two buttons: **Allow** / **Deny** (keyboard: `a` / `d` when card is focused in TUI)\n- Timeout countdown if the SDK has a configured approval timeout\n\nThe card is appended to the activity feed in arrival order. Multiple cards can be pending simultaneously; each is resolved independently. After the user responds, the card collapses to a one-line summary showing what was chosen.\n\n## SDK changes (AfkManager extension)\n`AfkState` must track blocked and pending events so late-joining frontends can reconstruct the full picture from an `afk_update` payload:\n```rust\npub blocked_events: Vec<AfkSecurityEvent>,\npub approved_events: Vec<AfkSecurityEvent>,\npub pending_approvals: Vec<AfkPendingApproval>,\n```\n`AfkUpdate` payloads include all three lists. This data is also what feeds into the TASK-54 conversation node types (`NetworkApprovalPrompt`, `NetworkApprovalResponse`, etc.) — AFK mode re-uses those nodes rather than inventing parallel types.\n\n## Protocol additions to afk_update activity entries\n```json\n{ \"kind\": \"network_blocked\", \"url\": \"...\", \"method\": \"POST\", \"ts\": \"...\" }\n{ \"kind\": \"tool_blocked\", \"tool\": \"bash\", \"args_summary\": \"rm -rf /\", \"ts\": \"...\" }\n{ \"kind\": \"approval_request\", \"id\": \"uuid\", \"type\": \"network\", \"url\": \"...\", \"method\": \"GET\", \"timeout_secs\": 30 }\n{ \"kind\": \"approval_response\", \"id\": \"uuid\", \"verdict\": \"allow\" }\n```\nBrowser sends: `{ \"type\": \"afk_approval\", \"id\": \"uuid\", \"verdict\": \"allow\" | \"deny\" }`
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Allowed network requests appear in the AFK activity feed, grouped by host after 3+
- [ ] #2 Blocked network requests appear as [BLOCKED] entries in both TUI overlay and React modal
- [ ] #3 Blocked tool calls appear with tool name and args summary
- [ ] #4 Pending approval requests render as interactive cards inside the AFK modal (not as separate dialogs)
- [ ] #5 TUI approval cards: a/d approve/deny without leaving AFK overlay; focus-managed correctly
- [ ] #6 React approval cards: Allow / Deny buttons with optional countdown timer
- [ ] #7 After response, card collapses to a one-line decision summary
- [ ] #8 Multiple pending approvals queue and resolve independently
- [ ] #9 Late-joining browser receives all blocked, approved, and pending events from AfkState in the first afk_update
- [ ] #10 Approval timeout shows a countdown; on expiry SDK applies default-deny and card updates to [TIMED OUT — denied]
- [ ] #11 AFK security events reuse TASK-54 ConversationNodeKind types rather than parallel types
- [ ] #12 Approval cards enforce the 5-second activation delay from security-prompts spec: approve control is disabled and shows a secondary 5-to-0 countdown before becoming active; deny is available immediately
<!-- AC:END -->
