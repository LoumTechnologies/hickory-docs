# hick-token

Macaroon-based capability token system for containers.

`TokenAuthority` mints and verifies `CapabilityToken`s (backed by the `macaroon`
crate). Each token encodes a `ContainerCapabilities` struct as first-party
caveats covering:

- Network rules (allowed hosts/ports)
- File and volume access
- Secret injection
- Data-access patterns
- API sinks
- Purpose constraints
- Expiry and max-calls limits
- Recipient restrictions

Capabilities are **strictly attenuating**: `CapabilityToken::attenuate()` only
adds restrictions, never removes them. `ContainerCapabilities::intersect()`
computes the most restrictive combination of two capability sets.

This crate is the central authority layer — consumed by `hick-sink` for
enforcement and by the agent executor for token issuance.
