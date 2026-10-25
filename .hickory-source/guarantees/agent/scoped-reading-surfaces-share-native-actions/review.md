<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Captured reading surfaces share owned native actions: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/ComparisonEditor.tsx](files/apps/web/src/components/ComparisonEditor.tsx.md)
- [apps/web/src/components/EvidenceDisclosure.tsx](files/apps/web/src/components/EvidenceDisclosure.tsx.md)
- [apps/web/src/debug/DebugRail.tsx](files/apps/web/src/debug/DebugRail.tsx.md)
- [apps/web/src/debug/ProfileEvidence.tsx](files/apps/web/src/debug/ProfileEvidence.tsx.md)
- [apps/web/src/debug/launchProfiles.tsx](../../../shared/files/apps/web/src/debug/launchProfiles.tsx.md)
- [apps/web/src/debug/useDebugLaunchActions.ts](files/apps/web/src/debug/useDebugLaunchActions.ts.md)
- [apps/web/src/editor/EditorRuler.tsx](../../../shared/files/apps/web/src/editor/EditorRuler.tsx.md)
- [apps/web/src/elements/patch/Editor.tsx](../../../shared/files/apps/web/src/elements/patch/Editor.tsx.md)
- [apps/web/src/elements/session/view.tsx](../../../shared/files/apps/web/src/elements/session/view.tsx.md)
- [apps/web/src/lineage/HoleRow.tsx](files/apps/web/src/lineage/HoleRow.tsx.md)
- [apps/web/src/lineage/LineageColumns.tsx](files/apps/web/src/lineage/LineageColumns.tsx.md)
- [apps/web/src/lineage/StageColumn.tsx](files/apps/web/src/lineage/StageColumn.tsx.md)
- [apps/web/src/lineage/TimeSlider.tsx](../../../shared/files/apps/web/src/lineage/TimeSlider.tsx.md)
- [apps/web/src/lineage/useLineageControls.ts](files/apps/web/src/lineage/useLineageControls.ts.md)
- [apps/web/src/views/LineageView.tsx](../../../shared/files/apps/web/src/views/LineageView.tsx.md)
- [apps/web/src/views/SessionLens.tsx](../../../shared/files/apps/web/src/views/SessionLens.tsx.md)

## Tests and fixtures

- [apps/web/src/components/Engaged.test.tsx](files/apps/web/src/components/Engaged.test.tsx.md)
- [apps/web/src/components/graph/GraphEditorPanel.test.tsx](../../../shared/files/apps/web/src/components/graph/GraphEditorPanel.test.tsx.md)
- [apps/web/src/debug/DebugRail.test.tsx](files/apps/web/src/debug/DebugRail.test.tsx.md)
- [apps/web/src/debug/ProfileEvidence.test.tsx](files/apps/web/src/debug/ProfileEvidence.test.tsx.md)
- [apps/web/src/debug/launchProfiles.test.tsx](../../../shared/files/apps/web/src/debug/launchProfiles.test.tsx.md)
- [apps/web/src/editor/EditorRuler.test.tsx](../../../shared/files/apps/web/src/editor/EditorRuler.test.tsx.md)
- [apps/web/src/editor/replaceCapturedBlock.test.ts](files/apps/web/src/editor/replaceCapturedBlock.test.ts.md)
- [apps/web/src/elements/patch/Editor.test.tsx](../../../shared/files/apps/web/src/elements/patch/Editor.test.tsx.md)
- [apps/web/src/elements/session/evidence.test.tsx](../../../shared/files/apps/web/src/elements/session/evidence.test.tsx.md)
- [apps/web/src/lib/capturedCode.test.ts](files/apps/web/src/lib/capturedCode.test.ts.md)
- [apps/web/src/lineage/LineageColumns.test.tsx](files/apps/web/src/lineage/LineageColumns.test.tsx.md)
- [apps/web/src/views/CommitReading.controls.test.tsx](files/apps/web/src/views/CommitReading.controls.test.tsx.md)
- [apps/web/src/views/LineageView.test.tsx](files/apps/web/src/views/LineageView.test.tsx.md)
- [apps/web/src/views/SessionLens.test.tsx](../../../shared/files/apps/web/src/views/SessionLens.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
