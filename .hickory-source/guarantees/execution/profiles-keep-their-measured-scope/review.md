<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Profiles keep their measured scope: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/performance/inline.ts](files/apps/web/src/performance/inline.ts.md)
- [crates/hickory-cli/src/performance/browser.rs](../../../support/files/crates/hickory-cli/src/performance/browser.rs.md)
- [crates/hickory-cli/src/performance/native.rs](../../../support/files/crates/hickory-cli/src/performance/native.rs.md)
- [crates/hickory-cli/src/performance/resolve.rs](../../../support/files/crates/hickory-cli/src/performance/resolve.rs.md)
- [crates/hickory-cli/src/performance/tests.rs](files/crates/hickory-cli/src/performance/tests.rs.md)
- [crates/hickory-cli/src/source_mapping.rs](../../../support/files/crates/hickory-cli/src/source_mapping.rs.md)

## Tests and fixtures

- [apps/web/src/performance/PerformanceTools.test.tsx](files/apps/web/src/performance/PerformanceTools.test.tsx.md)
- [apps/web/src/performance/inline.test.ts](files/apps/web/src/performance/inline.test.ts.md)
- [apps/web/src/performance/selection.test.ts](files/apps/web/src/performance/selection.test.ts.md)
- [crates/hick-profile/tests/metrics.rs](files/crates/hick-profile/tests/metrics.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
