---
id: TASK-37.2
title: Shared hick-agent-tui crate — replace per-binary tui.rs files
status: To Do
assignee: []
created_date: '2026-05-19 11:54'
labels:
  - tui
  - ui
dependencies:
  - TASK-37.1
parent_task_id: TASK-37
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
copper-ledge and iron-peak each contain a nearly identical ~950-line tui.rs. This task extracts the TUI into a new shared crate `hick-agent-tui` that consumes `UiEvent`/`UiCommand` from `hick-agent-sdk` (task-37.1). Binary crates then drop their own tui.rs entirely.

## What to build

### New crate: `hick-agent/crates/hick-agent-tui`

A ratatui-based terminal UI driven entirely by `UiEvent`/`UiCommand`. The binary crate calls one function:

```rust
pub fn run_tui(
    event_rx: mpsc::UnboundedReceiver<UiEvent>,
    cmd_tx: mpsc::UnboundedSender<UiCommand>,
    config: TuiConfig,
) -> anyhow::Result<()>
```

where `TuiConfig` carries: `agent_name: String`, `provider_info: String`, `project_dir: PathBuf`.

### Conversation rendering

The TUI maintains an internal `ConversationTree` (from task-37.1). It renders nodes top-to-bottom as a list. Branch structure is shown with ASCII connectors (similar to `git log --graph`):

```
○  User: "do X"
├─ ● Script [python] ▶ (collapsed — press Enter to expand)
│  └─ ○  Output (exit 0) ▶
└─ ○  Done
```

Collapsed nodes display a one-line summary with a `▶` expand indicator. The user navigates nodes with arrow keys and toggles collapse with Enter or Space.

### Collapsible nodes

`ActionStart` (script source) and `ActionEnd` (output), `McpCall`, `McpResponse` are collapsed by default. All others are expanded. The TUI tracks collapse state locally (not round-tripped through `UiCommand` for this task).

### Network approval and MCP credential prompts

Preserve the existing approval/credential prompt flows from the current tui.rs. These are driven by the `UiEvent::NetworkApprovalRequest` variant and the existing `McpCredentialRequest` channel. Map them 1:1 from the current implementation.

### All existing keybindings preserved

Ctrl+C (interrupt / quit), Ctrl+D (quit), Up/Down/PageUp/PageDown (scroll), Enter (submit), arrow keys (cursor), `/mcp` slash commands. No regressions.

## Migration

- Remove `copper-ledge/src/tui.rs` and `iron-peak/src/tui.rs`
- Update both binaries' `main.rs` to call `hick_agent_tui::run_tui(...)` after bridging their `AgentEvent` stream to `UiEvent`
- Update `Cargo.toml` for both binary crates to depend on `hick-agent-tui` instead of the inline module

## Constraints

- `hick-agent-tui` depends on `hick-agent-sdk` but NOT on any extension crate (script-first, engine, etc.)
- ratatui and crossterm stay out of `hick-agent-sdk`
- The TUI feature set must be a strict superset of the current per-binary TUIs (no regressions)
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 New crate hick-agent-tui exists with a single public run_tui() entry point
- [ ] #2 copper-ledge/src/tui.rs and iron-peak/src/tui.rs are deleted
- [ ] #3 Both binaries compile and their TUI behavior is unchanged for all existing keybindings
- [ ] #4 Script source and output nodes are collapsed by default; user can expand with Enter/Space
- [ ] #5 Branch structure (sub-agent nodes) renders with ASCII git-log-style connectors
- [ ] #6 cargo test --workspace passes
<!-- AC:END -->
