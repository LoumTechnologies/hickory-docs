<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Workspace Tree Is A Lens

For: Authors editing executable documents and the files those documents produce.

Given files and other objects associated with folders, when the workspace tree
is shown, then each object is a node at its place; expanding and editing a node
asks its source through capabilities that node declared; the tree has no save
path or serialized editable form of its own; and a batch spanning independent
sources reports an outcome per node rather than claiming atomicity.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-workspace-tree-is-a-lens.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/lib/workspaceTree.ts](../../../mechanisms/apps/web/src/lib/workspaceTree.ts.md#output-59ed104225da)
- [apps/web/src/shell/FolderTreePane.tsx](../../../mechanisms/apps/web/src/shell/FolderTreePane.tsx.md#output-9c0af3061e25)
- [apps/web/src/shell/dired.ts](../the-tree-is-a-dired/contract.md#output-fabcac2ae554)
- [apps/web/src/shell/useDired.ts](../the-tree-is-a-dired/contract.md#output-b7b29d60436f)

### Tests and fixtures

- [apps/web/src/shell/FolderTreePane.test.tsx](../../../mechanisms/apps/web/src/shell/FolderTreePane.tsx.md#output-508e32e299fd)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
