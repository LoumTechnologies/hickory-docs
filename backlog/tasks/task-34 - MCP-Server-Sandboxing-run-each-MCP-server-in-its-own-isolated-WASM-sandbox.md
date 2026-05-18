---
id: TASK-34
title: 'MCP Server Sandboxing: run each MCP server in its own isolated WASM sandbox'
status: To Do
assignee: []
created_date: '2026-05-18 13:01'
labels: []
milestone: m-5
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Goal

Run each configured MCP server in its own WASM sandbox so that:
1. The LLM cannot access MCP credentials (already true via `McpCredentialStore`, but the server process itself needs to be confined too)
2. One MCP server cannot read another server's credentials or environment
3. MCP servers with network access are gated through the existing `NetworkFilter` framework
4. The existing `ToolGate` (which already blocks calls before `McpClient::call`) is unaffected — it remains the intent-level gate

## Background

Currently `StdioMcpClient` spawns the MCP server as a plain host process with full host access. `HttpMcpClient` connects to a remote endpoint with credentials injected at call time (already isolated). The WASM sandbox used for LLM scripts is the right model to apply here.

## Architecture decisions already settled

- `ToolGate` is NOT made obsolete by per-server network filtering. They operate at different layers: ToolGate at intent/classification, NetworkFilter at transport/host. Both are needed.
- Sandboxing the MCP server process is a change at the `StdioMcpClient` layer — below `ToolGate`. No changes to any tool-calling mode (WASM component injection, magic-IP stubs, direct call) are needed.
- Per-server `NetworkFilterConfig` reuses the existing framework; each server gets its own config section.
- Credentials must be injected via WASM host functions (not env vars), matching the `hick-agent-mcp-sdk` pattern.

## Subtasks

See child tasks for implementation breakdown.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Each stdio MCP server runs inside its own WASM container (one container per server name)
- [ ] #2 No MCP server can read env vars or files belonging to another server's sandbox
- [ ] #3 MCP servers requiring network access use a per-server NetworkFilter config (allow/block/prompt by host)
- [ ] #4 Credentials are injected via WASM host functions, not env vars or sandbox-visible files
- [ ] #5 ToolGate continues to gate all three tool-calling modes unchanged
- [ ] #6 Per-server sandbox config is declarative (TOML) and co-located with the existing mcp-servers config
<!-- AC:END -->
