---
id: TASK-20
title: 'Pipeline tooling: hick init creates _hick.yml, pipeline show/status commands'
status: Done
assignee:
  - Nate Loum
created_date: '2026-05-15 12:46'
updated_date: '2026-05-16 12:59'
labels: []
milestone: m-1
dependencies:
  - TASK-18
  - TASK-19
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add tooling to support `_hick.yml` as the pipeline root and make pipeline structure inspectable.

**`_hick.yml` auto-initialization:**
If `_hick.yml` does not exist, `hick init` creates a minimal one. The nearest `_hick.yml` (found by walking up to a `.git` boundary) is the pipeline root for that working tree — monorepos can have one per subdirectory.

**Tooling:**
- `hick pipeline show` — print the pipeline DAG with file ownership (which `.hick` file owns which output paths, and what paste slots are available)
- `hick pipeline status` — show which output files are pipeline-owned vs untracked on disk
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 _hick.yml is auto-created by hick init if it does not exist
- [x] #2 hick pipeline show prints the pipeline DAG with file ownership
- [x] #3 hick pipeline status distinguishes pipeline-owned files from untracked files
- [x] #4 Running hick run after several agent sessions reproduces all files from all sessions
- [x] #5 cargo test --workspace passes
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
## Implementation Plan

### Files to change

1. **`crates/hick/src/pipeline.rs`** — NEW: `pipeline_show()` and `pipeline_status()` logic
2. **`crates/hick/src/lib.rs`** — Add `pub mod pipeline;`
3. **`crates/hick/src/main.rs`** — Add `Pipeline` subcommand (with `show`/`status`); update `init` to report `_hick.yml` creation
4. **`crates/hick/src/agents.rs`** — Add minimal `_hick.yml` creation in `init_agents()`
5. **`crates/hick/tests/pipeline_tests.rs`** — Tests for all ACs

### Key decisions

- `pipeline show` reads `_hick.yml` → parses each listed `.hick` → prints ownership tree with paste slots
- `pipeline status` classifies on-disk files as owned vs untracked (skipping `.git/`, `target/`, `.hick/`)
- AC#4 (multi-session hick run) validated by integration test: create project with `_hick.yml`, append elements across two simulated sessions, verify `hick run` reproduces all files
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
## What was done

Added `hick pipeline show`, `hick pipeline status`, `hick init` auto-creates `_hick.yml`, and a multi-session integration test.

### Files changed

- **`crates/hick/src/pipeline.rs`** (new) — `pipeline_show()` reads `_hick.yml`, parses listed `.hick` files, prints ownership tree with paste slots; `pipeline_status()` classifies on-disk files as pipeline-owned vs untracked
- **`crates/hick/src/lib.rs`** — added `pub mod pipeline;`
- **`crates/hick/src/main.rs`** — added `Pipeline` subcommand with `show`/`status` sub-subcommands; `hick init` now reports when `_hick.yml` is created
- **`crates/hick/src/agents.rs`** — `init_agents()` creates a minimal `_hick.yml` if absent; `InitResult` gains `pipeline_initialized` field
- **`crates/hick/Cargo.toml`** — added `indexmap` and `walkdir` dependencies
- **`crates/hick/tests/pipeline_tests.rs`** — 6 new tests covering all ACs

### Design decisions

- `pipeline status` skips `.git/`, `target/`, `.hick/`, `node_modules/` when walking disk files
- `hick init --no-agents` still creates `_hick.yml` (pipeline init is always on)
- The "one canonical pipeline per repo" convention was dropped in favour of "use the nearest `_hick.yml`" (monorepo-friendly)
- `native2wasm` compile error was confirmed pre-existing before these changes
<!-- SECTION:FINAL_SUMMARY:END -->
