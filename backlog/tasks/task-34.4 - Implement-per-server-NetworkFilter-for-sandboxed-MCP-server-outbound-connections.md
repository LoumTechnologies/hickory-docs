---
id: TASK-34.4
title: >-
  Implement: per-server NetworkFilter for sandboxed MCP server outbound
  connections
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

Apply `NetworkFilter` to each sandboxed MCP server's WASM container so its outbound network connections are gated by the same allow/block/prompt rules used for LLM script sandboxes.

## Why this is not the same as the existing NetworkFilter

The existing `NetworkFilter` in `copper-ledge/src/main.rs` wraps the LLM client (`FilteredLlmClient`) and gates the agent's outbound call to the LLM API. It is NOT applied to MCP server processes.

The new per-server filter gates network connections made by the MCP server process itself (e.g. a GitHub MCP server calling `api.github.com`).

## Approach

`NetworkFilter` already supports derivation from `ContainerCapabilities::from_capabilities(...)`. The per-server sandbox config (TASK-34.1) declares `allowed_hosts` and `deny_all_network`. Wire these into the `WasmSandboxConfig` for the MCP server container exactly as the agent-level security config wires into the LLM script sandbox in `copper-ledge/src/main.rs`.

The per-server filter's approval channel should route to the same TUI approval handler — the user sees a prompt like "GitHub MCP server wants to connect to api.github.com — allow?".

## Note on ToolGate relationship

This network filter does NOT replace `ToolGate`. `ToolGate` prevents the LLM from invoking certain tools at all (based on data classification). This filter prevents an invoked tool's server process from exfiltrating data to unexpected hosts. They are complementary.

## References

- `hick-agent-engine/src/network_filter.rs` — `NetworkFilter`, `NetworkFilterConfig`, `NetworkApprovalRequest`
- `hick-agent-sdk/src/network_filter.rs` — same, in SDK crate
- `copper-ledge/src/main.rs` lines ~318–355 — how per-agent NetworkFilter is wired today
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Each sandboxed MCP server container uses a NetworkFilter derived from its per-server allowed_hosts config
- [ ] #2 deny_all_network=true (or no allowed_hosts declared) blocks all outbound connections from the server container
- [ ] #3 Network approval prompts for MCP servers are distinguishable from LLM API approval prompts in the TUI
- [ ] #4 An MCP server configured with allowed_hosts=[api.github.com] cannot connect to api.openai.com
- [ ] #5 NetworkFilter approval responses are cached per (server-name, host) for the session
<!-- AC:END -->
