<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Generated Picture Shows As A Picture: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/PicturePanel.tsx](files/apps/web/src/components/PicturePanel.tsx.md)
- [apps/web/src/editor/CardRail.tsx](../../../shared/files/apps/web/src/editor/CardRail.tsx.md)
- [apps/web/src/editor/DocumentEditor.tsx](../../../shared/files/apps/web/src/editor/DocumentEditor.tsx.md)
- [apps/web/src/editor/cards.ts](../../../shared/files/apps/web/src/editor/cards.ts.md)
- [apps/web/src/editor/hickDoc.ts](../../../shared/files/apps/web/src/editor/hickDoc.ts.md)
- [apps/web/src/editor/rendered.ts](../../../shared/files/apps/web/src/editor/rendered.ts.md)
- [apps/web/src/lib/railActions.ts](../../../shared/files/apps/web/src/lib/railActions.ts.md)
- [apps/web/src/styles.css](../../../shared/files/apps/web/src/styles.css.md)
- [crates/hick-literate/src/weave.rs](../../../shared/files/crates/hick-literate/src/weave.rs.md)

## Tests and fixtures

- [apps/web/src/editor/cards.test.ts](../../../shared/files/apps/web/src/editor/cards.test.ts.md)
- [apps/web/src/editor/rendered.test.ts](../../../shared/files/apps/web/src/editor/rendered.test.ts.md)
- [apps/web/src/lib/railActions.test.ts](../../../shared/files/apps/web/src/lib/railActions.test.ts.md)
- [crates/hick-literate/tests/picture_weave.rs](../../../shared/files/crates/hick-literate/tests/picture_weave.rs.md)
- [crates/hickory-cli/tests/weave_keeps_artifacts.rs](../../../shared/files/crates/hickory-cli/tests/weave_keeps_artifacts.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
