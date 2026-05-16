---
id: TASK-22
title: >-
  Rename agent binaries to codenames and strip hick-XML from
  hick-agent-script-first
status: Done
assignee: []
created_date: '2026-05-15 18:20'
updated_date: '2026-05-15 18:27'
labels: []
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Two goals in one refactor:

1. **Strip hick-XML from `hick-agent-script-first`** — the crate currently depends on `hick-agent-hick-edit`, `hick-flow`, `hick-lang`, and `hick`, and re-exports PipelineWatcher/reverse_provenance/file_io/backing through thin stubs. All of this gets deleted. The crate becomes generic SDK + script extraction + ReAct loop, zero hick pipeline knowledge.
   - Also make `ReactLoop` configurable: `with_agent_dir(str)`, `with_system_prompt(str)`, `with_sdk()` builder methods
   - Default system prompt becomes generic (no hick XML)
   - Default agent_dir = ".agent"

2. **Rename binary crates to codenames** (defer the "which one gets called hick" decision):
   - `hick-basic` → `copper-ledge` (standard agent: WASM sandbox, no hick XML, no SDK)
   - `hick-agent` (produces `hick` binary) → `iron-peak` (hick-native agent)
   - `iron-peak` injects the hick-aware system prompt via `with_system_prompt()`, uses `with_agent_dir(".hick")`, `with_sdk()`
   - `iron-peak` imports PipelineWatcher from `hick-agent-hick-edit` directly (no longer re-exported from script-first)
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 hick-agent-script-first has no deps on hick, hick-lang, hick-flow, hick-agent-hick-edit
- [x] #2 backing.rs, file_io.rs, pipeline_watcher.rs, reverse_provenance.rs deleted from script-first
- [x] #3 ReactLoop accepts with_agent_dir(), with_system_prompt(), with_sdk() builders
- [x] #4 Default ReactLoop system prompt contains no hick XML mentions
- [x] #5 hick-basic directory renamed to copper-ledge with updated Cargo.toml and binary name
- [x] #6 hick-agent directory renamed to iron-peak with updated Cargo.toml and binary name iron-peak
- [x] #7 iron-peak injects hick-aware system prompt and uses .hick agent_dir
- [x] #8 cargo test --workspace passes
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Stripped all hick-XML dependencies from `hick-agent-script-first` (removed `hick`, `hick-lang`, `hick-flow`, `hick-agent-hick-edit` deps; deleted `backing.rs`, `file_io.rs`, `pipeline_watcher.rs`, `reverse_provenance.rs`). Added `with_agent_dir()`, `with_system_prompt()`, `with_sdk()` builders to `ReactLoop`. Default system prompt is now generic with no hick XML content.

Renamed `hick-basic` → `copper-ledge` (binary: `copper-ledge`, agent_dir: `.copper-ledge`, no SDK setup). Renamed `hick-agent` → `iron-peak` (binary: `iron-peak`, agent_dir: `.hick`, SDK enabled, hick-aware system prompt injected via `with_system_prompt(HICK_SYSTEM_PROMPT)`). `iron-peak` imports `PipelineWatcher` directly from `hick-agent-hick-edit`. Also fixed `hick-agent-engine` which re-exported `PipelineWatcher` from script-first. All agent crate tests pass.
<!-- SECTION:FINAL_SUMMARY:END -->
