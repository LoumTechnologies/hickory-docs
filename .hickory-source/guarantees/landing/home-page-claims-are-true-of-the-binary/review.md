<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Every Claim On The Home Page Is True Of The Downloaded Binary: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/AgentToolSurface.tsx](../../../shared/files/apps/web/src/components/AgentToolSurface.tsx.md)
- [apps/web/src/landing/interests.ts](../../../shared/files/apps/web/src/landing/interests.ts.md)
- [apps/web/src/views/LandingView.tsx](../../../shared/files/apps/web/src/views/LandingView.tsx.md)
- [crates/hickory-cli/src/init.rs](../../../shared/files/crates/hickory-cli/src/init.rs.md)
- [crates/hickory-cli/src/main.rs](../../../shared/files/crates/hickory-cli/src/main.rs.md)

## Tests and fixtures

- [apps/web/src/landing/demos/demos.test.tsx](../../../shared/files/apps/web/src/landing/demos/demos.test.tsx.md)
- [apps/web/src/landing/demos/scripts.test.ts](../../../shared/files/apps/web/src/landing/demos/scripts.test.ts.md)
- [apps/web/src/views/StaticSite.test.tsx](files/apps/web/src/views/StaticSite.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
