<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A comparison keeps current code editable: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/editor/DocumentEditor.tsx](../../../shared/files/apps/web/src/editor/DocumentEditor.tsx.md)
- [apps/web/src/editor/comparison.ts](../../../support/files/apps/web/src/editor/comparison.ts.md)
- [apps/web/src/lsp/representationMapping.ts](../../../support/files/apps/web/src/lsp/representationMapping.ts.md)
- [apps/web/src/views/RepresentationTools.tsx](../../../support/files/apps/web/src/views/RepresentationTools.tsx.md)
- [crates/hickory-cli/src/serve/revision.rs](../../../support/files/crates/hickory-cli/src/serve/revision.rs.md)

## Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../shared/files/apps/web/e2e/literate-editor.spec.ts.md)
- [apps/web/src/editor/comparison.controls.test.tsx](files/apps/web/src/editor/comparison.controls.test.tsx.md)
- [apps/web/src/editor/comparison.test.ts](../../../support/files/apps/web/src/editor/comparison.test.ts.md)
- [apps/web/src/lsp/cmLspFeatures.test.ts](../../editor-intelligence/save-can-format-first/files/apps/web/src/lsp/cmLspFeatures.test.ts.md)
- [apps/web/src/lsp/representationMapping.test.ts](../../../support/files/apps/web/src/lsp/representationMapping.test.ts.md)
- [crates/hickory-cli/tests/literate_views.rs](../../../support/files/crates/hickory-cli/tests/literate_views.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</book:doc>
