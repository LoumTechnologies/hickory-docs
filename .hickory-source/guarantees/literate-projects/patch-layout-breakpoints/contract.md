<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Patch layout exposes endpoint breakpoints

For: Maintainers reorganizing source while preserving the complete generated program.

Given a comparison with a debug-capable captured project whose displayed file
matches its complete endpoint, when Patch layout is selected, then its code
rows offer breakpoint toggles through the same debug session as the other
layouts. Removed lines address the old endpoint, added lines address the new
endpoint, and context lines offer both. Hunk headers and metadata offer none.
Line numbers come from the unified diff's hunk coordinates. Held breakpoints
show a selected toggle that removes them when clicked.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/literate-projects/patch-layout-breakpoints.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Implementation

## Implementation review

### Implementation

- [apps/web/src/components/ComparisonEditor.tsx](../../agent/scoped-reading-surfaces-share-native-actions/implementation/apps/web/src/components/ComparisonEditor.tsx.md#output-2884182333a4)
- [apps/web/src/components/ComparisonTools.tsx](../../debugging/debug-uses-the-latest-project-containing-the-displayed-code/contract.md#output-455afcfce7fd)
- [apps/web/src/components/DiffView.tsx](../../collaboration/the-git-pane-does-the-daily-loop/contract.md#output-2d0324b58c4d)

### Tests and fixtures

- [apps/web/src/components/ComparisonTools.test.tsx](../../../mechanisms/apps/web/src/components/ComparisonTools.test.tsx.md#output-d1cfc2ed25eb)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
