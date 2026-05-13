---
id: TASK-8
title: 'WASM sandbox: coexist guest networking with host /workspace mount'
status: To Do
assignee: []
created_date: '2026-05-13 19:57'
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
- [ ] #1 Default hick-agent run lists real project files under /workspace without disabling guest networking.
- [ ] #2 Alpine/Ubuntu-style package installs inside the sandbox (apk/pip paths) still work via virtio socket bootstrap.
- [ ] #3 Automated integration test covers mount plus net-on together.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Phase A: Instrument / document WASI fd assignment for dirs vs socket.

Phase B: Fix Hick container WASI builder ordering / explicit fd for listener.

Phase C: Align container2wasm guest init/kernel with new fd contract; rebuild images.

Phase D: Tests + rollout (optional compatibility env for legacy images).
<!-- SECTION:PLAN:END -->
