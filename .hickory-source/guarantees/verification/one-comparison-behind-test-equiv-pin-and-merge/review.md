<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# One Comparison Behind Test, Equiv, The Pin, And The Merge Check: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-literate/src/equiv.rs](files/crates/hick-literate/src/equiv.rs.md)
- [crates/hickory-cli/src/ingest_recording.rs](../../../shared/files/crates/hickory-cli/src/ingest_recording.rs.md)
- [crates/hickory-cli/src/lib.rs](../../../shared/files/crates/hickory-cli/src/lib.rs.md)
- [crates/hickory-cli/src/main.rs](../../../shared/files/crates/hickory-cli/src/main.rs.md)
- [crates/hickory-cli/src/serve/refactor.rs](../../../shared/files/crates/hickory-cli/src/serve/refactor.rs.md)

## Tests and fixtures

- [crates/hickory-cli/tests/round_trip.rs](../../../shared/files/crates/hickory-cli/tests/round_trip.rs.md)
- [crates/hickory-cli/tests/test_command_tests.rs](../../../shared/files/crates/hickory-cli/tests/test_command_tests.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
