# connection-filter

Async trait and implementations for allow/block decisions on outbound TCP
connections and DNS queries.

## Implementations

| Filter | Description |
|---|---|
| `AllowAll` | Passes all connections and queries unconditionally |
| `SwappableFilter` | Wraps any filter behind `ArcSwap` for lock-free hot-swap at runtime |
| `CapabilityFilter` (feature: `capability`) | Validates requests against `hick-token::ContainerCapabilities` macaroon rules; maintains a DNS→IP cache so TCP destination IPs can be matched back to hostnames |

`SwappableFilter` is used to replace a startup placeholder filter with the real
capability filter without stopping the network loop.
