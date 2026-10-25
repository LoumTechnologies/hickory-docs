<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Only `hickory-executor-canopy` Knows Cloud Canopy's API: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hickory-cli/src/lib.rs](../../../shared/files/crates/hickory-cli/src/lib.rs.md)
- [crates/hickory-executor-canopy/src/executor.rs](../../../shared/files/crates/hickory-executor-canopy/src/executor.rs.md)
- [crates/hickory-executor-canopy/src/frame.rs](../../verification/an-expectation-means-the-same-on-every-platform/files/crates/hickory-executor-canopy/src/frame.rs.md)
- [crates/hickory-executor-canopy/src/lib.rs](files/crates/hickory-executor-canopy/src/lib.rs.md)

## Tests and fixtures

- [crates/hickory-cli/tests/test_command_tests.rs](../../../shared/files/crates/hickory-cli/tests/test_command_tests.rs.md)
- [crates/hickory-executor-canopy/tests/contract.rs](files/crates/hickory-executor-canopy/tests/contract.rs.md)
- [crates/hickory-executor-canopy/tests/isolation.rs](files/crates/hickory-executor-canopy/tests/isolation.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
