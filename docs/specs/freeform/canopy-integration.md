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
- Customer surface: control plane GraphQL (`POST /graphql`, bearer capability
  token): `spawnSandbox(node, id, image, vcpus, memMib, lifetimeSecs,
  egressHosts)`, `destroySandbox`, plus `fleet` queries.
- Sandbox I/O: one duplex byte channel (vsock → guest pty). Protocol:
  handshake, then base64-encoded script + sentinels (pty echoes). Reference
  client: cloud-canopy `crates/canopy-cli/src/guest.rs`.

## Known gaps (tracked, not worked around silently)

1. **Output over HTTP**: the control plane's `/api/terminal` WebSocket is
   session-cookie-authed only; bearer-token API clients cannot attach today.
   Until canopy grows token auth on that route (or an exec mutation), the
   hosted app needs either (a) mesh access (WireGuard peer + gRPC
   `AttachSandbox`) or (b) that small canopy change. The adapter supports
   both transports behind `CanopyTransport`.
2. **Reachability**: Nate's node control plane binds 127.0.0.1:8088. The
   Railway app reaches it via a portzero tunnel or canopy's nginx/ACME
   `domain` option. Endpoint + token are env config (`CANOPY_URL`,
   `CANOPY_TOKEN`, `CANOPY_NODE`).
3. **Image mapping**: `.hick` `image="python:3.12"` attributes map through a
   deployment-config table (`CANOPY_IMAGE_MAP`, JSON) to Nix store paths
   declared in the tenant ledger. Unknown image → clear error listing
   configured images.

## Env contract

```
HICKORY_EXECUTOR=local|canopy
CANOPY_URL=https://…           # control plane (GraphQL)
CANOPY_TOKEN=…                 # base64 capability token (spawn/read ops)
CANOPY_NODE=colo-1
CANOPY_IMAGE_MAP={"python:3.12":"/nix/store/…","alpine:3.20":"/nix/store/…"}
```

`GET /api/health` reports which executor is active and whether canopy is
reachable. With `HICKORY_EXECUTOR=local`, canopy vars are ignored.
