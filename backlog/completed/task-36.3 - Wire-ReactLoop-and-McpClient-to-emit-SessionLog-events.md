---
id: TASK-36.3
title: Wire ReactLoop and McpClient to emit SessionLog events
status: Done
assignee:
  - nate@loumtechnologies.com
created_date: '2026-05-19 11:01'
updated_date: '2026-05-19 11:32'
labels:
  - agent
  - logging
dependencies:
  - TASK-36.2
parent_task_id: TASK-36
priority: high
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Thread a `SessionLog` handle through `ReactLoop` (in `hick-agent-engine`) and `McpClient` call sites so the SDK and extension crates own logging — not the binary crates.

## Depends on

TASK-36.2 — provides `XmlSessionLog`, `NullSessionLog`, and the full `LogEvent` enum.

## ReactLoop changes

`ReactLoop` gains a `session_log: Arc<dyn SessionLog>` field (defaults to `NullSessionLog`). Builder method: `with_session_log(log: Arc<dyn SessionLog>)`.

The loop calls `session_log.record(...)` at each significant point:
- `SessionStart` on first `run()` call (include agent config: model, sandbox image, max_turns)
- `UserTurn` when a user message is appended to history
- `StateTransition` on every state change driven by `step()`
- `LlmResponse` after each accepted LLM response
- `ActionStart` / `ActionEnd` wrapping each script execution
- `SecurityViolation` when policy denies execution
- `SessionEnd` when `run()` returns (Done or Error)

## McpClient changes

`McpClient::call()` currently has no logging. Add `McpCall` / `McpResponse` events via a logging wrapper:

```rust
pub struct LoggingMcpClient<C: McpClient> {
    inner: C,
    server_name: String,
    log: Arc<dyn SessionLog>,
}
```

`LoggingMcpClient` implements `McpClient`, records `McpCall` before delegating and `McpResponse` (or an error observation) after. Binary crates wrap their clients with `LoggingMcpClient` when constructing the agent — this requires no changes to `McpClient` trait itself.

## hick-agent-script-first

`hick-agent-script-first/src/react_loop.rs` has its own `ReactLoop` variant. Apply the same `session_log` field and recording calls there.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 ReactLoop gains with_session_log() builder method; defaults to NullSessionLog so existing call sites compile unchanged
- [ ] #2 SessionStart, UserTurn, StateTransition, LlmResponse, ActionStart, ActionEnd, SecurityViolation, and SessionEnd are all recorded at the correct points in the ReactLoop
- [ ] #3 LoggingMcpClient<C> wrapper implements McpClient, records McpCall before and McpResponse after each delegated call
- [ ] #4 hick-agent-script-first ReactLoop receives the same session_log field and recording calls
- [ ] #5 No session log writing remains in the TUI or binary crate layers after this task
<!-- AC:END -->
