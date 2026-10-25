<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Every Hover Hint Is Drawn In The App Theme: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/App.tsx](../../../shared/files/apps/web/src/App.tsx.md)
- [apps/web/src/components/TooltipLayer.tsx](../../../shared/files/apps/web/src/components/TooltipLayer.tsx.md)
- [apps/web/src/debug/cmDebug.ts](../../../shared/files/apps/web/src/debug/cmDebug.ts.md)
- [apps/web/src/editor/folding.ts](../../../shared/files/apps/web/src/editor/folding.ts.md)
- [apps/web/src/editor/mathRender.ts](../../../shared/files/apps/web/src/editor/mathRender.ts.md)
- [apps/web/src/lib/tooltip.ts](../../../shared/files/apps/web/src/lib/tooltip.ts.md)
- [apps/web/src/styles.css](../../../shared/files/apps/web/src/styles.css.md)
- [apps/web/src/views/LandingView.tsx](../../../shared/files/apps/web/src/views/LandingView.tsx.md)

## Tests and fixtures

- [apps/web/src/components/TooltipLayer.test.tsx](../../agent/ui-action-dispatch/files/apps/web/src/components/TooltipLayer.test.tsx.md)
- [apps/web/src/lib/tooltip.test.ts](files/apps/web/src/lib/tooltip.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
