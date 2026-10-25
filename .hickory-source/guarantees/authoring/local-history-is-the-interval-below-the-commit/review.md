<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Local History Is The Interval Below The Commit: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hickory-cli/src/history.rs](files/crates/hickory-cli/src/history.rs.md)
- [crates/hickory-cli/src/ingest_exec.rs](../../../shared/files/crates/hickory-cli/src/ingest_exec.rs.md)
- [crates/hickory-cli/src/serve/find.rs](../../search/find-and-replace-is-exhaustive/files/crates/hickory-cli/src/serve/find.rs.md)
- [crates/hickory-cli/src/up/reverse.rs](../../../shared/files/crates/hickory-cli/src/up/reverse.rs.md)
- [crates/hickory-workspace/src/history.rs](files/crates/hickory-workspace/src/history.rs.md)

## Tests and fixtures

- [crates/hickory-cli/tests/serve_find.rs](../../../shared/files/crates/hickory-cli/tests/serve_find.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
