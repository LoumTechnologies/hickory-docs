---
id: TASK-34
title: 'MCP Server Sandboxing: run each MCP server in its own isolated WASM sandbox'
status: To Do
assignee: []
created_date: '2026-05-18 13:01'
updated_date: '2026-05-18 13:45'
labels: []
milestone: m-5
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Goal

Sandbox untrusted MCP server processes so that:
1. The LLM cannot access MCP credentials (API keys / PATs)
2. One MCP server cannot read another server's credentials
3. MCP servers with network access are gated per-server through the existing `NetworkFilter` framework
4. The existing `ToolGate` is unaffected — it remains the intent-level gate above `McpClient::call`

## MCP auth reality (from spec research)

There are two completely separate worlds:

**Stdio MCP servers (local, the common case today):**
The MCP spec explicitly says stdio servers SHOULD NOT use OAuth. They read credentials from env vars — a PAT or API key the user created directly in the SaaS (e.g. `GITHUB_PERSONAL_ACCESS_TOKEN`). No OAuth app is involved. This is the case for the GitHub MCP server, Slack, Linear, and most servers in the ecosystem today.

**Remote HTTP MCP servers (hosted, emerging):**
These use OAuth 2.1. The MCP server developer/operator owns the OAuth app in the SaaS. The user just approves it (consent screen). The Hickory Agent acts as the MCP *client* — it needs one Client ID Metadata Document (CIMD) at a URL we control, which becomes the client identity for all remote servers. Auth for these is already handled by `HttpMcpClient` (credentials injected at call time). No sandboxing concern here.

## Architecture decisions

- **No per-server `WasmSandboxExecutor`.** MCP servers run inside the existing WASM VM infrastructure, not in separate Wasmtime instances. This avoids the per-server memory and startup overhead.
- **Credential proxy for stdio servers.** Each stdio MCP server is given a fake sentinel token as its env var (e.g. `GITHUB_PERSONAL_ACCESS_TOKEN=hick-proxy:github:<nonce>`). The smoltcp network interceptor (already used by `hick-agent-mcp-sdk`) recognises the sentinel, strips it, and injects the real credential from `McpCredentialStore`. The real credential never enters the WASM sandbox.
- **Remote HTTP servers are already isolated.** `HttpMcpClient` injects real credentials at call time on the host. No change needed.
- **Per-server NetworkFilter at the interceptor level.** The interceptor that does credential substitution also enforces the per-server allowed-hosts list. Same `NetworkFilter` framework, applied inside the interceptor rather than as a separate container boundary.
- **ToolGate is unaffected.** It sits above `McpClient::call`. Sandboxing is a change below the gate.

## Subtasks

See child tasks for implementation breakdown.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Stdio MCP servers receive a fake sentinel credential via env var; the real credential never appears in the WASM sandbox
- [ ] #2 The smoltcp interceptor recognises sentinel tokens, substitutes real credentials, and enforces per-server allowed-hosts rules
- [ ] #3 Remote HTTP MCP servers (HttpMcpClient) are unchanged — they already handle credential isolation on the host
- [ ] #4 No per-server WasmSandboxExecutor is created; MCP servers run in the existing WASM VM infrastructure
- [ ] #5 ToolGate continues to gate all three tool-calling modes unchanged
- [ ] #6 Per-server network and credential config is declarative (TOML) and co-located with the existing mcp-servers config
<!-- AC:END -->
