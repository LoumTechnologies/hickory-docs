---
id: TASK-8.2
title: >-
  Phase B: Hick-container — non-colliding listen socket FD after directory
  preopens
status: To Do
assignee: []
created_date: '2026-05-13 19:57'
labels: []
dependencies:
  - TASK-8.1
parent_task_id: TASK-8
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Change hick-container/crates/container/src/container.rs so the virtio listener socket uses a WASI fd that does not overlap directory preopens (e.g. first socket fd after last preopened_dir). Update LISTEN_FDS-related env/argv only if guest contract requires it; prefer passing explicit fd via env agreed with kernel team.
<!-- SECTION:DESCRIPTION:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Implement fd offset based on preopened_dirs.len(); verify with Phase A harness; ensure stdin/stdout/stderr unchanged.
<!-- SECTION:PLAN:END -->
