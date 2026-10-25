<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Typed Path Opens That File: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/lib/fileRanking.ts](files/apps/web/src/lib/fileRanking.ts.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [crates/hick-lsp/src/lang_detect.rs](../../../shared/files/crates/hick-lsp/src/lang_detect.rs.md)
- [crates/hick-plugins/src/routing.rs](../../../support/files/crates/hick-plugins/src/routing.rs.md)

## Tests and fixtures

- [apps/web/src/lib/fileRanking.test.ts](files/apps/web/src/lib/fileRanking.test.ts.md)
- [crates/hick-lang/tests/fixtures/literate-project/app.ts](../../../support/files/crates/hick-lang/tests/fixtures/literate-project/app.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
