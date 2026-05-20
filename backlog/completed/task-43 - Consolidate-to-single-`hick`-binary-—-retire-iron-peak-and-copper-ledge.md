---
id: TASK-43
title: Consolidate to single `hick` binary — retire iron-peak and copper-ledge
status: Done
assignee: []
created_date: '2026-05-20 12:16'
updated_date: '2026-05-20 13:00'
labels:
  - agent
  - architecture
dependencies:
  - TASK-27
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Promote `hick-agent/crates/hick-agent/` to be the one and only agent binary. Delete the `iron-peak` and `copper-ledge` binary crates. Their current extension combinations become named TOML configs shipped in a `configs/` directory (e.g. `configs/hick-edit.toml`, `configs/standard.toml`). Running `hick` with no arguments starts the TUI and hosts the web UI, identical to what iron-peak does today. The active config is determined by the first of: `--config` flag, `.hick/agent.toml` in the project root, `~/.hick/agent.toml`, built-in default.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Running `hick` with no arguments starts the TUI and web server
- [x] #2 `iron-peak` and `copper-ledge` crate directories are deleted
- [x] #3 A `configs/hick-edit.toml` reproduces iron-peak's extension combination
- [x] #4 A `configs/standard.toml` reproduces copper-ledge's extension combination
- [x] #5 Config resolution order: --config flag > .hick/agent.toml > ~/.hick/agent.toml > built-in default
- [x] #6 All existing iron-peak integration tests pass against the `hick` binary
- [x] #7 Workspace Cargo.toml no longer references iron-peak or copper-ledge
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Consolidated to a single `hick` binary by:

- Renamed `hick-agent` binary from `hick-agent` → `hick` and added a `[lib]` target for integration test imports
- Rewrote `hick-agent/src/main.rs` based on iron-peak's TUI/web server main, adding config-driven extension gating via `load_extensions()` (resolution: `--config` > `.hick/agent.toml` > `~/.hick/agent.toml` > built-in hick-edit default)
- Created `hick-agent/src/lib.rs` with the same re-exports as iron-peak's lib.rs so integration tests can use `hick_agent::events`, `hick_agent::llm`, `hick_agent::tools`
- Updated `hick-agent/Cargo.toml`: added `hick-agent-tui`, `hick-agent-engine` (optional), `hick-literate`, `hick-lang`, `hick-secrets`, `hick-token`, `chrono`, `hick-agent-sdk` with `["web", "relay"]` features, `integration-test` feature flag
- Moved all 7 iron-peak integration test files to `hick-agent/tests/`; updated `common/mod.rs` and `react_loop_integration.rs` to use `hick_agent::` imports
- `configs/hick-edit.toml` and `configs/standard.toml` were already present at `hick-agent/configs/`
- Deleted `hick-agent/crates/iron-peak/` and `hick-agent/crates/copper-ledge/`
- Full workspace test suite passes (0 failures)
<!-- SECTION:FINAL_SUMMARY:END -->
