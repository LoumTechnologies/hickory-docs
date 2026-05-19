# network

Full virtual network stack for hick WASM containers, built on `smoltcp` and
Tokio.

## Capabilities

- **DHCP** — address assignment to containers
- **DNS** — forwarding to upstream resolver
- **TCP proxy** — transparent egress proxying
- **HTTP interception** — intercepts and inspects HTTP traffic
- **TLS MITM** — ephemeral CA via `rcgen`/`rustls`; SNI-based per-host cert
  issuance
- **Auth injection** — inserts secrets into HTTP headers keyed by host rules
  from `hick-secrets`

## Key types

- `VirtualNetwork` — owns the smoltcp interface and socket set
- `VirtualDevice` — in-memory Ethernet frame queues
- `HickCa` — per-host ephemeral TLS CA
- `InterceptorRegistry` — registry of traffic interceptors
- `AuthInjector` — injects auth headers based on host rules

## Entry points

`run_network`, `run_network_with_config`, `run_network_bridged`
