---
id: DRAFT-1
title: >-
  Remove: per-server NetworkFilter is now handled inside the smoltcp interceptor
  (TASK-34.3)
status: Draft
assignee: []
created_date: '2026-05-18 13:02'
updated_date: '2026-05-18 13:46'
labels: []
milestone: m-5
dependencies:
  - TASK-34.2
parent_task_id: TASK-34
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Status: superseded

This task was originally scoped as a separate `NetworkFilter` wiring step for each sandboxed MCP server container. That approach assumed a per-server `WasmSandboxExecutor` (rejected in the TASK-34.2 revision for resource overhead reasons).

The per-server network filtering is now handled directly inside the smoltcp interceptor implemented in TASK-34.3. The interceptor checks the destination host against the per-server `allowed_hosts` config before forwarding any outbound request. No separate `NetworkFilter` instance or wiring step is needed.

## What was merged into TASK-34.3

- Per-server allowed-hosts enforcement at the interceptor
- Block/allow verdict before forwarding each outbound request
- Config consumed from the `[servers.sandbox]` block (TASK-34.1)

This task can be closed once TASK-34.3 is complete.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 No separate implementation work required — functionality merged into TASK-34.3
- [ ] #2 Verify TASK-34.3 acceptance criteria cover all network filtering requirements originally listed here
<!-- AC:END -->
