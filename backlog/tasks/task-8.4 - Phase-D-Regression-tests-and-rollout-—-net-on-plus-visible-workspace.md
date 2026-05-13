---
id: TASK-8.4
title: 'Phase D: Regression tests and rollout — net-on plus visible /workspace'
status: To Do
assignee: []
created_date: '2026-05-13 19:58'
updated_date: '2026-05-13 19:58'
labels: []
dependencies:
  - TASK-8.3
parent_task_id: TASK-8
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extend hick-agent integration tests (sandbox_workspace_mount or sibling) to run with guest_network_enabled true and assert host seed files appear under /workspace. Keep existing net-off case if useful. Document removal or demotion of HICK_AGENT_DISABLE_SANDBOX_GUEST_NET once default path works. Coordinate rebuilt wasm images in native2wasm converted_images.
<!-- SECTION:DESCRIPTION:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Add test helper create_executor_with_guest_network(workdir,true) passing mount assertions; CI wiring for wasm fixtures; changelog note for image version bump.
<!-- SECTION:PLAN:END -->
