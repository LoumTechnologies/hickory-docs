<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Coverage retains its measured scope: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/coverage/gutter.ts](files/apps/web/src/coverage/gutter.ts.md)
- [crates/hick-coverage/src/compare.rs](files/crates/hick-coverage/src/compare.rs.md)
- [crates/hick-coverage/src/import.rs](files/crates/hick-coverage/src/import.rs.md)
- [crates/hick-coverage/src/lib.rs](files/crates/hick-coverage/src/lib.rs.md)
- [crates/hickory-cli/src/coverage/mapping.rs](../../../support/files/crates/hickory-cli/src/coverage/mapping.rs.md)
- [crates/hickory-cli/src/coverage/tests.rs](files/crates/hickory-cli/src/coverage/tests.rs.md)

## Tests and fixtures

- [apps/web/src/coverage/CoverageTools.test.tsx](files/apps/web/src/coverage/CoverageTools.test.tsx.md)
- [apps/web/src/coverage/gutter.test.ts](files/apps/web/src/coverage/gutter.test.ts.md)
- [apps/web/src/coverage/selection.test.ts](files/apps/web/src/coverage/selection.test.ts.md)
- [crates/hick-coverage/tests/evidence.rs](files/crates/hick-coverage/tests/evidence.rs.md)
- [crates/hickory-cli/src/serve/test_coverage.rs](../../../support/files/crates/hickory-cli/src/serve/test_coverage.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve.

</book:doc>
