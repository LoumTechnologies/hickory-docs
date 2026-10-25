<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# S3 publishes one manifest: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/embed/s3Storage.ts](files/apps/web/src/embed/s3Storage.ts.md)

## Tests and fixtures

- [apps/web/src/embed/s3Storage.test.ts](files/apps/web/src/embed/s3Storage.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
