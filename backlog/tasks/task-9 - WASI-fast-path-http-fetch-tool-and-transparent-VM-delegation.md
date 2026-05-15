---
id: TASK-9
title: >-
  WASI fast-path: agent-oriented HTTP fetch tool + transparent delegation from
  Linux WASM VM
status: To Do
assignee: []
created_date: '2026-05-13'
updated_date: '2026-05-13'
labels:
  - wasm
  - wasi
  - networking
  - performance
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->

### Problem

Agent workloads routinely invoke simple networking utilities (especially **`curl`-style HTTP GET**) inside the Hick Linux WASM VM. That path boots/emulates a full guest stack and is comparatively expensive even though the actual need is often a bounded HTTP fetch.

Separately, the repo already runs **WASI Preview 1** for the Linux VM container (`wasi-common` + `wasi_snapshot_preview1`). **WASI Preview 2 / components** are enabled at the Wasmtime engine feature level but are not used for that VM path. Network-heavy utilities are strong candidates for a **native WASI component fast path** on the host while preserving Hick’s WASM-level policy model (`hick-agent` / capabilities).

### Porting criteria (product guidance)

Prefer WASI ports for tools that:

1. **Use the network** in the common agent invocation shape (HTTP fetch is the canonical example).
2. Are **disproportionately expensive inside the Linux VM** compared to host Wasmtime (guest virtio path + emulator overhead).
3. Are **invoked frequently by AI coding agents** (documentation fetch, small API calls, release tarball probes, etc.).
4. Have a **narrow syscall/WASI surface** so Preview 2 worlds (`wasi:http` / `wasi:sockets` / DNS resolution as exposed by Wasmtime) are sufficient.

### Proposed first tool: `hick-fetch`

Ship a **minimal Rust CLI** (working name **`hick-fetch`**) compiled as a **WASI 0.2 component**:

- Subset behavior aligned with common agent usage: HTTPS GET, timeouts, response size cap (abort oversized bodies), optional fail-on-HTTP-error exit code, stdout = response body (binary-safe path later).
- Explicit non-goals for v1: full `curl` flags, cookies jars, HTTP/2 client knobs beyond defaults—avoid parity traps.

This becomes the reference implementation for “criteria met → WASI component exists.”

### Transparent delegation from the VM

**Goal:** When a pipeline command line **would** run the slow-path utility inside the VM (e.g. `curl -fsSL …`), optionally **execute the WASI component on the host** instead and **return output as if** the command ran in-guest—without agents rewriting scripts.

**Primary integration point:** `ContainerExecutor::execute` in `crates/hick/src/executor.rs` feeds shell lines to the guest. Add an **opt-in delegation layer before stdin injection**:

- Match **conservative, configured patterns** (initially: single-line commands whose first token normalizes to `curl` with an allowlisted flag subset + URL extractable without shell expansion).
- Run **`hick-fetch`** via Wasmtime Preview 2 with **`wasmtime-wasi`** networking and **the same URL/host policy** already enforced for container networking (`ContainerCapabilities` / existing filter types from `hick-agent-engine`).
- **Transcript / replay:** Record whether execution was delegated so fork/replay semantics stay deterministic; decide whether replay re-invokes WASI or replays captured output (document tradeoff in implementation).

**Secondary / later:** Guest-side PATH shim or stub binaries inside the rootfs are fragile with nested shells and pipelines; treat host-side interception as the supported approach unless product demands otherwise.

### References

- Linux VM WASI host surface: `hick-container/crates/container/src/container.rs` (`wasi_common`, Preview 1).
- Command injection path: `crates/hick/src/executor.rs` (`execute`, `execute_with_stdin`).
- Wasmtime workspace features: root `Cargo.toml` (`component-model`, `component-model-async`).
- Prior coexistence work: `backlog/tasks/task-8*.md`.

<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria

<!-- AC:BEGIN -->
- [ ] **AC1** — New Rust crate under workspace builds `hick-fetch` as a **WASI Preview 2 component** and documents build/run with Wasmtime 41+.
- [ ] **AC2** — `hick-fetch` supports at least: URL positional arg, connect/read timeouts, max response bytes, HTTPS GET, sensible exit codes for DNS/TLS/HTTP failures.
- [ ] **AC3** — Host-side runner executes the component with Wasmtime **Preview 2** WASI networking (`wasmtime-wasi` / documented worlds), without nesting inside the Linux VM.
- [ ] **AC4** — Delegation hook in `ContainerExecutor::execute` (behind **explicit config / feature flag**) can intercept a **narrow class** of `curl` invocations and produce stdout compatible enough for existing pipelines.
- [ ] **AC5** — Delegated calls enforce **policy parity** with container networking rules (same allow/deny inputs as the container path; tests prove deny wins).
- [ ] **AC6** — Command history / replay behavior for delegated executions is **defined, implemented, and covered by tests** (no silent divergence after fork).
- [ ] **AC7** — Short design note in-repo listing **porting criteria** and how to register additional WASI fast-path tools later.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->

### Phase A — Scaffolding and reference component (`hick-fetch`)

1. Add workspace member crate (suggested path: `crates/hick-fetch/` or `hick-exec/crates/hick-fetch/`—follow sibling crate layout conventions).
2. Toolchain: **`cargo-component`** (or project-standard component build) targeting WASI 0.2; depend on **`wasi:http`** outgoing handler world **or** lower-level **`wasi:sockets`** + minimal TLS story—pick smallest stack that matches Wasmtime 41 support (prototype spike if TLS via Wasmtime’s WASI HTTP client is simpler than raw sockets + TLS in guest).
3. Implement CLI parsing (`clap` or minimal `std::env::args` for Wasm footprint).
4. Golden-path integration test: run compiled component via **`wasmtime` CLI or small Rust harness** in `hick-agent`/`container-testbed`-style test crate.

### Phase B — In-process Preview 2 runner inside Hick stack

1. New module (e.g. `crates/hick/src/wasi_fast_path.rs` or `hick-agent-engine`) that owns **`Engine`/`ComponentLinker`** setup distinct from `wasi-common` Preview 1.
2. Wire **`wasmtime-wasi`** Preview 2 + HTTP/outgoing request interfaces per Wasmtime 41 docs; reuse workspace `wasmtime` dependency features as needed.
3. Thread **network capability / URL filter** from existing policy types into the WASI host (deny before guest runs).

### Phase C — Transparent delegation from VM commands

1. Introduce configuration struct: `WasiFastPathConfig { enabled: bool, delegate_curl: bool, … }` plumbed from `hick-agent` → executor construction site(s).
2. In `ContainerExecutor::execute`, **before** writing shell lines to the container:

   - Parse candidate commands (start with **single logical line** + **`curl`** heuristic only).
   - If match and allowed by policy: invoke Phase B runner, synthesize captured transcript consistent with marker stripping (`START_MARKER`/`END_MARKER`/`SYNC_MARKER`) expectations—prefer shared helper to avoid drift from `execute` output normalization.

3. Extend **`execute_with_stdin`** only if POST delegation is in scope; otherwise explicitly defer (large surface).

### Phase D — Tests and rollout

1. Unit tests: URL extraction, matcher edge cases (`curl` not first token → no delegate).
2. Integration tests: denied URL fails delegated path; allowed URL succeeds; replay/history entries behave per AC6.
3. Feature flag default **off** until soak; document operational checklist.

### Follow-on tasks (optional backlog splits)

- **TASK-9.1** — Additional tools (`wget`-shaped, registry HEAD probes) once matcher framework exists.
- **TASK-9.2** — Guest PATH shim only if product requires coverage outside `ContainerExecutor::execute`.

<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->

_(Fill when task completes.)_

<!-- SECTION:FINAL_SUMMARY:END -->
