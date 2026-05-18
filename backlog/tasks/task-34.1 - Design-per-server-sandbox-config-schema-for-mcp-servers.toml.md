---
id: TASK-34.1
title: 'Design: per-server sandbox config schema for mcp-servers.toml'
status: To Do
assignee: []
created_date: '2026-05-18 13:01'
labels: []
milestone: m-5
dependencies: []
parent_task_id: TASK-34
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Goal

Extend the MCP server configuration schema (currently in `mcp_config.rs`) with per-server sandbox declarations: whether to sandbox, which WASM image to use, network rules, and file-access patterns.

## Design questions to resolve

1. **Sandbox mode** — should `sandbox = true/false` default to `true` for new servers, or opt-in? Recommendation: default to `true` (fail-secure) with an explicit `sandbox = false` escape hatch for trusted/internal servers.

2. **Image selection** — use the same WASM images as the LLM script sandbox (alpine), or allow per-server image override?

3. **Network rules** — inline in `mcp-servers.toml` as a `[servers.my-server.network]` section, or a separate `mcp-server-network.toml`? Recommendation: inline, matching how `AgentSecurityConfig` and `NetworkFilterConfig` are composed in `security_config.rs`.

4. **File access** — MCP servers likely need read access to the project directory. Should this be declared per-server or defaulted?

## Proposed schema sketch

```toml
[[servers]]
name = "github"
transport = "stdio"
command = ["npx", "-y", "@modelcontextprotocol/server-github"]

[servers.sandbox]
enabled = true
image = "alpine"          # optional, defaults to agent default
deny_all_network = true
allowed_hosts = [
  { host = "api.github.com", port = "443" },
  { host = "github.com",     port = "443" },
]
file_access.read  = ["/workspace/**"]
file_access.write = []
```

## References

- `hick-agent-sdk/src/mcp_config.rs` — current schema
- `hick-agent-engine/src/security_config.rs` — `AgentSecurityConfig` pattern to follow
- `hick-agent-engine/src/network_filter.rs` — `NetworkFilterConfig`
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 TOML schema is documented in code with examples (doc comments on structs)
- [ ] #2 Defaults to sandbox=true when field is omitted
- [ ] #3 network rules inline in the server block, reusing NetworkFilterConfig field layout
- [ ] #4 Schema is validated on load with clear error messages for missing required fields
- [ ] #5 Existing configs without sandbox block continue to work (backwards compatible: sandbox=false default until feature is stabilised, then flip)
<!-- AC:END -->
