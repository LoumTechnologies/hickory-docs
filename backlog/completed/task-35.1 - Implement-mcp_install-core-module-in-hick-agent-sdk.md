---
id: TASK-35.1
title: 'Implement: mcp_install core module in hick-agent-sdk'
status: Done
assignee:
  - Nate Loum
created_date: '2026-05-18 17:57'
updated_date: '2026-05-18 19:33'
labels: []
milestone: m-6
dependencies: []
parent_task_id: TASK-35
priority: high
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add `hick-agent-sdk/src/mcp_install.rs` with the shared installer logic used by both the CLI subcommand and the TUI slash command.

## Responsibilities

- `McpInstallSpec`: parsed from a raw string — either `McpInstallSpec::Stdio { command: Vec<String> }` or `McpInstallSpec::Http { url: String }`
- `parse_spec(raw: &str) -> McpInstallSpec`: URL (starts with `http://` or `https://`) → Http; everything else → Stdio (shell-split the string)
- `infer_name(spec: &McpInstallSpec) -> Option<String>`: last path segment of npm package name (`@scope/server-github` → `github`), last segment of URL host, etc. Returns `None` when ambiguous so the caller can prompt.
- `detect_package_manager(command: &str) -> PackageManager`: returns `Npm`, `Uvx`, `Pip`, `Cargo`, or `Unknown` based on the first token (`npx`/`npm`, `uvx`, `pip`/`pip3`, `cargo`)
- `PackageManagerNetworkPolicy`: a `McpSandboxConfig`-compatible allowed-hosts list covering the relevant package registries (registry.npmjs.org, pypi.org, files.pythonhosted.org, static.crates.io, crates.io) — used only during install, not at runtime
- `McpInstaller`: async struct that drives the install sequence:
  1. `run_package_install(spec, sandbox_executor) -> Result<()>` — for stdio only; builds the install command (e.g. `npm install -g <pkg>`) and runs it in the WASM VM under `PackageManagerNetworkPolicy`; no-op for Http
  2. `verify_connection(spec, creds) -> Result<Vec<McpToolDef>>` — spawns a `StdioMcpClient` or `HttpMcpClient`, calls `list_tools()`, returns the tool list; fails if the server doesn't respond
  3. `write_config(spec, name, trust, project_dir) -> Result<()>` — appends a `[[servers]]` entry to `.hick/mcp-servers.toml` (creates file if absent); idempotent on name (errors if name already exists)
  4. `write_credentials(name, creds: Vec<(String, String)>, project_dir) -> Result<()>` — appends entries to `.hick/mcp-secrets.toml`

## What this module does NOT do

- Credential prompting (I/O strategy differs between CLI and TUI — callers handle that)
- Name prompting (same reason)
- Anything TUI- or CLI-specific

## Tests

- `parse_spec` handles URLs, bare commands, quoted commands, `npx -y` flags
- `infer_name` extracts expected names from npm package paths and URLs
- `detect_package_manager` covers all supported prefixes and `Unknown`
- `write_config` round-trips through `McpServersConfig::from_str` and produces valid TOML
- `write_config` returns an error when a server with the same name already exists
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 All functions above are implemented and `pub(crate)` where internal, `pub` only where needed by CLI/TUI callers
- [ ] #2 `write_config` produces TOML parseable by `McpServersConfig`
- [ ] #3 `run_package_install` uses `PackageManagerNetworkPolicy` (not open network) for the WASM install step
- [ ] #4 Unit tests pass for parse, infer, detect, write_config round-trip, and duplicate-name error
<!-- AC:END -->
