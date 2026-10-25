<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Generated File Starts At Its First Byte: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-literate/src/lib.rs](../../../shared/files/crates/hick-literate/src/lib.rs.md)
- [crates/hickory-cli/src/adopt.rs](../../../shared/files/crates/hickory-cli/src/adopt.rs.md)
- [crates/hickory-cli/src/ingest_exec.rs](../../../shared/files/crates/hickory-cli/src/ingest_exec.rs.md)

## Tests and fixtures

- [crates/hick-literate/tests/file_first_byte.rs](files/crates/hick-literate/tests/file_first_byte.rs.md)
- [crates/hick-literate/tests/picture_weave.rs](../../../shared/files/crates/hick-literate/tests/picture_weave.rs.md)
- [crates/hickory-cli/tests/ingest_scaffold.rs](../../../shared/files/crates/hickory-cli/tests/ingest_scaffold.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
