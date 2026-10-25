<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Runtime observations retain their producing revision and occurrence: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-observe/examples/local_run.rs](../../../support/files/crates/hick-observe/examples/local_run.rs.md)
- [crates/hick-observe/src/lib.rs](../../../shared/files/crates/hick-observe/src/lib.rs.md)
- [crates/hick-observe/src/reading.rs](../../debugging/source-anchored-observation-reading/files/crates/hick-observe/src/reading.rs.md)
- [crates/hick-observe/src/tests.rs](files/crates/hick-observe/src/tests.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
