<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Cell Cannot Hang A Run: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-exec/src/dag.rs](../../../shared/files/crates/hick-exec/src/dag.rs.md)
- [crates/hick-literate/src/cell_timeout.rs](files/crates/hick-literate/src/cell_timeout.rs.md)
- [crates/hick-literate/src/lib.rs](../../../shared/files/crates/hick-literate/src/lib.rs.md)
- [crates/hickory-cli/src/lib.rs](../../../shared/files/crates/hickory-cli/src/lib.rs.md)
- [crates/hickory-executor-canopy/src/executor.rs](../../../shared/files/crates/hickory-executor-canopy/src/executor.rs.md)
- [crates/hickory-executor-docker/src/lib.rs](../../../shared/files/crates/hickory-executor-docker/src/lib.rs.md)
- [crates/hickory-executor-sandbox/src/lib.rs](../../../shared/files/crates/hickory-executor-sandbox/src/lib.rs.md)
- [crates/hickory-executor/src/lib.rs](../../../shared/files/crates/hickory-executor/src/lib.rs.md)

## Tests and fixtures

- [crates/hick-exec/src/dag/tests/tests.rs](../../../shared/files/crates/hick-exec/src/dag/tests/tests.rs.md)
- [crates/hick-literate/tests/cell_timeout_tests.rs](files/crates/hick-literate/tests/cell_timeout_tests.rs.md)
- [crates/hickory-cli/tests/cell_timeout_env.rs](files/crates/hickory-cli/tests/cell_timeout_env.rs.md)
- [crates/hickory-executor-sandbox/tests/confinement.rs](../../../shared/files/crates/hickory-executor-sandbox/tests/confinement.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
