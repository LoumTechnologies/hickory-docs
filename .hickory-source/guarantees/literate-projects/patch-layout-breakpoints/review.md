<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Patch layout exposes endpoint breakpoints: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/ComparisonEditor.tsx](../../agent/scoped-reading-surfaces-share-native-actions/files/apps/web/src/components/ComparisonEditor.tsx.md)
- [apps/web/src/components/ComparisonTools.tsx](../../debugging/debug-uses-the-latest-project-containing-the-displayed-code/files/apps/web/src/components/ComparisonTools.tsx.md)
- [apps/web/src/components/DiffView.tsx](../../collaboration/the-git-pane-does-the-daily-loop/files/apps/web/src/components/DiffView.tsx.md)

## Tests and fixtures

- [apps/web/src/components/ComparisonTools.test.tsx](../../../shared/files/apps/web/src/components/ComparisonTools.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</book:doc>
