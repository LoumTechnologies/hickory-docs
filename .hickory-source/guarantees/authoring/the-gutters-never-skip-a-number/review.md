<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Gutters Never Skip A Number: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/MathPanel.tsx](../../../shared/files/apps/web/src/components/MathPanel.tsx.md)
- [apps/web/src/editor/CardRail.tsx](../../../shared/files/apps/web/src/editor/CardRail.tsx.md)
- [apps/web/src/editor/DocumentEditor.tsx](../../../shared/files/apps/web/src/editor/DocumentEditor.tsx.md)
- [apps/web/src/editor/cards.ts](../../../shared/files/apps/web/src/editor/cards.ts.md)
- [apps/web/src/editor/mdLinks.ts](../../../shared/files/apps/web/src/editor/mdLinks.ts.md)
- [apps/web/src/editor/rendered.ts](../../../shared/files/apps/web/src/editor/rendered.ts.md)
- [apps/web/src/editor/taskList.ts](../../../shared/files/apps/web/src/editor/taskList.ts.md)
- [apps/web/src/editor/wrapGutter.ts](../../../support/files/apps/web/src/editor/wrapGutter.ts.md)
- [apps/web/src/editor/wysiwyg.ts](../../../shared/files/apps/web/src/editor/wysiwyg.ts.md)
- [apps/web/src/lib/cardRail.ts](../../../shared/files/apps/web/src/lib/cardRail.ts.md)
- [apps/web/src/lib/railActions.ts](../../../shared/files/apps/web/src/lib/railActions.ts.md)
- [apps/web/src/shell/Ribbons.tsx](../../../shared/files/apps/web/src/shell/Ribbons.tsx.md)
- [apps/web/src/styles.css](../../../shared/files/apps/web/src/styles.css.md)
- [scripts/dev-seed.sh](files/scripts/dev-seed.sh.md)

## Tests and fixtures

- [apps/web/src/editor/DocumentEditor.test.tsx](../../../shared/files/apps/web/src/editor/DocumentEditor.test.tsx.md)
- [apps/web/src/editor/cards.test.ts](../../../shared/files/apps/web/src/editor/cards.test.ts.md)
- [apps/web/src/lib/cardRail.test.ts](files/apps/web/src/lib/cardRail.test.ts.md)
- [apps/web/src/styles.test.ts](../../../shared/files/apps/web/src/styles.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
