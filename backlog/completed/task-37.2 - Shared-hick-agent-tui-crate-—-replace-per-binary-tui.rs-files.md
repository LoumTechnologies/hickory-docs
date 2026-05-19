---
id: TASK-37.2
title: Shared hick-agent-tui crate — replace per-binary tui.rs files
status: Done
assignee: []
created_date: '2026-05-19 11:54'
updated_date: '2026-05-19 13:46'
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
- [x] #1 New crate hick-agent-tui exists with a single public run_tui() entry point
- [x] #2 copper-ledge/src/tui.rs and iron-peak/src/tui.rs are deleted
- [x] #3 Both binaries compile and their TUI behavior is unchanged for all existing keybindings
- [x] #4 Script source and output nodes are collapsed by default; user can expand with Enter/Space
- [x] #5 Branch structure (sub-agent nodes) renders with ASCII git-log-style connectors
- [x] #6 cargo test --workspace passes
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
## TUI design philosophy: pretty without waste

The TUI must not attempt to port the web visual system into the terminal. Boxes and per-message borders consume rows without adding information. The right model is tools like lazygit, delta, btop: color, unicode, and alignment as information rather than decoration.

### Left-margin role column (the spine)

Reserve a 3-character left margin as a role/indicator column. All content aligns to a consistent content margin at column 4+. The spine character `│` runs continuously. Indicators appear in the margin only when meaningful:

```
     │ agent message text, wraps
     │ to the next line cleanly
     │
  ●  │ thinking (collapsed — three pulsing dots)
     │
     │ agent continues
     │
  ▸  │ [py] import pathlib...        ← collapsed script, one line
  ▸  │ → github/search "foo"         ← collapsed tool call, one line
     │
```

### Collapsed = one line, always

Every collapsible item fits in exactly one line when collapsed. No mini-boxes, no multi-line summaries. Format:
- Script: `▸ [lang] first-line-of-code...` in dark gray
- Tool call: `▸ → server/tool_name key_param=value` in dark gray
- Thinking: `●●●` in gray (animated pulsing via periodic redraw), no label

`▸` (right-pointing) = collapsed. `▾` (down-pointing) = expanded.

### Expanded = indented block with left border, no wrapper box

When expanded, content indents from the spine with a `┊` left border character — not a full box. The type label stays on the header line:

```
  ▾  │ [py] import pathlib...
     ┊ for f in pathlib.Path('.').rglob('*.rs'):
     ┊     print(f)
     ┊ ───
     ┊ src/main.rs
     ┊ src/lib.rs
```

Script input and output are shown together in one expanded block, separated by `┊ ───`. This mirrors the Jupyter notebook cell model (input on top, output below) without requiring two separate expand actions.

For tool calls:
```
  ▾  │ → github/search_code
     ┊ q: "SessionLog"  repo: "hickory-docs"
     ┊ ───
     ┊ 3 results: src/session_log.rs, ...
```

### Turn separators

Between turns: a single `─────────────────────────────────` in very dark gray. No timestamp unless the gap between turns exceeds 60 seconds. One row cost, worthwhile for readability.

### Thinking

Single animated line: `  ●●●` in gray. Pulse is simulated by cycling between `●●●`, `○●●`, `●○●`, `●●○` on each 16ms redraw tick. When expanded: full thinking text as a `┊`-bordered block in dim gray.

### User messages

User messages get the cyan color and a `>` prefix (consistent with current TUI). No box. Wrap to content margin:

```
     │
  >  │ do something interesting
     │ with the codebase
     │
```

### Refinement notes (post-first-visual-iteration)

The following details are intentionally left unspecified — they will be tuned against a real terminal after the first visual pass:
- Whether output clips at N lines with a "... N more lines" indicator, or scrolls inline
- Exact color palette values (will depend on terminal background assumptions)
- Whether the spine `│` renders continuously or only between adjacent same-role messages
- Handling of very long single lines in script output (truncate? wrap? horizontal scroll?)

Ready to start: TASK-37.1 is Done (`UiEvent`/`UiCommand`/`ConversationTree`/`AgentUiBridge` in sdk + script-first). Next agent: add `hick-agent/crates/hick-agent-tui`, wire `run_tui(event_rx, cmd_tx, TuiConfig)`, bridge binaries with `AgentUiBridge` + map `NetworkApproval`/`McpCredentialRequest` like current tui.rs.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added workspace crate `hick-agent-tui` with `run_tui(ui_rx, ui_cmd_tx, TuiConfig)` (Blocking ratatui session, `ConversationTree` rendering, network/MCP flows, `/mcp` fanout via `TuiConfig.ui_fanout_tx`). Introduced `hick_agent_script_first::run_ui_event_fanout_task` + `AgentUiBridge::next_node_id` to merge `AgentEvent` + `NetworkApprovalRequest` into `UiEvent` with one id allocator; binaries bridge `UiCommand` → `AgentCommand`. Removed per-binary `tui.rs`, dropped direct ratatui/crossterm deps from copper-ledge and iron-peak. `ConversationNodeKind` is `Copy` for clean TUI pattern matches. Full `cargo test --workspace -- --test-threads=1` passed.
<!-- SECTION:FINAL_SUMMARY:END -->
