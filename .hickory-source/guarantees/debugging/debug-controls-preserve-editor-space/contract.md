<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Choose a launch without replacing the editor being read

For: Authors debugging generated code while navigating its document source.

Open the launch picker while reading generated code. Choosing a target should not replace the editor or silently switch the code being debugged.

## What must happen

Debug controls preserve the mounted editor and its available reading space. Launch profiles use the current floating picker and exact displayed-source selection. Source identity is rechecked by the host; a stale or unavailable mapping remains unavailable. Ordinary language-service scope is independent of the selected debugger launch.

## Limits of the evidence

A controller test does not establish native visual layout across every viewport.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/debug-controls-preserve-editor-space.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Use the launch-profile continuation instead of the earlier global version-menu and replacement-editor wording.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Implementation

## Implementation review

### Implementation

- [apps/web/src/components/PlainFilePane.tsx](../../../mechanisms/apps/web/src/components/PlainFilePane.tsx.md#output-2f891a49445f)
- [apps/web/src/debug/DebugRail.tsx](../../agent/scoped-reading-surfaces-share-native-actions/implementation/apps/web/src/debug/DebugRail.tsx.md#output-3f2d1a1c0986)
- [apps/web/src/debug/DebugStrip.tsx](../../../mechanisms/apps/web/src/debug/DebugStrip.tsx.md#output-09306f4ec1a5)
- [apps/web/src/views/workspaceTabs.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-b5a48fd337ae)

### Tests and fixtures

- [apps/web/src/components/ComparisonTools.test.tsx](../../../mechanisms/apps/web/src/components/ComparisonTools.test.tsx.md#output-d1cfc2ed25eb)
- [apps/web/src/components/PlainFilePane.test.tsx](../../../mechanisms/apps/web/src/components/PlainFilePane.tsx.md#output-d923b2fdae48)
- [apps/web/src/debug/DebugRail.test.tsx](../../agent/scoped-reading-surfaces-share-native-actions/implementation/apps/web/src/debug/DebugRail.test.tsx.md#output-141b8a9dd63d)
- [apps/web/src/debug/DebugStrip.test.tsx](../../../mechanisms/apps/web/src/debug/DebugStrip.tsx.md#output-1063c352c210)
- [apps/web/src/debug/GeneratedDebugTools.test.tsx](../../../mechanisms/apps/web/src/debug/GeneratedDebugTools.test.tsx.md#output-7003cb507e09)
- [apps/web/src/debug/plainDebugHosts.test.tsx](../launch-profiles-choose-targets-within-the-viewed-project/contract.md#output-3b3668793103)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
