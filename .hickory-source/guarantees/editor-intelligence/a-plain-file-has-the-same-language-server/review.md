<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Plain File Has The Same Language Server A Document Has: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/PlainFilePane.tsx](../../../shared/files/apps/web/src/components/PlainFilePane.tsx.md)
- [apps/web/src/lib/fileProblems.ts](files/apps/web/src/lib/fileProblems.ts.md)
- [apps/web/src/lib/revealLine.ts](../../../shared/files/apps/web/src/lib/revealLine.ts.md)
- [apps/web/src/lsp/useLsp.ts](files/apps/web/src/lsp/useLsp.ts.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [crates/hick-lsp/src/backend.rs](../../../shared/files/crates/hick-lsp/src/backend.rs.md)
- [crates/hick-lsp/src/position_map.rs](../../../shared/files/crates/hick-lsp/src/position_map.rs.md)

## Tests and fixtures

- [apps/web/src/components/PlainFilePane.test.tsx](../../../shared/files/apps/web/src/components/PlainFilePane.test.tsx.md)
- [crates/hickory-cli/tests/lsp_languages.rs](../../../shared/files/crates/hickory-cli/tests/lsp_languages.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
