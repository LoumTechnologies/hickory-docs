<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Settings preserves the open workspace

For: Authors editing executable documents and the files those documents produce.

Given open documents, generated files, unnamed drafts or an agent conversation,
when Settings is opened and closed, then the same workspace and editor instances
remain mounted. Draft bytes, selections, undo history and open tabs survive the
round trip. Back and Escape return to the route from which Settings was opened,
without replaying the startup introduction or opening another unnamed buffer.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/settings-preserves-the-open-workspace.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/App.tsx](../../../mechanisms/apps/web/src/App.tsx.md#output-25cc6f821b46)
- [apps/web/src/lib/workspaceVisibility.ts](../../../mechanisms/apps/web/src/lib/workspaceVisibility.ts.md#output-4350d48f61e3)
- [apps/web/src/shell/CommandBar.tsx](../the-editor-keeps-its-space/contract.md#output-9a0b2ebc41a8)
- [apps/web/src/shell/ShellView.tsx](../../../mechanisms/apps/web/src/shell/ShellView.tsx.md#output-e5f8a7657f14)
- [apps/web/src/views/SettingsView.tsx](../../../mechanisms/apps/web/src/views/SettingsView.tsx.md#output-38894484eb8f)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)
- [apps/web/src/views/useWorkspaceAppearance.ts](../../../mechanisms/apps/web/src/views/useWorkspaceAppearance.ts.md#output-14288d7adf50)

### Tests and fixtures

- [apps/web/e2e/workflow-polish.spec.ts](../../../mechanisms/apps/web/e2e/workflow-polish.spec.ts.md#output-2519c064887c)
- [apps/web/src/App.test.tsx](../../../mechanisms/apps/web/src/App.tsx.md#output-65ac24c24624)
- [apps/web/src/views/useWorkspaceAppearance.test.tsx](../../../mechanisms/apps/web/src/views/useWorkspaceAppearance.ts.md#output-3606526b11a4)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
