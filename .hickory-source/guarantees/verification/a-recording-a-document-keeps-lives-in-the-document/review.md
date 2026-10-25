<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Recording A Document Keeps Lives In The Document: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-lang/src/lib.rs](../../../shared/files/crates/hick-lang/src/lib.rs.md)
- [crates/hick-literate/src/lib.rs](../../../shared/files/crates/hick-literate/src/lib.rs.md)
- [crates/hick-literate/src/render.rs](../../../shared/files/crates/hick-literate/src/render.rs.md)
- [crates/hick-literate/src/weave.rs](../../../shared/files/crates/hick-literate/src/weave.rs.md)
- [crates/hickory-cli/src/ingest_recording.rs](../../../shared/files/crates/hickory-cli/src/ingest_recording.rs.md)
- [crates/hickory-cli/src/init.rs](../../../shared/files/crates/hickory-cli/src/init.rs.md)
- [crates/hickory-cli/src/main.rs](../../../shared/files/crates/hickory-cli/src/main.rs.md)

## Tests and fixtures

- [crates/hickory-cli/tests/round_trip.rs](../../../shared/files/crates/hickory-cli/tests/round_trip.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
