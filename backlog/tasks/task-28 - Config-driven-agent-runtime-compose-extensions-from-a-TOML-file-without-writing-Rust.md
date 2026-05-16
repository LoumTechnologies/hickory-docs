---
id: TASK-28
title: >-
  Config-driven agent runtime: compose extensions from a TOML file without
  writing Rust
status: To Do
assignee: []
created_date: '2026-05-16 13:04'
updated_date: '2026-05-16 13:04'
labels:
  - sdk
  - extensions
  - config
  - dx
milestone: m-3
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Today, wiring a new agent binary requires writing Rust: instantiate extensions, call builder methods, compile. This task makes extension composition declarative — an operator specifies which extensions to activate and how to configure them in a TOML file, and a pre-built host binary reads that config at startup and wires the runtime accordingly.

## Motivation

The extension architecture (hick-agent-sdk + hick-agent-* crates) is designed for composability, but composing currently requires a Rust crate (e.g. copper-ledge, iron-peak). Eval work (TASK-23) and MCP config work (TASK-24.7) both need per-deployment config knobs that should not require recompiling a binary. A config-driven runtime lets new agent configurations be deployed by editing a TOML file.

## Config schema — `.hick/agent.toml`

```toml
[agent]
model = "claude-opus-4-7"
system_prompt_extra = ".hick/system-prompt.md"   # optional append

[extensions]
script_first = true       # hick-agent-script-first
hick_edit    = true       # hick-agent-hick-edit
mcp_wasm     = true       # hick-agent-mcp-wasm
mcp_sdk      = true       # hick-agent-mcp-sdk
mcp_xml      = false
mcp_graphql  = true

[mcp]
servers_file = ".hick/mcp-servers.toml"
secrets_file = ".hick/mcp-secrets.toml"

[mcp.response_format]
default = "json"

[mcp.response_format.by_shape]
list   = "xml"
nested = "markdown"

[security]
config_file = ".hick/agent-security.toml"
```

## Implementation approach

A single "universal" agent binary (e.g. `hick-agent`) is compiled with all extension crates linked in. At startup it reads `.hick/agent.toml`, constructs the `AgentRuntime` with only the enabled extensions, and runs. No dynamic linking, no code generation — feature activation is a runtime branch, not a compile-time gate.

This is safe because extension crates are already designed to be opt-in via the builder pattern; disabling one simply means not calling its `register()` method.

## Scope boundaries
- One universal binary per release; the adjective-noun binaries (copper-ledge, iron-peak) remain valid for A/B eval purposes where a fixed wiring is intentional.
- Config schema versioning: add a `[agent] config_version = 1` field from the start.
- Hot-reload of `.hick/agent.toml` is out of scope for v1 (requires restart).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A universal hick-agent binary starts with all extension crates linked; which extensions activate is determined solely by .hick/agent.toml.
- [ ] #2 Enabling/disabling an extension in the TOML and restarting produces the expected runtime behaviour (integration test for at least script_first and mcp_graphql).
- [ ] #3 All config sections defined in the schema (agent, extensions, mcp, mcp.response_format, security) deserialize correctly; unknown keys produce a clear error.
- [ ] #4 An agent.toml with only [agent] and no [extensions] table starts with zero extensions and does not panic.
- [ ] #5 The universal binary accepts a --config flag to point at a non-default agent.toml path.
- [ ] #6 config_version field is present and validated; a mismatched version emits a warning but does not hard-fail for v1.
- [ ] #7 Existing adjective-noun binaries (copper-ledge, iron-peak) are unaffected — they continue to compile and run with their hardwired extension sets.
<!-- AC:END -->
