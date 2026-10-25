<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Equation Renders And Shows Its Source: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/desktop/src-tauri/src/lib.rs](../../../shared/files/apps/desktop/src-tauri/src/lib.rs.md)
- [apps/web/src/components/MathPanel.tsx](../../../shared/files/apps/web/src/components/MathPanel.tsx.md)
- [apps/web/src/editor/CardRail.tsx](../../../shared/files/apps/web/src/editor/CardRail.tsx.md)
- [apps/web/src/editor/DocumentEditor.tsx](../../../shared/files/apps/web/src/editor/DocumentEditor.tsx.md)
- [apps/web/src/editor/cards.ts](../../../shared/files/apps/web/src/editor/cards.ts.md)
- [apps/web/src/editor/mathRender.ts](../../../shared/files/apps/web/src/editor/mathRender.ts.md)
- [apps/web/src/editor/rendered.ts](../../../shared/files/apps/web/src/editor/rendered.ts.md)
- [apps/web/src/lib/insertCatalog.ts](../../../shared/files/apps/web/src/lib/insertCatalog.ts.md)
- [apps/web/src/lib/math.ts](files/apps/web/src/lib/math.ts.md)
- [apps/web/src/lib/railActions.ts](../../../shared/files/apps/web/src/lib/railActions.ts.md)
- [apps/web/src/styles.css](../../../shared/files/apps/web/src/styles.css.md)
- [crates/hick-literate/src/weave.rs](../../../shared/files/crates/hick-literate/src/weave.rs.md)

## Tests and fixtures

- [apps/desktop/src-tauri/tests/insert_menu_matches_catalogue.rs](../../../shared/files/apps/desktop/src-tauri/tests/insert_menu_matches_catalogue.rs.md)
- [apps/web/src/editor/mathRender.test.ts](files/apps/web/src/editor/mathRender.test.ts.md)
- [apps/web/src/lib/math.test.ts](files/apps/web/src/lib/math.test.ts.md)
- [crates/hickory-cli/tests/math.rs](files/crates/hickory-cli/tests/math.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
