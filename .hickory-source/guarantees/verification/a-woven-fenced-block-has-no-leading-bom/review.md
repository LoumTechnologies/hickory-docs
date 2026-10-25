<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Woven Fenced Block Has No Leading BOM: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-literate/src/lib.rs](../../../shared/files/crates/hick-literate/src/lib.rs.md)
- [crates/hick-literate/src/weave.rs](../../../shared/files/crates/hick-literate/src/weave.rs.md)

## Tests and fixtures

- [crates/hick-lang/tests/fixtures/efcore-project/app/Program.cs](../../../support/files/crates/hick-lang/tests/fixtures/efcore-project/app/Program.cs.md)
- [crates/hick-literate/src/lib/tests/tests.rs](../../../shared/files/crates/hick-literate/src/lib/tests/tests.rs.md)
- [crates/hickory-cli/tests/weave_reads_naturally.rs](../../../shared/files/crates/hickory-cli/tests/weave_reads_naturally.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
