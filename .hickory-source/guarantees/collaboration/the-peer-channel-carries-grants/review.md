<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Peer Channel Admits Only Keys You Hold, And Serves Them Only What Their Grants Allow: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/views/FleetPane.tsx](../../../shared/files/apps/web/src/views/FleetPane.tsx.md)
- [crates/hickory-cli/src/main.rs](../../../shared/files/crates/hickory-cli/src/main.rs.md)
- [crates/hickory-fleet/src/lib.rs](../../../shared/files/crates/hickory-fleet/src/lib.rs.md)
- [crates/hickory-peer/src/grants.rs](../../../shared/files/crates/hickory-peer/src/grants.rs.md)
- [crates/hickory-peer/src/tunnel.rs](../../../shared/files/crates/hickory-peer/src/tunnel.rs.md)

## Tests and fixtures

- [crates/hickory-peer/tests/peer_channel.rs](../../../shared/files/crates/hickory-peer/tests/peer_channel.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
