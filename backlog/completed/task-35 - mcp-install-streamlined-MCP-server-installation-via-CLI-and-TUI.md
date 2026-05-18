---
id: TASK-35
title: 'mcp install: streamlined MCP server installation via CLI and TUI'
status: Done
assignee:
  - Nate Loum
created_date: '2026-05-18 17:56'
updated_date: '2026-05-18 19:33'
labels: []
milestone: m-6
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Goal

Provide a single, uniform way to install MCP servers that works:
- As a standalone CLI subcommand (`iron-peak mcp install`, `copper-ledge mcp install`)
- As a TUI slash command (`/mcp install`) inside a running agent session

The config written by `mcp install` is the same `.hick/mcp-servers.toml` / `.hick/mcp-secrets.toml` that all agents and all extension modes (GraphQL fenced blocks, Python SDK, XML dispatcher, etc.) already read. No per-agent or per-extension install path.

## Interface

```
# stdio — command is the full invocation; name inferred from last path segment or prompted
iron-peak mcp install "npx -y @modelcontextprotocol/server-github"
iron-peak mcp install "uvx mcp-server-fetch" --name fetch

# remote HTTP JSON-RPC
iron-peak mcp install https://api.example.com/mcp --name example

# trust level (default: prompt)
iron-peak mcp install "npx -y @modelcontextprotocol/server-github" --trust trusted

# TUI (idle only — not during an active agent turn)
/mcp install "npx -y @modelcontextprotocol/server-github"
```

## Install flow

1. Parse spec → infer transport (URL = HTTP, otherwise stdio)
2. Infer server name from last path segment, or prompt if ambiguous
3. For stdio: detect package manager from command prefix (`npx`, `uvx`, `pip`, `cargo`, etc.) and run package install in WASM VM under a "package registry" network policy (npmjs.org, pypi.org, etc.)
4. Connect and call `tools/list` to verify the server works; surface the tool list to the user
5. Write entry to `.hick/mcp-servers.toml`
6. Prompt for credentials interactively → write to `.hick/mcp-secrets.toml`
7. Print summary of installed tools

## Architecture

- Install logic lives in a new `mcp_install` module in `hick-agent-sdk` (not `hick-mcp`, which is the MCP server the agent itself exposes)
- CLI subcommand: each binary adds `Mcp { #[command(subcommand)] cmd: McpSubcommand }` to its CLI enum, delegates to shared installer
- TUI: `/mcp install ...` is a new slash-command prefix alongside the existing `!` direct-command prefix; credential prompting uses the same channel-based approval pattern as network approval prompts

## Subtasks

See child tasks.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 `iron-peak mcp install <spec>` and `copper-ledge mcp install <spec>` work as standalone CLI commands with no running agent
- [ ] #2 `/mcp install <spec>` works inside the TUI when the agent is idle
- [ ] #3 HTTP and stdio transport are both supported; transport is inferred from the spec
- [ ] #4 For stdio servers, the package is installed in the WASM VM before the config entry is written
- [ ] #5 Server connection is verified (`tools/list`) before the config entry is written; install fails fast if the server doesn't respond
- [ ] #6 Config is written to `.hick/mcp-servers.toml` and credentials to `.hick/mcp-secrets.toml` — the same files all agents and extension modes already read
- [ ] #7 Credential prompting works in both CLI (blocking stdin) and TUI (channel-based prompt overlay, same pattern as network approval)
- [ ] #8 All existing tests continue to pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
## Implementation Plan

### Files changed
1. `hick-agent-sdk/src/mcp_install.rs` — new core module (spec parsing, installer, list/remove helpers)
2. `hick-agent-sdk/src/lib.rs` — add `pub mod mcp_install` + pub use exports
3. `hick-agent-script-first/src/events.rs` — add `McpInstallProgress`, `McpInstallComplete` to AgentEvent
4. `iron-peak/src/main.rs` — add `Mcp` subcommand to Commands enum, CLI handlers, update run_tui call
5. `copper-ledge/src/main.rs` — same (add optional Commands enum with Mcp variant)
6. `iron-peak/src/tui.rs` — update run_tui signature (add event_tx, project_dir), add /mcp slash handling + credential overlay
7. `copper-ledge/src/tui.rs` — same

### Channel design for TUI credential prompting
- `McpCredentialRequest` struct (in mcp_install.rs) carries server name, suggested env var, oneshot::Sender
- Channel created inside `run_tui`; Sender cloned for each spawned install task
- Install task spawned via `tokio::runtime::Handle::current().spawn()` from spawn_blocking TUI
- TUI renders credential overlay (same pattern as network approval) via pending_mcp_credential state field

### CLI install flow
- Optional `--images-dir` / `--default-image`; if present, run WASM package install; if absent, skip (server must be pre-installed)
- Blocking stdin prompts for name (if inferred as None) and credential env var
- Verify connection, write config, write credentials

### Subtask sequence
- 35.1 first (core module), then 35.2 and 35.3 in parallel (both depend on 35.1)
<!-- SECTION:PLAN:END -->
