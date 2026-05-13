---
id: TASK-8.1
title: 'Phase A: Verify WASI fd collision (preopened dirs vs listener socket)'
status: Done
assignee:
  - Nate Loum
created_date: '2026-05-13 19:57'
updated_date: '2026-05-13 20:34'
labels: []
dependencies: []
parent_task_id: TASK-8
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Prove whether forcing preopened_socket(3, ...) after preopened_dir clobbers the first directory capability. Work in hick-container Container::start and/or a reduced wasmtime example. Output: short design note or issue comment with fd assignment evidence and recommended fix.
<!-- SECTION:DESCRIPTION:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1) Trace WasiCtxBuilder in wasi-common for fd allocation. 2) Add temporary debug or unit-level harness. 3) Confirm hypothesis that /workspace maps to fd 3 then socket steals fd 3.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Confirmed via wasi-common-41.0.4 source: `Table::push()` auto-allocates from `next_key=3` and skips occupied keys. `preopened_socket(3, ...)` calls `insert_at(3)` which overwrites whatever is at FD 3. Current code registered dirs first (auto-allocated FD 3), then the socket overwrote FD 3 — directory lost. Fix: register socket before dirs.
<!-- SECTION:FINAL_SUMMARY:END -->
