<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Word Navigation Is The Person's Choice: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/OutputEditorPane.tsx](../../../shared/files/apps/web/src/components/OutputEditorPane.tsx.md)
- [apps/web/src/components/PlainFilePane.tsx](../../../shared/files/apps/web/src/components/PlainFilePane.tsx.md)
- [apps/web/src/editor/DocumentEditor.tsx](../../../shared/files/apps/web/src/editor/DocumentEditor.tsx.md)
- [apps/web/src/editor/wordMotion.ts](files/apps/web/src/editor/wordMotion.ts.md)
- [apps/web/src/lib/channelWidth.ts](../../../support/files/apps/web/src/lib/channelWidth.ts.md)
- [apps/web/src/lib/ribbonStyle.ts](../../../support/files/apps/web/src/lib/ribbonStyle.ts.md)
- [apps/web/src/lib/tabStyle.ts](../../../support/files/apps/web/src/lib/tabStyle.ts.md)
- [apps/web/src/lib/wordMotion.ts](files/apps/web/src/lib/wordMotion.ts.md)
- [apps/web/src/views/SettingsView.tsx](../../../shared/files/apps/web/src/views/SettingsView.tsx.md)

## Tests and fixtures

- [apps/web/src/editor/wordMotion.test.ts](files/apps/web/src/editor/wordMotion.test.ts.md)
- [apps/web/src/lib/wordMotion.test.ts](files/apps/web/src/lib/wordMotion.test.ts.md)
- [apps/web/src/views/SettingsView.test.tsx](../../../shared/files/apps/web/src/views/SettingsView.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
