---
id: TASK-34.1
title: 'Design: per-server sandbox config schema for mcp-servers.toml'
status: Done
assignee: []
created_date: '2026-05-18 13:01'
updated_date: '2026-05-18 17:30'
labels: []
milestone: m-5
dependencies: []
parent_task_id: TASK-34
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Goal

Extend the MCP server configuration schema (currently in `mcp_config.rs`) with per-server sandbox declarations covering: which env var name carries the credential, the sentinel token prefix to use, and network access rules.

## Context

Only **stdio servers** need this config. Remote HTTP servers are already handled by `HttpMcpClient` (credentials injected at call time on the host via `McpCredentialStore`). The sandbox for stdio servers runs inside the existing WASM VM — no separate container config or image selection is needed.

## Design questions to resolve

1. **Env var mapping** — how does the config declare which env var name the server reads its credential from? The sentinel token needs to be injected under the right name (e.g. `GITHUB_PERSONAL_ACCESS_TOKEN`, not a generic name).

2. **Network rules** — inline in `mcp-servers.toml` as a `[servers.my-server.network]` section. Reuse `NetworkFilterConfig` field layout from `security_config.rs`.

3. **Servers that don't use bearer auth** — some servers may use custom auth header names or non-bearer schemes. Config should allow declaring the header name (default: `Authorization: Bearer`).

## Proposed schema sketch

```toml
[[servers]]
name = "github"
transport = "stdio"
command = ["npx", "-y", "@modelcontextprotocol/server-github"]

[servers.sandbox]
# env var the server reads its credential from
credential_env = "GITHUB_PERSONAL_ACCESS_TOKEN"
# optional: auth header the server sends (default: "Authorization: Bearer {token}")
auth_header = "Authorization"
auth_scheme = "Bearer"
# network rules applied by the interceptor
deny_all_network = true
allowed_hosts = [
  { host = "api.github.com", port = "443" },
]
```

## References

- `hick-agent-sdk/src/mcp_config.rs` — current schema
- `hick-agent-engine/src/security_config.rs` — `AgentSecurityConfig` pattern to follow
- `hick-agent-engine/src/network_filter.rs` — `NetworkFilterConfig`
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 TOML schema is documented in code with examples (doc comments on structs)
- [x] #2 credential_env field declares which env var name receives the sentinel token
- [x] #3 network rules inline in the server block, reusing NetworkFilterConfig field layout
- [x] #4 Schema is validated on load with clear error messages for missing required fields
- [x] #5 Existing configs without sandbox block continue to work (sandbox is opt-in per server)
- [x] #6 Remote HTTP servers (transport = "http") are not affected by this config block
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added `McpSandboxAllowedHost` and `McpSandboxConfig` structs to `mcp_config.rs` with full doc comments and TOML examples. Added `sandbox: Option<McpSandboxConfig>` to `McpServerConfig`. Schema validated on load via serde (missing `credential_env` produces a clear toml parse error). Existing configs without sandbox block deserialize to `sandbox: None` with no regression. Added 5 tests covering schema loading, opt-in absence, host filtering, wildcard port, and deny-all-with-allowlist.
<!-- SECTION:FINAL_SUMMARY:END -->
