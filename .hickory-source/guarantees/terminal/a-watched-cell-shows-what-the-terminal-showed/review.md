<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Watched Cell Shows What The Terminal Showed: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/CellPanel.tsx](../../../shared/files/apps/web/src/components/CellPanel.tsx.md)
- [apps/web/src/lib/watchStream.ts](files/apps/web/src/lib/watchStream.ts.md)
- [apps/web/src/terminal/WatchingTerminal.tsx](files/apps/web/src/terminal/WatchingTerminal.tsx.md)
- [apps/web/src/test-setup.ts](../../../shared/files/apps/web/src/test-setup.ts.md)

## Tests and fixtures

- [apps/web/src/lib/watchStream.test.ts](files/apps/web/src/lib/watchStream.test.ts.md)
- [apps/web/src/terminal/WatchingTerminal.test.tsx](files/apps/web/src/terminal/WatchingTerminal.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
