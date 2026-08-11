# Cloud Canopy integration

`hickory-executor-canopy` is the ONLY code in this repo that knows canopy's
API. Canopy is under active concurrent development (a non-bare-metal mode is
in flight), so this adapter must degrade gracefully and fail with actionable
errors, never panic.

## What canopy provides (as of 2026-08-05)

- Nodes run Firecracker microVMs booted from prebuilt **Nix store images**
  (`/nix/store/...-canopy-sandbox-image`), declared per-tenant in a
  hash-chained ledger with limits (images allowlist, vcpus, memory, lifetime,
  egress hosts).
- Node agent gRPC (`canopy.proto`, vendored into
  `crates/hickory-executor-canopy/proto/` with the source commit recorded):
  `SpawnSandbox`, `DestroySandbox`, `ListSandboxes`, and `AttachSandbox` — a
  bidirectional byte stream bridging the guest channel, with the vsock
  `CONNECT` preamble spoken agent-side. Capability tokens travel as
  `x-canopy-capability-bin` binary metadata (raw biscuit bytes); over the
  node-local unix socket no token is needed.
- Sandbox I/O: the guest hands each channel connection to a shell on a
  **pty**. Protocol: ready banner, then one base64-encoded script line
  bracketed by sentinels (the pty echoes input, so the script must be one
  predictable line). Reference client: cloud-canopy
  `crates/canopy-cli/src/guest.rs`; mirrored in the adapter's `frame`
  module.

## How the adapter maps the Executor trait onto canopy

- `ensure_started` → `SpawnSandbox` (shape from `CANOPY_VCPUS` /
  `CANOPY_MEM_MIB` / `CANOPY_LIFETIME_SECS`), then poll the attach stream
  for the guest ready banner. One microVM per container.
- `execute` → a fresh `AttachSandbox` stream per exec (each connection gets
  a fresh guest shell, matching `LocalExecutor`'s fresh-`sh` semantics).
  Commands run under a fixed `/hickory-work` workdir so relative paths
  behave like the local executor's per-container temp dir. Output is a pty,
  so stdout/stderr arrive merged: transcripts record `cmd`/`out`/`exit`
  events with the same millisecond-epoch timing as `LocalExecutor`, but no
  `err` events.
- **Forks** spawn a new sandbox and replay the source container's recorded
  command history (transcripts hold it). Stdin payloads are not stored in
  transcripts and are not replayed; a replayed command that fails is logged
  and skipped (its failure already surfaced in the source container).
- **Volumes** are shuttled through the guest pty as base64-chunked tar:
  inject appends ~2 KiB base64 lines to a staging file in the guest, then
  `base64 -d | tar -x`; extract runs `tar -c | base64` and decodes the
  captured output. Tradeoff: dependency-free and correct over the one
  channel canopy guarantees, but O(volume size / 2 KiB) attach round-trips —
  fine for the small volumes documents pass today, wrong for bulk data. The
  upgrade path is a dedicated transfer RPC or a second vsock port, both of
  which are canopy-side changes.
- `shutdown` → `DestroySandbox` for every sandbox this executor spawned
  (failures are reported but sandboxes also die at their ledgered lifetime
  deadline).

## Known gaps (tracked, not worked around silently)

1. **Reachability**: the hosted app must reach a node agent — either the
   node-local unix socket (same machine) or the mesh `host:port`, which
   requires the app to be a WireGuard peer and to hold a capability token.
   Endpoint + token are env config (`CANOPY_AGENT`, `CANOPY_TOKEN`).
2. **Image mapping**: `.hick` `image="python:3.12"` attributes map through a
   deployment-config table (`CANOPY_IMAGE_MAP`, JSON) to Nix store paths
   declared in the tenant ledger. Unknown image → clear error listing
   configured images.
3. **No spare byte channel**: volumes ride the pty (see above).

## Env contract

```
HICKORY_EXECUTOR=local|canopy
CANOPY_AGENT=/run/canopy/agent.sock   # or mesh host:port, e.g. 10.77.0.1:7433
CANOPY_TOKEN=…                        # base64 capability token; omit on the local socket
CANOPY_NODE=colo-1                    # informational (error messages)
CANOPY_IMAGE_MAP={"python:3.12":"/nix/store/…","alpine:3.20":"/nix/store/…"}
CANOPY_VCPUS=1                        # per-sandbox shape, defaults shown
CANOPY_MEM_MIB=512
CANOPY_LIFETIME_SECS=900
CANOPY_EGRESS_HOSTS=pypi.org,…        # optional, default none
CANOPY_BOOT_TIMEOUT_SECS=180
CANOPY_EXEC_TIMEOUT_SECS=300
```

`GET /api/health` reports which executor is active and whether canopy is
reachable. With `HICKORY_EXECUTOR=local`, canopy vars are ignored.
`CanopyExecutor::from_env()` (or `::new(CanopyConfig)`) is the constructor
the CLI and server share; connection to the agent is lazy, so selection
succeeds before the agent is reachable and the first use carries the error.

**Windows.** The crate builds and the mesh (`host:port`) path works, because
that is plain TCP. The unix-socket path does not exist there — Windows has no
unix domain sockets, so a `CANOPY_AGENT` beginning with `/` fails on first
use with a message naming the mesh alternative. That is a deliberate `cfg`
gate in `crates/hickory-executor-canopy/src/executor.rs`
(`connect_unix_socket`), not an oversight: the whole `hickory` binary would
otherwise not build for `x86_64-pc-windows-msvc` at all. A local node agent
is unreachable from Windows in any case — it only ever listens on a socket.

## Live smoke status (2026-08-05)

The env-gated live test (`tests/live.rs`, `#[ignore]`, needs
`HICKORY_CANOPY_LIVE=1`) was run against cloud-canopy's dev agent
(`just agent-dev`, unix socket `/tmp/canopy-dev.sock`). How far it got:

1. Connect over the unix socket and `SpawnSandbox` RPC: **works** — the
   agent authenticated the caller as `operator@dev-local` and initially
   refused with "no limits in force" (correct: absent limits deny).
2. After `canopy limits set operator@dev-local --image …`: authz **passed**
   and the request reached the compute backend, which refused because the
   mapped image path is "not a directory on this node. An image directory
   must contain a kernel (`vmlinux`) and a root filesystem (`rootfs.ext4`)".
3. Beyond that a real guest is required: `just sandbox-image` builds the Nix
   image on a remote root NixOS host over ssh, which this dev machine does
   not have — so the attach round-trip (banner → exec → sentinels) could
   not be completed live. It is fully covered by the in-crate mock contract
   test (`tests/contract.rs`), which fakes the pty guest against the real
   vendored proto.

So: transport, token plumbing, spawn, and error surfaces are verified
against a real agent; guest boot + attach remain verified against the mock
only, until a node with a built sandbox image is available. The live test
stays `#[ignore]` and should be pointed at a real node then.

## Cell images (what software a cell has)

Canopy guests are NixOS configurations, so cell toolchains are defined in
this repo's `flake.nix` by extending canopy's `sandbox-guest` module with
package sets: `cell-shell` (coreutils/sed/awk), `cell-python` (python3 +
polars + duckdb + uv), `cell-datasci` (adds R + ggplot2 + the DuckDB CLI).
Build on a nix machine (`nix build .#cell-datasci`, optionally
`--override-input cloud-canopy path:<local checkout>`), declare the store
path in the tenant ledger, and map `image=` refs to it via
`CANOPY_IMAGE_MAP`. The LocalExecutor ignores images; the same document runs
locally against host tools.
