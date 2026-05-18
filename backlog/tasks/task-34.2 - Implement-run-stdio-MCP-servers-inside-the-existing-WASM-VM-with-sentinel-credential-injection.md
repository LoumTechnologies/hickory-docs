---
id: TASK-34.2
title: >-
  Implement: run stdio MCP servers inside the existing WASM VM with sentinel
  credential injection
status: To Do
assignee: []
created_date: '2026-05-18 13:01'
updated_date: '2026-05-18 13:46'
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

Run stdio MCP server processes inside the existing WASM VM (same infrastructure used for LLM scripts) with a fake sentinel credential injected via env var. No new `WasmSandboxExecutor` instances are created per server — this reuses the existing sandbox machinery.

## What changes vs the original design

The original design proposed a `SandboxedStdioMcpClient` that owns its own `WasmSandboxExecutor` per server. This was rejected due to resource overhead (per-server Wasmtime instances running for the full session duration).

The revised approach:
1. At MCP server startup, generate a unique sentinel token: `hick-proxy:<server-name>:<nonce>`
2. Start the stdio MCP server process inside the existing WASM container with the sentinel injected into the declared `credential_env` env var (from TASK-34.1 config)
3. The real credential is stored only in `McpCredentialStore` on the host — it never enters the sandbox
4. The smoltcp network interceptor (TASK-34.3) handles substitution when the server makes outbound calls

`StdioMcpClient` otherwise stays the same — same JSON-RPC framing, same `McpClient` trait, same call sites.

## What "existing WASM VM" means here

MCP server processes are longer-lived than LLM scripts (they stay up for the session). The implementation needs to handle this lifecycle difference: the container for an MCP server is kept alive across multiple JSON-RPC calls, unlike the short-lived script containers.

## References

- `hick-agent-sdk/src/mcp_client.rs` — `StdioMcpClient` to modify
- `hick-agent-core` — `WasmSandboxExecutor`, `WasmSandboxConfig`
- `hick-agent-sdk/src/mcp_config.rs` — per-server sandbox config (TASK-34.1)
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Stdio MCP server process starts inside the existing WASM VM with sentinel token in the declared credential_env
- [ ] #2 Real credential is absent from the container environment
- [ ] #3 The container is kept alive across multiple JSON-RPC calls for the session duration
- [ ] #4 StdioMcpClient JSON-RPC framing is unchanged
- [ ] #5 No new WasmSandboxExecutor is instantiated per MCP server; existing sandbox infrastructure is reused
- [ ] #6 Servers without a sandbox config block start as plain host processes (opt-in, no regression)
<!-- AC:END -->
