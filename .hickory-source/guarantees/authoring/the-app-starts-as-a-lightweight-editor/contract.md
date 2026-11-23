<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The App Starts As A Lightweight Editor

For: Authors editing executable documents and the files those documents produce.

Given the app is launched without an explicit file or folder, when its window
opens, then no folder is selected, even if an older launch remembered a folder.
It opens the editable introduction as an unsaved Untitled document, supports
Agent and Save, and shows no repository setup banner. Recent files and folders
remain available for explicit opening. File → New Window uses the same editor
surface. Closing a folderless window checks unsaved work through the same close
gate as a folder window.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-app-starts-as-a-lightweight-editor.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- apps/desktop/src-tauri/src/lib.rs
- apps/web/src/App.tsx
- apps/web/src/lib/focusWhenShown.ts
- apps/web/src/lib/newDoc.ts
- apps/web/src/views/WorkspaceView.tsx
- apps/web/src/views/workspaceState.ts
- apps/web/src/views/workspaceTabs.tsx

### Tests and fixtures

- apps/desktop/src-tauri/tests/serves_one_origin.rs
- apps/web/e2e/startup.spec.ts
- apps/web/src/App.test.tsx
- apps/web/src/views/workspaceState.test.ts

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
