---
id: TASK-34.2
title: 'Implement: SandboxedStdioMcpClient — spawn MCP server inside WASM container'
status: To Do
assignee: []
created_date: '2026-05-18 13:01'
labels: []
milestone: m-5
dependencies:
  - TASK-34.1
parent_task_id: TASK-34
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Goal

Add a `SandboxedStdioMcpClient` in `hick-agent-sdk` that wraps `WasmSandboxExecutor` to spawn the MCP server process inside an isolated WASM container instead of as a plain host child process.

## Approach

`StdioMcpClient` today calls `tokio::process::Command` directly. `SandboxedStdioMcpClient` should instead:
1. Use `WasmSandboxExecutor` (from `hick-agent-core`) to launch the server's command inside a WASM container
2. Expose the same `McpClient` trait so call sites are unchanged
3. Wire stdin/stdout through the container's I/O — same JSON-RPC framing as today
4. Accept a `WasmSandboxConfig` derived from the per-server sandbox config (TASK-34.1)

## Key design constraint

Each server instance gets **its own** `WasmSandboxExecutor` / container — not a shared sandbox. This is what provides the inter-server isolation guarantee.

## References

- `hick-agent-sdk/src/mcp_client.rs` — `StdioMcpClient` to use as a model
- `hick-agent-core` — `WasmSandboxExecutor`, `WasmSandboxConfig`, `ContainerCapabilities`
- `hick-agent-engine/src/security_config.rs` — how `AgentSecurityConfig::to_capabilities()` produces `ContainerCapabilities`
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 SandboxedStdioMcpClient implements McpClient trait
- [ ] #2 Each instance owns its own WasmSandboxExecutor (no sharing between server instances)
- [ ] #3 JSON-RPC framing is identical to StdioMcpClient — same wire protocol
- [ ] #4 ContainerCapabilities are derived from the per-server sandbox config block
- [ ] #5 Unit tests confirm the client can call tools/list and tools/call against a mock server run inside the container
<!-- AC:END -->
