<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Apply repository exclusions when seeding execution inputs: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/lib/scaffold.ts](files/apps/web/src/lib/scaffold.ts.md)
- [crates/hick-blocks/src/lib.rs](../../../shared/files/crates/hick-blocks/src/lib.rs.md)
- [crates/hick-exec/src/dag.rs](../../../shared/files/crates/hick-exec/src/dag.rs.md)
- [crates/hick-literate/src/lib.rs](../../../shared/files/crates/hick-literate/src/lib.rs.md)

## Tests and fixtures

- [crates/hick-literate/src/lib/tests/tests.rs](../../../shared/files/crates/hick-literate/src/lib/tests/tests.rs.md)
- [crates/hickory-cli/tests/serve_scaffold.rs](../../../shared/files/crates/hickory-cli/tests/serve_scaffold.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
