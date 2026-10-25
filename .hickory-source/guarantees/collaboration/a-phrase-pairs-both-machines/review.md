<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# One Spoken Phrase Enrols Both Machines, With No Server And Nobody Trusted: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hickory-cli/src/main.rs](../../../shared/files/crates/hickory-cli/src/main.rs.md)
- [crates/hickory-peer/src/pairing.rs](files/crates/hickory-peer/src/pairing.rs.md)
- [crates/hickory-peer/src/tunnel.rs](../../../shared/files/crates/hickory-peer/src/tunnel.rs.md)

## Tests and fixtures

- [apps/web/src/views/FleetPane.test.tsx](../../../shared/files/apps/web/src/views/FleetPane.test.tsx.md)
- [crates/hickory-peer/tests/pairing_cancel.rs](files/crates/hickory-peer/tests/pairing_cancel.rs.md)
- [crates/hickory-peer/tests/peer_channel.rs](../../../shared/files/crates/hickory-peer/tests/peer_channel.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
