<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The daemon exits with its last client: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hickory-cli/src/engine/client.rs](../../../shared/files/crates/hickory-cli/src/engine/client.rs.md)
- [crates/hickory-cli/src/engine/daemon.rs](../../../shared/files/crates/hickory-cli/src/engine/daemon.rs.md)
- [crates/hickory-cli/src/engine/mod.rs](../../../shared/files/crates/hickory-cli/src/engine/mod.rs.md)
- [crates/hickory-cli/src/engine/watch.rs](../../../support/files/crates/hickory-cli/src/engine/watch.rs.md)

## Tests and fixtures

- [crates/hickory-cli/tests/engine_lifecycle.rs](../../../shared/files/crates/hickory-cli/tests/engine_lifecycle.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
