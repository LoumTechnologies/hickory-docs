---
id: TASK-24.1
title: >-
  MCP client core in hick-agent-sdk: server config, auth management, JSON-RPC
  client, tool manifest
status: Done
assignee: []
created_date: '2026-05-16 12:22'
updated_date: '2026-05-17 11:07'
labels:
  - mcp
  - security
  - sdk
milestone: m-3
dependencies: []
references:
  - hick-agent/crates/hick-agent-sdk/src/security_config.rs
  - hick-agent/crates/hick-agent-sdk/src/network_filter.rs
  - hick-agent/crates/hick-agent-sdk/src/sink_guard.rs
  - hick-agent/crates/hick-agent-sdk/src/lib.rs
parent_task_id: TASK-24
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add MCP client primitives to `hick-agent-sdk` so any extension crate or binary can connect to MCP servers and invoke tools. This is the foundation all four exposure-mode extensions (TASK-24.3–24.6) depend on.

## Scope

### Config schema — `.hick/mcp-servers.toml`

```toml
[[servers]]
name = "filesystem"
transport = "stdio"           # or "http"
command = ["mcp-server-filesystem", "--root", "/workspace"]

[[servers]]
name = "github"
transport = "http"
url = "http://localhost:3001/mcp"
# auth is in a separate secrets file — never here
```

Auth credentials live in `.hick/mcp-secrets.toml` (gitignored) or come from environment variables. The config schema must make the split explicit — no inline secrets in `mcp-servers.toml`.

### Auth credential management

- Support: bearer token, API key header, basic auth, env-var indirection (`env:GITHUB_TOKEN`).
- Credentials loaded into a `McpCredentialStore` struct held in the SDK.
- `McpCredentialStore` is never serialized, never written to disk, never included in `Message` content passed to the LLM.
- All four extension crates receive credentials via a Rust `Arc<McpCredentialStore>` — credentials do not cross any serialization boundary (no JSON, no TOML round-trips after load).

### JSON-RPC 2.0 client

- Support both transports:
  - **stdio** — spawn child process, write/read newline-delimited JSON over stdin/stdout.
  - **HTTP** — POST to the server URL, use `reqwest` (already a workspace dependency if present, or add it).
- `McpClient` trait with `async fn call(&self, method: &str, params: serde_json::Value) -> anyhow::Result<serde_json::Value>`.
- Connection lifecycle: lazy connect on first call, reconnect on stdio process exit.

### Tool manifest loading

- `tools/list` → `Vec<McpToolDef>` with name, description, JSON Schema for input and output.
- `McpToolDef` is the canonical representation used by all extension crates to generate wasm stubs, SDK files, GraphQL types, or XML element docs.
- Cache manifest in memory per session; expose a `refresh()` method.

### Integration with `AgentSecurityConfig`

- `AgentSecurityConfig` gains an optional `[[mcp_servers]]` stanza that references server names from `mcp-servers.toml`.
- Existing `to_capabilities()` and `to_sink_defs()` are not changed; the new MCP layer sits alongside them.

## What this task does NOT do
- Does not implement the tool gate (TASK-24.2).
- Does not generate any WASI, SDK, XML, or GraphQL artifacts (TASK-24.3–24.6).
- Does not expose tools to the LLM in any form.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 McpServerConfig and McpSecretsConfig are separate structs; secrets file path is gitignored by default in new projects.
- [x] #2 McpCredentialStore holds loaded credentials; it has no Serialize/Deserialize impl and no Display impl — no accidental leakage path.
- [x] #3 McpClient trait is implemented for both stdio and HTTP transports with a shared test harness (mock server or fixture).
- [x] #4 tools/list is fetched on connect and cached; McpToolDef carries name, description, and input/output JSON Schema.
- [x] #5 Stdio transport reconnects automatically when the child process exits; test covers restart-on-exit.
- [x] #6 HTTP transport injects auth headers at call time using credentials from McpCredentialStore — never from serialized config.
- [x] #7 Unit tests cover: credential env-var indirection, tools/list parsing, stdio reconnect, HTTP auth header injection.
- [x] #8 McpClient is pub-use'd from hick-agent-sdk lib.rs with full doc comments on all public items.
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added MCP client core to `hick-agent-sdk` in three new source files:

- `mcp_config.rs`: `McpServersConfig`/`McpServerConfig` (TOML, safe to commit), `McpCredentialStore` (no Serialize/Deserialize/Display), `ResolvedCredential` (bearer, API key, basic, env-var indirection). Credentials are held in-memory behind `Arc<McpCredentialStore>` and never cross a serialization boundary.
- `mcp_client.rs`: `McpClient` trait with `call`/`list_tools`/`refresh_tools`; `StdioMcpClient` (spawns child process, reconnects automatically on exit); `HttpMcpClient` (reqwest POST, injects auth at call time from `McpCredentialStore`). Tool manifest cached per session.
- Updated `lib.rs` with full `pub use` facade and doc comments for all new public items.

All 8 acceptance criteria verified by passing tests (credential env-var indirection, tools/list parsing, stdio reconnect, HTTP auth header injection).
<!-- SECTION:FINAL_SUMMARY:END -->
