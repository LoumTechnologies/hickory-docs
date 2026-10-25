<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Expectation Means The Same Thing On Every Platform: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-literate/src/expect.rs](../../../shared/files/crates/hick-literate/src/expect.rs.md)
- [crates/hickory-executor-canopy/src/frame.rs](files/crates/hickory-executor-canopy/src/frame.rs.md)
- [crates/hickory-executor-docker/src/lib.rs](../../../shared/files/crates/hickory-executor-docker/src/lib.rs.md)
- [crates/hickory-executor/src/capture.rs](files/crates/hickory-executor/src/capture.rs.md)
- [crates/hickory-executor/src/lib.rs](../../../shared/files/crates/hickory-executor/src/lib.rs.md)

## Tests and fixtures

- [crates/hickory-cli/tests/common/mod.rs](files/crates/hickory-cli/tests/common/mod.rs.md)
- [crates/hickory-cli/tests/test_command_tests.rs](../../../shared/files/crates/hickory-cli/tests/test_command_tests.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
