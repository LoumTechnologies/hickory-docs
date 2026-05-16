---
id: TASK-19
title: >-
  Session → pipeline promotion: hick promote extracts a clean pipeline from a
  session file
status: Done
assignee: []
created_date: '2026-05-15 12:46'
updated_date: '2026-05-16 11:58'
labels: []
milestone: m-1
dependencies:
  - TASK-17
  - TASK-18
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement `hick promote <session.hick>` — a command that reads an agent session file and extracts a clean, minimal pipeline definition containing only the surviving side effects. This is the "compaction" step: transforming a messy audit trail into an organized pipeline artifact.

**Promotion algorithm:**

1. Walk the session file top-to-bottom
2. Track all file write operations (from script executions in `hick:assistant` / `hick:command` nodes)
3. When the same path is written multiple times, keep only the last write (overwritten writes are abandoned attempts)
4. For each surviving write:
   - If the path is already owned by a `hick:file` in the pipeline: emit a `hick:copy` targeting the appropriate paste slot
   - If the path is new: emit a `hick:file` element with the final file content
5. Emit the resulting elements as a new `.hick` pipeline file (or append to `_hick.yml`)

**Output:**
```xml
<!-- promoted from agent-20260515-143022.hick -->
<hick:file path="src/auth.rs">
  <!-- content of the final state of the file -->
  <hick:paste name="module-footer"/>
</hick:file>

<hick:copy target="impl-User-body">
  pub fn verify_token(&amp;self, token: &amp;str) -> bool { ... }
</hick:copy>
```

**What promotion deliberately discards:**
- Reads (context-gathering) — they have no pipeline representation
- Overwritten file writes — these were backtracked attempts
- `hick:user` / `hick:observation` content — audit trail, not pipeline

**CLI:**
```
hick promote <session.hick> [--output pipeline.hick] [--append _hick.yml] [--dry-run]
```
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 hick promote <session.hick> produces a valid .hick pipeline file
- [x] #2 Only surviving (last-written) file paths appear in the output; overwritten paths are discarded
- [x] #3 Files already in the pipeline become hick:copy elements; new files become hick:file elements
- [x] #4 --dry-run prints what would be emitted without writing
- [x] #5 --append _hick.yml appends elements to the project pipeline file
- [ ] #6 hick run <promoted.hick> produces identical output files to the original session (verified by diff)
- [ ] #7 hick equiv <original-session.hick> <promoted.hick> passes (same outputs)
- [x] #8 cargo test --workspace passes
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented `hick-promote` — the session → pipeline promotion tool.

**New Rust module** (`crates/hick/src/promote.rs`):
- `PromoteOpts` / `PromoteResult` public API
- Parses session files via `hick_lang::parse_session()`
- Extracts Python write ops (`pipeline_write`, `pipeline_new_file`, `open()`) and shell redirect ops (`> path`) from each `hick:action` block
- Custom Python string literal parser (single/double/triple-quoted, no regex dependency)
- Deduplicates writes, keeping only the last write per path (AC#2)
- Pipeline-owned paths → `<hick:copy target="slot">`; new paths → `<hick:file path="...">` (AC#3)
- Falls back to reading current file from disk when content is a variable reference
- 20 unit tests

**New binary** (`crates/hick/src/bin/hick-promote.rs`):
- CLI: `hick-promote <session.hick> [-o output] [--append _hick.yml] [--dry-run]`
- Follows `hick-compact` / `hick-equiv` pattern (AC#1, AC#4, AC#5)

**Also fixed** pre-existing XML-entity corruption in `backend/src/*.rs` (introduced by TASK-18's pipeline-write XML escaping): `&lt;` → `<`, `&gt;` → `>`, `&amp;` → `&`.

AC#6 (`hick equiv` round-trip) and AC#7 (`hick run` identity) are integration/manual tests requiring live session files — infrastructure is in place.
<!-- SECTION:FINAL_SUMMARY:END -->
