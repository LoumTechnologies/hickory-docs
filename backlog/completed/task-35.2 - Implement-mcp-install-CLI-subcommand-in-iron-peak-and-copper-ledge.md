---
id: TASK-35.2
title: 'Implement: mcp install CLI subcommand in iron-peak and copper-ledge'
status: Done
assignee: []
created_date: '2026-05-18 17:57'
updated_date: '2026-05-18 19:33'
labels: []
milestone: m-6
dependencies:
  - TASK-35.1
parent_task_id: TASK-35
priority: high
ordinal: 2000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Wire `McpInstaller` into a CLI subcommand on each binary.

## Changes per binary (iron-peak, copper-ledge)

Add to the `Commands` enum:

```rust
/// Manage MCP server installations.
Mcp {
    #[command(subcommand)]
    cmd: McpCommand,
},
```

And a shared `McpCommand` enum (defined once in a shared location, e.g. `hick-agent-sdk` or inlined per binary if trivial):

```rust
enum McpCommand {
    /// Install an MCP server and add it to .hick/mcp-servers.toml
    Install(McpInstallArgs),
    /// List configured MCP servers
    List,
    /// Remove a configured MCP server
    Remove { name: String },
}

struct McpInstallArgs {
    /// Command (stdio) or URL (HTTP). Examples:
    ///   "npx -y @modelcontextprotocol/server-github"
    ///   https://api.example.com/mcp
    spec: String,
    /// Server name (inferred from spec if omitted)
    #[arg(long)]
    name: Option<String>,
    /// Trust level: trusted, untrusted, prompt (default: prompt)
    #[arg(long, default_value = "prompt")]
    trust: McpTrustArg,
    /// Project directory (default: current dir)
    #[arg(long)]
    project_dir: Option<PathBuf>,
}
```

## Install handler (blocking/async main, not TUI)

1. Parse spec → `McpInstallSpec`
2. Resolve name: use `--name` if given, else `infer_name()`; if still `None`, prompt via `print!` / `read_line`
3. Run `McpInstaller::run_package_install` (creates a `WasmSandboxExecutor` with the package-registry network policy; streams stdout/stderr to terminal)
4. Run `McpInstaller::verify_connection` — print tool list on success, exit 1 on failure
5. Prompt for credentials: `print!("Credential env var name (blank to skip): ")` loop until blank
6. Run `McpInstaller::write_config` + `write_credentials`
7. Print success summary

## List handler

Read and parse `.hick/mcp-servers.toml`, print a table: name | transport | trust | command/url.

## Remove handler

Remove the matching `[[servers]]` block from `.hick/mcp-servers.toml`. Error if name not found.

## Notes

- `hick-mcp` (the crate) is the MCP server the agent itself exposes — do not add `mcp install` there.
- `iron-peak` already has pipeline subcommands; `Mcp` sits alongside them.
- `copper-ledge` gets identical CLI shape for consistency.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 `iron-peak mcp install "npx -y @modelcontextprotocol/server-github"` runs end-to-end (install → verify → prompt → write)
- [ ] #2 `iron-peak mcp list` prints current config
- [ ] #3 `iron-peak mcp remove <name>` removes the entry
- [ ] #4 `copper-ledge mcp install/list/remove` are identical in shape and behaviour
- [ ] #5 Exit code is non-zero on install failure (server didn't respond, name conflict, etc.)
- [ ] #6 No TUI is launched when `mcp` subcommand is used
<!-- AC:END -->
