---
id: TASK-34.5
title: >-
  Audit: verify ToolGate covers all three tool-calling modes under sandboxed MCP
  servers
status: Done
assignee: []
created_date: '2026-05-18 13:02'
updated_date: '2026-05-18 17:30'
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
- [x] #1 Each of the three tool-calling modes has a test that confirms a Block verdict from ToolGate prevents McpClient::call from being reached
- [x] #2 If any mode lacks a ToolGate check, it is added
- [x] #3 Findings documented in task notes regardless of outcome
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
## Audit Findings

All three tool-calling modes already call `ToolGate::check` before `McpClient::call`. No gaps found.

**Mode 1: `McpWasmContainerBackend`** (`hick-agent-mcp-wasm/src/lib.rs`)  
`execute_raw()` → `McpWasmExtension::call_tool()` → `tool_gate.check()` → `client.call()`. Existing test `call_tool_blocked_on_secret_classification` covers the Block path.

**Mode 2: `McpSdkInterceptor`** (`hick-agent-mcp-sdk/src/lib.rs`)  
`dispatch()` → `tool_gate.check()` → (if block: return 403) → `client.call()`. Added test `sdk_interceptor_block_verdict_prevents_mcp_call` using a spy McpClient to verify call is not reached.

**Mode 3: `McpXmlExtension`** (`hick-agent-mcp-xml/src/lib.rs`)  
`dispatch_element()` → `tool_gate.check()` → (if block: return blocked result) → `client.call()`. Added test `xml_extension_block_verdict_prevents_mcp_call` using a spy McpClient.

Note: task description references `hick-agent-hick-edit` as the "direct call" mode, but examining the code confirms `HickEditAgent` does not call `McpClient` directly — it only wraps `AgentRuntime` for hick-XML pipeline editing. The three modes verified are WASM/McpWasmContainerBackend, SDK/McpSdkInterceptor, and XML/McpXmlExtension (all confirmed ToolGate-gated).
<!-- SECTION:NOTES:END -->
