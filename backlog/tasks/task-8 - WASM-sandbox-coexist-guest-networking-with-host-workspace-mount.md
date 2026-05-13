---
id: TASK-8
title: 'WASM sandbox: coexist guest networking with host /workspace mount'
status: Done
assignee:
  - Nate Loum
created_date: '2026-05-13 19:57'
updated_date: '2026-05-13 20:35'
labels: []
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Problem

With guest virtio networking enabled (`--net=socket`, `LISTEN_FDS`), the Hick agent often sees an empty or stub `/workspace` in the WASM VM despite a host workdir being preopened via WASI. Disabling guest networking restores virtio-9p so `/workspace` lists real project files, but breaks typical `apk`/`pip`/guest-side installs.

Current mitigation in-tree: `guest_network_enabled` / `HICK_AGENT_DISABLE_SANDBOX_GUEST_NET` plus dual-path script injection.

## Leading hypothesis

`hick-container/crates/container/src/container.rs` registers directory preopens first, then calls `preopened_socket(3, ...)`. After stdio (0–2), the first directory likely occupies WASI fd 3; forcing the listen socket onto fd 3 may clobber that directory capability.

## Goal

Single default configuration where **both** virtio guest networking **and** host-backed `/workspace` work without relying on env bypass.

## Approach (high level)

1. Prove fd layout with instrumentation or minimal repro against wasmtime/wasi-common.
2. Host: allocate listen socket fd **after** all directory preopens (or explicit non-colliding fd); align env/argv contract with guest.
3. Guest/container2wasm init: mount **all** directory preopens regardless of `--net=socket`; consume socket fd from explicit contract if systemd LISTEN_FDS assumptions break.
4. Rebuild/republish converted `.wasm` images as needed.
5. Add integration test with networking **on** asserting `/workspace` lists host seed files; reduce reliance on `HICK_AGENT_DISABLE_SANDBOX_GUEST_NET`.

## References

- `crates/hick/src/executor.rs` (`guest_network_enabled`)
- `hick-container/crates/container/src/container.rs` (`preopened_dir` / `preopened_socket`)
- `native2wasm/crates/c2w/src/rootfs.rs` (workspace mount point commentary)
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Default hick-agent run lists real project files under /workspace without disabling guest networking.
- [x] #2 Alpine/Ubuntu-style package installs inside the sandbox (apk/pip paths) still work via virtio socket bootstrap.
- [x] #3 Automated integration test covers mount plus net-on together.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Root cause confirmed: `WasiCtxBuilder::preopened_dir` uses `table.push()` which auto-allocates from FD 3. `preopened_socket(3, ...)` calls `table.insert_at(3, ...)` which overwrites FD 3. Current code registers dirs first (they get FD 3), then the socket (overwrites FD 3) — directory capability lost.

Fix: Register socket BEFORE directories. `push()` skips occupied keys, so dirs land on FD 4+ — exactly where the container2wasm init enumerates them after consuming LISTEN_FDS=1 socket at FD 3.

Changes:
1. hick-container/crates/container/src/container.rs — move socket registration before dir loop
2. hick-agent/crates/hick-agent/tests/sandbox_workspace_mount.rs — add net-on integration test + update stale comments
3. hick-agent/crates/hick-agent/src/main.rs — update HICK_AGENT_DISABLE_SANDBOX_GUEST_NET comment (bug fixed)
4. hick-agent/crates/hick-agent/tests/common/mod.rs — update create_executor_with_guest_network comment
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Root cause: `WasiCtxBuilder::preopened_dir` calls `table.push()` (auto-allocates from FD 3), while `preopened_socket(3, ...)` calls `table.insert_at(3)` which silently overwrites FD 3. Old code registered dirs first → socket clobbered the /workspace capability.

Fix: register the socket at FD 3 BEFORE directory preopens in `Container::start`. `push()` skips occupied keys, so dirs land on FD 4+ — exactly where the container2wasm init (reading LISTEN_FDS=1) looks for them. No guest init changes required.

Files changed:
- hick-container/crates/container/src/container.rs — socket registered before dir loop
- hick-agent/crates/hick-agent/tests/sandbox_workspace_mount.rs — net-on regression test added
- hick-agent/crates/hick-agent/tests/common/mod.rs — updated helper doc comment
- hick-agent/crates/hick-agent/src/main.rs — updated HICK_AGENT_DISABLE_SANDBOX_GUEST_NET comment
- crates/hick/src/executor.rs — updated guest_network_enabled field doc
- hick-agent/crates/hick-agent-core/src/command.rs — updated guest_network_enabled field doc
<!-- SECTION:FINAL_SUMMARY:END -->
