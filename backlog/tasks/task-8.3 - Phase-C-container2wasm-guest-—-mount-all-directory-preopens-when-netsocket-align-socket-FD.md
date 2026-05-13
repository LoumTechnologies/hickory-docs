---
id: TASK-8.3
title: >-
  Phase C: container2wasm guest — mount all directory preopens when
  --net=socket; align socket FD
status: Done
assignee: []
created_date: '2026-05-13 19:57'
updated_date: '2026-05-13 20:34'
labels: []
dependencies:
  - TASK-8.2
parent_task_id: TASK-8
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Update container2wasm kernel/init (source of truth outside this repo snippet: see native2wasm/c2w image pipeline) so startup always applies virtio-9p (or equivalent) for every WASI directory preopen before or independently of networking bootstrap. Consume listen socket from the fd Phase B publishes (may differ from hardcoded fd 3). Rebuild oci/rootfs or wasm payload as required.
<!-- SECTION:DESCRIPTION:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Locate init enumeration of WASI caps; fix ordering; add regression in native2wasm or vendor kernel tests if present.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
No guest init changes required. The container2wasm init already enumerates dirs from FD `3 + LISTEN_FDS`. With socket at FD 3 (LISTEN_FDS=1) and dirs at FD 4+, the init finds /workspace correctly. Updated comments in rootfs.rs (workspace mount point already created unconditionally), and the doc comments on guest_network_enabled fields in executor.rs and command.rs to reflect the fix.
<!-- SECTION:FINAL_SUMMARY:END -->
