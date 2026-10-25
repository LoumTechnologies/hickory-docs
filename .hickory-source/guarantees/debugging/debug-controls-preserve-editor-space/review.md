<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Choose a launch without replacing the editor being read: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/PlainFilePane.tsx](../../../shared/files/apps/web/src/components/PlainFilePane.tsx.md)
- [apps/web/src/debug/DebugRail.tsx](../../agent/scoped-reading-surfaces-share-native-actions/files/apps/web/src/debug/DebugRail.tsx.md)
- [apps/web/src/debug/DebugStrip.tsx](../../../shared/files/apps/web/src/debug/DebugStrip.tsx.md)
- [apps/web/src/views/workspaceTabs.tsx](../../../shared/files/apps/web/src/views/workspaceTabs.tsx.md)

## Tests and fixtures

- [apps/web/src/components/ComparisonTools.test.tsx](../../../shared/files/apps/web/src/components/ComparisonTools.test.tsx.md)
- [apps/web/src/components/PlainFilePane.test.tsx](../../../shared/files/apps/web/src/components/PlainFilePane.test.tsx.md)
- [apps/web/src/debug/DebugRail.test.tsx](../../agent/scoped-reading-surfaces-share-native-actions/files/apps/web/src/debug/DebugRail.test.tsx.md)
- [apps/web/src/debug/DebugStrip.test.tsx](../../../shared/files/apps/web/src/debug/DebugStrip.test.tsx.md)
- [apps/web/src/debug/GeneratedDebugTools.test.tsx](../../../support/files/apps/web/src/debug/GeneratedDebugTools.test.tsx.md)
- [apps/web/src/debug/plainDebugHosts.test.tsx](../launch-profiles-choose-targets-within-the-viewed-project/files/apps/web/src/debug/plainDebugHosts.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</book:doc>
