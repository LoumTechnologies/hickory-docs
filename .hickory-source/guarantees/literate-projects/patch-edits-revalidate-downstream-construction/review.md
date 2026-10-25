<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Keep a patch edit as a draft until the whole construction still works: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/elements/patch/Editor.tsx](../../../shared/files/apps/web/src/elements/patch/Editor.tsx.md)
- [crates/hick-lang/src/project/edit.rs](files/crates/hick-lang/src/project/edit.rs.md)
- [crates/hick-lang/src/project/reconstruct.rs](../finalization-and-incorporation-check-distinct-equalities/files/crates/hick-lang/src/project/reconstruct.rs.md)

## Tests and fixtures

- [apps/web/src/components/ChangesetDocument.test.tsx](../../../shared/files/apps/web/src/components/ChangesetDocument.test.tsx.md)
- [apps/web/src/elements/patch/Editor.test.tsx](../../../shared/files/apps/web/src/elements/patch/Editor.test.tsx.md)
- [apps/web/src/lib/projectComparison.test.ts](../../../shared/files/apps/web/src/lib/projectComparison.test.ts.md)
- [crates/hick-lang/tests/project.rs](../../../shared/files/crates/hick-lang/tests/project.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
