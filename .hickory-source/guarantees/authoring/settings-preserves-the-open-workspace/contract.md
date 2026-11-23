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

- apps/web/src/App.tsx
- apps/web/src/lib/workspaceVisibility.ts
- apps/web/src/shell/CommandBar.tsx
- apps/web/src/shell/ShellView.tsx
- apps/web/src/views/SettingsView.tsx
- apps/web/src/views/WorkspaceView.tsx
- apps/web/src/views/useWorkspaceAppearance.ts

### Tests and fixtures

- apps/web/e2e/workflow-polish.spec.ts
- apps/web/src/App.test.tsx
- apps/web/src/views/useWorkspaceAppearance.test.tsx

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
