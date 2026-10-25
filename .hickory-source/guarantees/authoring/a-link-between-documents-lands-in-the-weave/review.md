<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Link Between Documents Lands In The Weave: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/lib/mdLinks.ts](../../../shared/files/apps/web/src/lib/mdLinks.ts.md)
- [crates/hick-literate/src/links.rs](../../../shared/files/crates/hick-literate/src/links.rs.md)

## Tests and fixtures

- [apps/web/src/lib/mdLinks.test.ts](../../../shared/files/apps/web/src/lib/mdLinks.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve.

</book:doc>
