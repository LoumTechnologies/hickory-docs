<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Derived Diagram Keeps Your Layout: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/graph/GraphEditorPanel.tsx](../a-drawn-diagram-is-document-text/files/apps/web/src/components/graph/GraphEditorPanel.tsx.md)
- [apps/web/src/components/graph/scene.ts](../../../shared/files/apps/web/src/components/graph/scene.ts.md)
- [apps/web/src/editor/DocumentEditor.tsx](../../../shared/files/apps/web/src/editor/DocumentEditor.tsx.md)
- [crates/hick-literate/src/render.rs](../../../shared/files/crates/hick-literate/src/render.rs.md)
- [crates/hick-literate/src/scene.rs](../../../shared/files/crates/hick-literate/src/scene.rs.md)

## Tests and fixtures

- [apps/web/src/components/graph/GraphEditorPanel.test.tsx](../../../shared/files/apps/web/src/components/graph/GraphEditorPanel.test.tsx.md)
- [apps/web/src/components/graph/scene.test.ts](../../../shared/files/apps/web/src/components/graph/scene.test.ts.md)
- [crates/hickory-cli/tests/diagrams.rs](../../../shared/files/crates/hickory-cli/tests/diagrams.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
