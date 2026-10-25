<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Show the requested connections without changing their meaning: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/lib/coalesceRibbons.ts](../../../support/files/apps/web/src/lib/coalesceRibbons.ts.md)
- [apps/web/src/lib/ribbonVisibility.ts](files/apps/web/src/lib/ribbonVisibility.ts.md)
- [apps/web/src/shell/Ribbons.tsx](../../../shared/files/apps/web/src/shell/Ribbons.tsx.md)
- [apps/web/src/shell/coalesceLinks.ts](../../../support/files/apps/web/src/shell/coalesceLinks.ts.md)
- [apps/web/src/shell/linkShapes.ts](../../../support/files/apps/web/src/shell/linkShapes.ts.md)
- [apps/web/src/views/SettingsView.tsx](../../../shared/files/apps/web/src/views/SettingsView.tsx.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)

## Tests and fixtures

- [apps/web/src/lib/ribbonVisibility.test.ts](files/apps/web/src/lib/ribbonVisibility.test.ts.md)
- [apps/web/src/views/SettingsView.test.tsx](../../../shared/files/apps/web/src/views/SettingsView.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
