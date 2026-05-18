---
id: TASK-35.3
title: 'Implement: /mcp slash command and credential prompt overlay in TUI'
status: Done
assignee: []
created_date: '2026-05-18 17:58'
updated_date: '2026-05-18 19:33'
labels: []
milestone: m-6
dependencies:
  - TASK-35.1
parent_task_id: TASK-35
priority: high
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add `/mcp install`, `/mcp list`, and `/mcp remove` slash commands to the TUI in both iron-peak and copper-ledge. Both TUIs share identical structure so the pattern is the same.

## Input handling

In the `Enter` key handler (alongside the existing `!` prefix check):

```rust
if msg.starts_with("/mcp ") {
    let rest = msg[5..].trim();
    handle_mcp_slash(rest, cmd_tx, self);
    return;
}
```

`handle_mcp_slash` dispatches:
- `install <spec>` or `install <spec> --name <n> --trust <t>` → `AgentCommand::McpInstall { spec, name, trust }`
- `list` → `AgentCommand::McpList`
- `remove <name>` → `AgentCommand::McpRemove { name }`
- anything else → push an `Info` line with usage

All slash commands are rejected (push an `Info` line) while `self.busy` is true.

## New AgentCommand variants

```rust
McpInstall { spec: String, name: Option<String>, trust: McpServerTrust },
McpList,
McpRemove { name: String },
```

## Agent event loop handling

The event loop (currently in `run_tui` / the select! loop) handles new commands:

- `McpList` and `McpRemove`: synchronous config reads/writes; push result lines back through the event channel as `AgentEvent::McpResult(lines)`
- `McpInstall`: async, multi-step — needs to drive `McpInstaller` and pause mid-way for credential input

## Credential prompt overlay (the tricky part)

`McpInstall` cannot do blocking stdin reads inside the TUI. Instead it follows the same pattern as `NetworkApprovalRequest`:

1. After `verify_connection` succeeds, the install task sends `AgentEvent::McpCredentialPrompt { server_name: String, suggested_env_var: String }` through the event channel
2. The TUI renders a credential prompt overlay (similar to the network approval overlay) showing:
   - `Server: <name> — enter credential env var name (blank to skip):`
   - A text input field (reuses `self.input` while in credential-prompt mode)
3. On Enter in that mode, the TUI sends `AgentCommand::McpCredentialResponse { env_var: String, value: String }` (value prompted second)
4. The install task receives the response and calls `write_credentials`, then sends `AgentEvent::McpInstallComplete { tools: Vec<String> }`
5. TUI renders the tool list and returns to normal input mode

## State additions to TUI struct

```rust
/// Set when awaiting a credential env-var name from the user.
pending_mcp_credential: Option<PendingMcpCredential>,
```

```rust
struct PendingMcpCredential {
    server_name: String,
    /// Which field we're collecting: EnvVarName or Value(env_var_name)
    stage: CredentialStage,
    response_tx: oneshot::Sender<(String, String)>, // (env_var, value)
}
```

## Notes

- Only one `/mcp install` can run at a time. Reject a second one while `busy`.
- The install task runs in a `tokio::spawn`; it uses the same `WasmSandboxExecutor` pattern as direct commands.
- Stdout from `run_package_install` is streamed as `AgentEvent::Output` lines (same as direct commands) so the user sees progress.
- `iron-peak/src/tui.rs` and `copper-ledge/src/tui.rs` are near-identical; update both.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 `/mcp install "npx -y @modelcontextprotocol/server-github"` inside the TUI runs end-to-end: package install progress shown, credential prompt overlay appears, config written, tool list printed
- [ ] #2 `/mcp list` prints the current server table inside the TUI
- [ ] #3 `/mcp remove <name>` removes the entry and confirms inside the TUI
- [ ] #4 Slash commands are no-ops (show usage) while the agent is busy
- [ ] #5 Credential prompt overlay renders correctly and returns to normal input after completion or skip
- [ ] #6 Both iron-peak and copper-ledge TUIs support all three commands
<!-- AC:END -->
