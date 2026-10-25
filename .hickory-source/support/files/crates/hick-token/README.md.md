<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-token/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `559a236cf1db47a010bcc06d04f5f245ccb3da313e61291a44d7459ed993b97f`.

<book:entry path="crates/hick-token/README.md">
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
</book:entry>

</book:doc>
