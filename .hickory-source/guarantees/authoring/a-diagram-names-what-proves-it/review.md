<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Diagram Names What Proves It: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/DiagramPanel.tsx](../../../shared/files/apps/web/src/components/DiagramPanel.tsx.md)
- [apps/web/src/editor/DocumentEditor.tsx](../../../shared/files/apps/web/src/editor/DocumentEditor.tsx.md)
- [apps/web/src/editor/rendered.ts](../../../shared/files/apps/web/src/editor/rendered.ts.md)
- [crates/hick-literate/src/render.rs](../../../shared/files/crates/hick-literate/src/render.rs.md)
- [crates/hick-literate/src/weave.rs](../../../shared/files/crates/hick-literate/src/weave.rs.md)
- [crates/hickory-cli/src/lib.rs](../../../shared/files/crates/hickory-cli/src/lib.rs.md)

## Tests and fixtures

- [apps/web/src/components/DiagramPanel.test.tsx](../../../shared/files/apps/web/src/components/DiagramPanel.test.tsx.md)
- [apps/web/src/editor/DocumentEditor.test.tsx](../../../shared/files/apps/web/src/editor/DocumentEditor.test.tsx.md)
- [crates/hickory-cli/tests/diagrams.rs](../../../shared/files/crates/hickory-cli/tests/diagrams.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
