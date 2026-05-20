---
id: TASK-43
title: Consolidate to single `hick` binary — retire iron-peak and copper-ledge
status: To Do
assignee: []
created_date: '2026-05-20 12:16'
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
- [ ] #1 Running `hick` with no arguments starts the TUI and web server
- [ ] #2 `iron-peak` and `copper-ledge` crate directories are deleted
- [ ] #3 A `configs/hick-edit.toml` reproduces iron-peak's extension combination
- [ ] #4 A `configs/standard.toml` reproduces copper-ledge's extension combination
- [ ] #5 Config resolution order: --config flag > .hick/agent.toml > ~/.hick/agent.toml > built-in default
- [ ] #6 All existing iron-peak integration tests pass against the `hick` binary
- [ ] #7 Workspace Cargo.toml no longer references iron-peak or copper-ledge
<!-- AC:END -->
