---
id: TASK-34.3
title: >-
  Implement: credential injection via WASM host functions for sandboxed MCP
  servers
status: To Do
assignee: []
created_date: '2026-05-18 13:02'
labels: []
milestone: m-5
dependencies:
  - TASK-34.2
parent_task_id: TASK-34
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Goal

Ensure MCP server credentials are injected into the sandboxed server process via WASM host functions — not via environment variables or files inside the container — so that neither the LLM scripts nor other MCP servers can observe them.

## Background

`McpCredentialStore` already prevents LLM scripts from seeing credentials: HTTP credentials are injected at call time in `HttpMcpClient::call`. For `SandboxedStdioMcpClient`, the MCP server process itself needs its credentials. The challenge: many MCP servers read secrets from env vars (e.g. `GITHUB_TOKEN`). If those env vars are set on the container, they are visible to anything running inside it — including code the LLM writes that the server might exec.

## Approach options

1. **Short-lived env var injection per-call** — set the env var only for the duration of the spawned process, ensuring it's not visible to LLM script containers (different containers). This is the simplest approach and probably sufficient since each MCP server has its own container.

2. **Host-function credential broker** — the sandboxed server calls a WIT import `get_credential(name)` rather than reading env vars. Requires modifying or wrapping the MCP server binary, which is impractical for third-party servers.

3. **Sealed env at spawn time, not inherited** — spawn the MCP server container with an explicit env (not inherited from the agent process), with only the server's own credentials.

**Recommendation**: Option 3 (sealed env at spawn time). The container is already isolated; the concern is that the LLM can't read those env vars via its own container (different container) and that server A can't see server B's env (different containers). Sealed env at spawn time satisfies both without requiring server modifications.

## Acceptance criteria focus

The goal is: given two MCP servers each with different credentials, running one server's container cannot reveal the other server's credentials.

## References

- `hick-agent-sdk/src/mcp_config.rs` — `McpCredentialStore`
- `hick-agent-mcp-sdk/src/lib.rs` — the existing host-function credential pattern to reference
- TASK-34.2 — `SandboxedStdioMcpClient`
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 MCP server container is spawned with a sealed env containing only its own credentials (not the full agent process env)
- [ ] #2 LLM script containers have no way to observe MCP server env vars (different container, different sandbox instance)
- [ ] #3 Server A's sandbox cannot access server B's credentials
- [ ] #4 Credential injection path is documented in `SandboxedStdioMcpClient`
- [ ] #5 Test: two servers with different tokens; confirm neither container can read the other's token
<!-- AC:END -->
