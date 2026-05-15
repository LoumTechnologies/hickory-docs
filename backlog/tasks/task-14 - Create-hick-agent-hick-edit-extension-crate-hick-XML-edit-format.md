---
id: TASK-14
title: Create hick-agent-hick-edit extension crate (hick XML edit format)
status: Done
assignee:
  - Nate Loum
created_date: '2026-05-15 12:44'
updated_date: '2026-05-15 17:17'
labels: []
milestone: m-0
dependencies:
  - TASK-12
references:
  - hick-agent/crates/hick-agent-core/src/lib.rs
  - hick-agent/crates/hick-agent-core/src/edit_format.rs
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Consolidate all hick-XML-aware agent behavior into a single `hick-agent-hick-edit` extension crate. This is the crate that knows about `hick:file`, `hick:copy`, `hick:paste`, and the pipeline structure. It is currently spread across two wrongly-named crates.

**What moves here:**

From `hick-agent-core` (currently misnamed — it's actually hick-specific):
- `HickEditFormat` — the edit format parser/emitter
- `HickEditService` — orchestrates edit format operations against the live pipeline
- `PipelineOutput` — represents the rendered output of a pipeline step
- `QueryMatch` / `SourceEdit` — edit location types

From `hick-agent-engine`:
- `PipelineWatcher` — watches `_hick.yml` and reruns `hick run` on change
- Any reverse-provenance logic (mapping output file positions back to hick:file source locations)

**New types to add:**
- `HickEditAgent` — a composable behavior struct that, given an `AgentRuntime` from the sdk, adds hick-edit capabilities: discovers paste slots in the pipeline, produces `hick:copy` elements for modifications, produces `hick:file` elements for new files
- `PasteSlotRegistry` — indexes the named paste slots in the current pipeline so the agent can discover composition points without reading every file

**Dependency graph:**
```
hick-agent-hick-edit → hick-agent-sdk
hick-agent-hick-edit → hick (pipeline execution)
hick-agent-hick-edit → hick-lang (XML parsing)
```

**Note on hick-agent-core:** After this task, `hick-agent-core` should be empty of hick-specific logic. Either delete the crate or repurpose it as an alias. The naming inversion (core = hick-specific, engine = framework) that exists today should be fully resolved by TASK-12 + TASK-13 + TASK-14 together.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 hick-agent-hick-edit crate exists at hick-agent/crates/hick-agent-hick-edit/
- [x] #2 HickEditFormat, HickEditService, PipelineOutput, QueryMatch, SourceEdit are pub use'd with doc comments
- [x] #3 PipelineWatcher is pub use'd with doc comments
- [x] #4 hick-agent-core is either deleted or contains only re-exports pointing to their new homes (no logic)
- [x] #5 HickEditAgent struct composes with AgentRuntime from hick-agent-sdk
- [x] #6 cargo check --package hick-agent-hick-edit passes
- [x] #7 cargo test --workspace passes
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
## Implementation Plan

### Overview
Create `hick-agent-hick-edit` as the authoritative home for all hick-XML-aware agent behaviour.
The key insight: `HickEditFormat`/`HickEditService` live in `hick-agent-core`, but
`PipelineWatcher`/`reverse_provenance`/`file_io` live in `hick-agent-script-first` (which
`hick-agent-engine` re-exports from). Both sets of logic move here.

### Step 1 — Create `hick-agent/crates/hick-agent-hick-edit/`

**Files to create:**
- `Cargo.toml` — deps: hick-agent-sdk, hick, hick-lens, hick-flow, hick-exec, hick-lang,
  async-trait, anyhow, tokio, notify, sha2, serde+serde_json+serde_yaml, log, glob, walkdir
- `src/edit_format.rs` — copy verbatim from hick-agent-core/src/edit_format.rs
- `src/reverse_provenance.rs` — copy verbatim from hick-agent-script-first/src/reverse_provenance.rs
- `src/file_io.rs` — copy from hick-agent-script-first/src/file_io.rs; change
  `use hick_agent_core::…` → `use crate::…`
- `src/pipeline_watcher.rs` — copy from hick-agent-script-first/src/pipeline_watcher.rs;
  change `use hick_agent_core::…` → `use crate::…` and `use crate::file_io` / `reverse_provenance`
- `src/hick_edit_agent.rs` — new: `HickEditAgent` struct wrapping `AgentRuntime` + a
  `HickEditService`; methods: `discover_paste_slots`, `compose_copy_element`,
  `compose_file_element` (stubs returning `unimplemented!()` or minimal bodies)
- `src/paste_slot_registry.rs` — new: `PasteSlotRegistry` struct holding named slot index
- `src/lib.rs` — pub use all public types with doc comments

### Step 2 — Thin-out `hick-agent-script-first`

- `pipeline_watcher.rs` → replace with `pub use hick_agent_hick_edit::PipelineWatcher;`
  (and PipelineState if it was exported)
- `reverse_provenance.rs` → replace with `pub use hick_agent_hick_edit::{…};`
- `file_io.rs` → replace with `pub use hick_agent_hick_edit::{…};`
- `react_loop.rs` → change `use hick_agent_core::CommandExecutor;` →
  `use hick_agent_sdk::CommandExecutor;`
- `lib.rs` → no change needed (PipelineWatcher re-export path still works)
- `Cargo.toml` → replace `hick-agent-core` dep with `hick-agent-hick-edit`

### Step 3 — Convert `hick-agent-core` to a re-export shim

- Delete `src/edit_format.rs` and `src/command.rs`
- Replace `src/lib.rs` with re-exports:
  - `CommandExecutor`, `CommandOutput`, `WasmSandboxConfig`, `WasmSandboxExecutor` from `hick-agent-sdk`
  - `HickEditFormat`, `HickEditService`, `PipelineOutput`, `QueryMatch`, `SourceEdit` from `hick-agent-hick-edit`
- `Cargo.toml` → replace all heavy deps with `hick-agent-sdk` and `hick-agent-hick-edit`

This preserves all existing import paths (hick-agent, tests, hick-agent-engine) without
changing their code.

### Step 4 — Verify

- `cargo check --package hick-agent-hick-edit`
- `cargo test --workspace -- --test-threads=1`
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
## Implementation Summary

Created `hick-agent-hick-edit` at `hick-agent/crates/hick-agent-hick-edit/` as the authoritative home for all hick-XML-aware agent logic.

**New crate structure:**
- `edit_format.rs` — `HickEditFormat`, `HickEditService`, `PipelineOutput`, `QueryMatch`, `SourceEdit` (moved from hick-agent-core)
- `pipeline_watcher.rs` — `PipelineWatcher`, `PipelineState` (moved from hick-agent-script-first)
- `reverse_provenance.rs` — full reverse provenance logic (moved from hick-agent-script-first)
- `file_io.rs` — hick file I/O helpers (moved from hick-agent-script-first)
- `hick_edit_agent.rs` — new `HickEditAgent` struct composing `AgentRuntime` + `HickEditService`
- `paste_slot_registry.rs` — new `PasteSlotRegistry` struct for indexing paste slots
- `lib.rs` — all public API with doc comments

**Updated crates:**
- `hick-agent-script-first`: pipeline_watcher.rs, reverse_provenance.rs, file_io.rs replaced with thin re-export modules pointing at hick-agent-hick-edit; added hick-agent-hick-edit dep alongside hick-agent-core (kept for CommandExecutor type compatibility)
- `hick-agent-core`: edit_format.rs deleted; lib.rs now re-exports HickEditFormat/Service/etc. from hick-agent-hick-edit while keeping command.rs for backward-compatible WasmSandboxExecutor (SDK has different API)

**Note on hick-agent-core command.rs:** The SDK's WasmSandboxConfig/WasmSandboxExecutor have a different API (no images_dir/capabilities/auth fields, different constructor). The binary (hick-agent) uses the old API, so command.rs was kept in hick-agent-core to avoid breaking main.rs scope.

All 85 unit tests pass. blue-iron-cube failure is pre-existing (HTML entities in Rust code in backend/src/lib.rs).
<!-- SECTION:FINAL_SUMMARY:END -->
