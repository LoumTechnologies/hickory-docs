---
id: TASK-34.5
title: >-
  Audit: verify ToolGate covers all three tool-calling modes under sandboxed MCP
  servers
status: To Do
assignee: []
created_date: '2026-05-18 13:02'
labels: []
milestone: m-5
dependencies:
  - TASK-34.2
parent_task_id: TASK-34
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Goal

Confirm that `ToolGate::check` is called before `McpClient::call` for all three tool-calling modes when MCP servers are sandboxed. The sandboxing change is at the `McpClient` implementation layer — below the gate — so the gate should be unaffected, but this needs explicit verification and test coverage.

## The three modes

1. **`hick-agent-mcp-wasm`** — generates a WASI component per tool; the component is intercepted by `McpWasmContainerBackend` which calls `ToolGate::check` then `McpClient::call`. Verify the gate path in `McpWasmContainerBackend`.

2. **`hick-agent-mcp-sdk`** — magic-IP stub interceptor (`McpSdkInterceptor`). Verify `ToolGate::check` is called in the interceptor before dispatching.

3. **Direct `McpClient::call`** — used when the react loop or agent code calls tools directly (e.g. in `hick-agent-hick-edit`). Verify the call site checks `ToolGate` first.

## Expected outcome

All three modes already go through `ToolGate`. This task is verification + documentation + regression tests, not implementation. If a gap is found, fix it as part of this task.

## References

- `hick-agent-mcp-wasm/src/lib.rs` — `McpWasmContainerBackend`
- `hick-agent-mcp-sdk/src/lib.rs` — `McpSdkInterceptor`
- `hick-agent-sdk/src/tool_gate.rs` — `ToolGate`
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Each of the three tool-calling modes has a test that confirms a Block verdict from ToolGate prevents McpClient::call from being reached
- [ ] #2 If any mode lacks a ToolGate check, it is added
- [ ] #3 Findings documented in task notes regardless of outcome
<!-- AC:END -->
