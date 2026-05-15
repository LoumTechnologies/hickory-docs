---
id: TASK-16
title: 'Create hick-basic: baseline agent binary (sdk + script-first, no hick XML)'
status: Done
assignee: []
created_date: '2026-05-15 12:44'
updated_date: '2026-05-15 17:58'
labels: []
milestone: m-0
dependencies:
  - TASK-12
  - TASK-13
  - TASK-15
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Create a second agent binary `hick-basic` that uses only `hick-agent-sdk` + `hick-agent-script-first` with no dependency on `hick`, `hick-lang`, or `hick-agent-hick-edit`. This binary is the **evaluation baseline** — a capable AI coding agent with zero hick-pipeline awareness.

`hick-basic` answers the question: "How much of the value comes from the generic script-first ReAct loop, before any hick-specific features?"

**What hick-basic does:**
- Launches the same ratatui TUI as `hick`
- Uses the same WASM sandbox, network firewall, LLM clients
- Uses the same script-first ReAct loop (Python scripts in the sandbox)
- Does NOT watch `_hick.yml`, does NOT produce hick:copy edits, does NOT know paste slots exist
- Works on any repo (not just hick repos)

**What hick-basic is NOT:**
- It is not a stripped-down toy — it should be a genuinely useful coding agent
- It is not permanent — once we have enough evaluation data to decide whether hick-native features add value, this binary may be retired

**Crate location:** `hick-agent/crates/hick-basic/` with `[[bin]] name = "hick-basic"`

**Cargo.toml must NOT depend on:**
- `hick` (the pipeline runner)
- `hick-lang`
- `hick-agent-hick-edit`
- Any other hickory pipeline crate

This is the compile-time guarantee that the baseline agent is truly hick-agnostic.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 hick-basic binary exists and compiles
- [x] #2 Cargo.toml has no dependency on hick, hick-lang, or hick-agent-hick-edit
- [x] #3 hick-basic launches the agent TUI identically to hick (same UX)
- [ ] #4 hick-basic can complete a real coding task on a non-hick repo
- [x] #5 cargo test --workspace passes
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Created `hick-agent/crates/hick-basic/` — a baseline agent binary (`[[bin]] name = "hick-basic"`) that depends only on `hick-agent-sdk`, `hick-agent-script-first`, `hick-agent-core`, and standard CLI crates. No direct `hick`, `hick-lang`, or `hick-agent-hick-edit` in the manifest.

Key differences from the full `hick` binary:
- Single subcommand (agent only — no Run/Up/Repl/Init/Validate pipeline commands)
- No auth injection (`auth: None` in `WasmSandboxConfig`)
- No `PipelineWatcher` startup (no `.hick` file watching)
- No `hick::executor::ensure_loadable_wasm_file` validation (replaced with a soft warning)
- Header displays "hick-basic" instead of "hick-agent"
- Session logs written to `.hick/basic-YYYYMMDD-HHMMSS.hick`

AC #4 (complete a real coding task) requires a running WASM image and is a runtime property not verifiable in CI.
<!-- SECTION:FINAL_SUMMARY:END -->
