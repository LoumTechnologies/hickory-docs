<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Captured reading surfaces share owned native actions

For: Authors using an agent to change documents while retaining review control.

Given the lineage browser, launch-profile picker, editor ruler, recorded session
cards, function-profile evidence or constructed patch editor in an attached UI,
when the agent observes and invokes a registered action, it uses the corresponding
native controller and its current owner, revision, enabled state and disclosure.
Replacing the document, project, picker or captured run retires prior handles.
Reading a collapsed surface does not disclose its folded body. The command result
separates a requested view/selection/pin from completed replay, process startup,
provider delivery, verification or persistence.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/scoped-reading-surfaces-share-native-actions.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation chapters

These chapters keep this behavior's larger implementation readable without duplicating shared code.

- [`apps/web/src/components/ComparisonEditor.tsx`](implementation/apps/web/src/components/ComparisonEditor.tsx.md#output-2884182333a4)
- [`apps/web/src/components/Engaged.test.tsx`](implementation/apps/web/src/components/Engaged.test.tsx.md#output-45bf37888c2a)
- [`apps/web/src/components/EvidenceDisclosure.tsx`](implementation/apps/web/src/components/EvidenceDisclosure.tsx.md#output-1872aa99abeb)
- [`apps/web/src/debug/DebugRail.test.tsx`](implementation/apps/web/src/debug/DebugRail.test.tsx.md#output-141b8a9dd63d)
- [`apps/web/src/debug/DebugRail.tsx`](implementation/apps/web/src/debug/DebugRail.tsx.md#output-3f2d1a1c0986)
- [`apps/web/src/debug/ProfileEvidence.test.tsx`](implementation/apps/web/src/debug/ProfileEvidence.test.tsx.md#output-8198489c1f35)
- [`apps/web/src/debug/ProfileEvidence.tsx`](implementation/apps/web/src/debug/ProfileEvidence.tsx.md#output-9b75394c3f2e)
- [`apps/web/src/debug/useDebugLaunchActions.ts`](implementation/apps/web/src/debug/useDebugLaunchActions.ts.md#output-5fc802a7856a)
- [`apps/web/src/editor/replaceCapturedBlock.test.ts`](implementation/apps/web/src/editor/replaceCapturedBlock.test.ts.md#output-cf677803e44e)
- [`apps/web/src/lib/capturedCode.test.ts`](implementation/apps/web/src/lib/capturedCode.test.ts.md#output-2a3bb47f729a)
- [`apps/web/src/lineage/HoleRow.tsx`](implementation/apps/web/src/lineage/HoleRow.tsx.md#output-51e109e34f87)
- [`apps/web/src/lineage/LineageColumns.test.tsx`](implementation/apps/web/src/lineage/LineageColumns.test.tsx.md#output-0ea1a191da2e)
- [`apps/web/src/lineage/LineageColumns.tsx`](implementation/apps/web/src/lineage/LineageColumns.tsx.md#output-d5afe1148ce1)
- [`apps/web/src/lineage/StageColumn.tsx`](implementation/apps/web/src/lineage/StageColumn.tsx.md#output-4d2e91f01732)
- [`apps/web/src/lineage/useLineageControls.ts`](implementation/apps/web/src/lineage/useLineageControls.ts.md#output-9bd624eaf8c9)
- [`apps/web/src/views/CommitReading.controls.test.tsx`](implementation/apps/web/src/views/CommitReading.controls.test.tsx.md#output-ee73e4c07840)
- [`apps/web/src/views/LineageView.test.tsx`](implementation/apps/web/src/views/LineageView.test.tsx.md#output-48d7da1042ae)
<storycode:include file="implementation/apps/web/src/components/ComparisonEditor.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/Engaged.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/EvidenceDisclosure.tsx.md" />
<storycode:include file="implementation/apps/web/src/debug/DebugRail.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/debug/DebugRail.tsx.md" />
<storycode:include file="implementation/apps/web/src/debug/ProfileEvidence.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/debug/ProfileEvidence.tsx.md" />
<storycode:include file="implementation/apps/web/src/debug/useDebugLaunchActions.ts.md" />
<storycode:include file="implementation/apps/web/src/editor/replaceCapturedBlock.test.ts.md" />
<storycode:include file="implementation/apps/web/src/lib/capturedCode.test.ts.md" />
<storycode:include file="implementation/apps/web/src/lineage/HoleRow.tsx.md" />
<storycode:include file="implementation/apps/web/src/lineage/LineageColumns.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/lineage/LineageColumns.tsx.md" />
<storycode:include file="implementation/apps/web/src/lineage/StageColumn.tsx.md" />
<storycode:include file="implementation/apps/web/src/lineage/useLineageControls.ts.md" />
<storycode:include file="implementation/apps/web/src/views/CommitReading.controls.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/views/LineageView.test.tsx.md" />

## Implementation review

### Implementation

- [apps/web/src/components/ComparisonEditor.tsx](implementation/apps/web/src/components/ComparisonEditor.tsx.md#output-2884182333a4)
- [apps/web/src/components/EvidenceDisclosure.tsx](implementation/apps/web/src/components/EvidenceDisclosure.tsx.md#output-1872aa99abeb)
- [apps/web/src/debug/DebugRail.tsx](implementation/apps/web/src/debug/DebugRail.tsx.md#output-3f2d1a1c0986)
- [apps/web/src/debug/ProfileEvidence.tsx](implementation/apps/web/src/debug/ProfileEvidence.tsx.md#output-9b75394c3f2e)
- [apps/web/src/debug/launchProfiles.tsx](../../../mechanisms/apps/web/src/debug/launchProfiles.tsx.md#output-1a7a9a03aa8d)
- [apps/web/src/debug/useDebugLaunchActions.ts](implementation/apps/web/src/debug/useDebugLaunchActions.ts.md#output-5fc802a7856a)
- [apps/web/src/editor/EditorRuler.tsx](../../../mechanisms/apps/web/src/editor/EditorRuler.tsx.md#output-75b1988d3607)
- [apps/web/src/elements/patch/Editor.tsx](../../../mechanisms/apps/web/src/elements/patch/Editor.tsx.md#output-dc5641c063e3)
- [apps/web/src/elements/session/view.tsx](../../../mechanisms/apps/web/src/elements/session/view.tsx.md#output-d096a8a10df6)
- [apps/web/src/lineage/HoleRow.tsx](implementation/apps/web/src/lineage/HoleRow.tsx.md#output-51e109e34f87)
- [apps/web/src/lineage/LineageColumns.tsx](implementation/apps/web/src/lineage/LineageColumns.tsx.md#output-d5afe1148ce1)
- [apps/web/src/lineage/StageColumn.tsx](implementation/apps/web/src/lineage/StageColumn.tsx.md#output-4d2e91f01732)
- [apps/web/src/lineage/TimeSlider.tsx](../../../mechanisms/apps/web/src/lineage/TimeSlider.tsx.md#output-dca5a7314b59)
- [apps/web/src/lineage/useLineageControls.ts](implementation/apps/web/src/lineage/useLineageControls.ts.md#output-9bd624eaf8c9)
- [apps/web/src/views/LineageView.tsx](../../../mechanisms/apps/web/src/views/LineageView.tsx.md#output-f7cde247eee4)
- [apps/web/src/views/SessionLens.tsx](../../../mechanisms/apps/web/src/views/SessionLens.tsx.md#output-11b8d4e005a0)

### Tests and fixtures

- [apps/web/src/components/Engaged.test.tsx](implementation/apps/web/src/components/Engaged.test.tsx.md#output-45bf37888c2a)
- [apps/web/src/components/graph/GraphEditorPanel.test.tsx](../../../mechanisms/apps/web/src/components/graph/GraphEditorPanel.test.tsx.md#output-b7404652ca7f)
- [apps/web/src/debug/DebugRail.test.tsx](implementation/apps/web/src/debug/DebugRail.test.tsx.md#output-141b8a9dd63d)
- [apps/web/src/debug/ProfileEvidence.test.tsx](implementation/apps/web/src/debug/ProfileEvidence.test.tsx.md#output-8198489c1f35)
- [apps/web/src/debug/launchProfiles.test.tsx](../../../mechanisms/apps/web/src/debug/launchProfiles.tsx.md#output-af7d3cd605b8)
- [apps/web/src/editor/EditorRuler.test.tsx](../../../mechanisms/apps/web/src/editor/EditorRuler.tsx.md#output-a0ab5d3c94e1)
- [apps/web/src/editor/replaceCapturedBlock.test.ts](implementation/apps/web/src/editor/replaceCapturedBlock.test.ts.md#output-cf677803e44e)
- [apps/web/src/elements/patch/Editor.test.tsx](../../../mechanisms/apps/web/src/elements/patch/Editor.tsx.md#output-dc4ea70ab787)
- [apps/web/src/elements/session/evidence.test.tsx](../../../mechanisms/apps/web/src/elements/session/evidence.test.tsx.md#output-4f5d36dc9d9d)
- [apps/web/src/lib/capturedCode.test.ts](implementation/apps/web/src/lib/capturedCode.test.ts.md#output-2a3bb47f729a)
- [apps/web/src/lineage/LineageColumns.test.tsx](implementation/apps/web/src/lineage/LineageColumns.test.tsx.md#output-0ea1a191da2e)
- [apps/web/src/views/CommitReading.controls.test.tsx](implementation/apps/web/src/views/CommitReading.controls.test.tsx.md#output-ee73e4c07840)
- [apps/web/src/views/LineageView.test.tsx](implementation/apps/web/src/views/LineageView.test.tsx.md#output-48d7da1042ae)
- [apps/web/src/views/SessionLens.test.tsx](../../../mechanisms/apps/web/src/views/SessionLens.tsx.md#output-3a59c565f3d4)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
