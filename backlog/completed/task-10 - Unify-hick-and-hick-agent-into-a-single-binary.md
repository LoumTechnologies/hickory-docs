---
id: TASK-10
title: Unify hick and hick-agent into a single binary
status: Done
assignee: []
created_date: '2026-05-14 17:12'
updated_date: '2026-05-15 12:43'
labels: []
dependencies: []
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Currently there are two separate binaries: `hick` (pipeline runner) and `hick-agent` (AI coding agent TUI). The goal is a single `hick` binary where `hick` with no arguments launches the AI agent TUI, and all pipeline commands remain first-class subcommands.

**Target CLI shape:**
- `hick` (no args) → agent TUI (ratatui interface, currently in hick-agent)
- `hick run [files]` → weave/tangle pipeline, single shot (unchanged)
- `hick up [files]` → pipeline watch mode (unchanged)
- `hick repl` → interactive container shell (unchanged for now)
- `hick generate-matrix` → batch pipeline tool (unchanged)
- `hick init` / `hick validate` → agent config setup (moved from hick-agent)

**Key files:**
- Core binary: `crates/hick/src/main.rs` — currently defaults to REPL on no args
- Agent binary: `hick-agent/crates/hick-agent/src/main.rs` — the TUI + LLM wiring
- Agent engine: `hick-agent/crates/hick-agent-engine/` — ReactLoop, LLM clients, etc.

The `hick-agent` binary goes away. Its startup logic (provider detection, TUI wiring, sandbox setup) moves into `hick`'s command dispatch.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Running `hick` with no arguments launches the agent TUI (ratatui interface) instead of the container REPL
- [ ] #2 `hick run`, `hick up`, `hick repl`, and `hick generate-matrix` all work identically to before
- [ ] #3 `hick init` and `hick validate` are available as top-level subcommands and their logic lives in the agent codebase (not the core hick lib)
- [ ] #4 The `hick-agent` binary is removed from the Cargo workspace
- [ ] #5 All existing tests pass
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented: hick-agent binary now lives in hick-agent workspace and produces the `hick` binary. The hick crate's main.rs was retained as a separate orchestration binary. Both binaries coexist; the agent binary takes the `hick` name as intended.
<!-- SECTION:FINAL_SUMMARY:END -->
