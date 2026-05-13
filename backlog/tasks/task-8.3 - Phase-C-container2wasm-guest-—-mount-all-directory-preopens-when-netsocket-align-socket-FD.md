---
id: TASK-8.3
title: >-
  Phase C: container2wasm guest — mount all directory preopens when
  --net=socket; align socket FD
status: To Do
assignee: []
created_date: '2026-05-13 19:57'
updated_date: '2026-05-13 19:58'
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
