<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A GitHub Issue Belongs To Its Associated Folder

For: Engineers connecting repository issues, reviews and locally installed tools.

Given a person associates `owner/repository#number` (or its GitHub URL) from a
folder's context menu, when the workspace tree is read here or from another
clone, then the issue is a child of that folder. The committed-capable
`.hick-workspace.json` record contains only provider identity, issue number,
and root-relative folder—never a token or copied provider response.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/integrations/a-github-issue-belongs-to-its-associated-folder.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/shell/FolderTreePane.tsx](../../../mechanisms/apps/web/src/shell/FolderTreePane.tsx.md#output-9c0af3061e25)
- [apps/web/src/shell/GithubTreeNodes.tsx](../../../mechanisms/apps/web/src/shell/GithubTreeNodes.tsx.md#output-8c9096f3895b)
- [apps/web/src/shell/treeMenu.ts](../../../mechanisms/apps/web/src/shell/treeMenu.ts.md#output-19605cc8cff0)
- [crates/hickory-cli/src/serve/github.rs](../../../mechanisms/crates/hickory-cli/src/serve/github.rs.md#output-1de81b976483)

### Tests and fixtures

- [apps/web/src/shell/FolderTreePane.test.tsx](../../../mechanisms/apps/web/src/shell/FolderTreePane.tsx.md#output-508e32e299fd)
- [apps/web/src/shell/GithubTreeNodes.test.tsx](../a-review-belongs-to-the-worktree-for-its-head/contract.md#output-7956bbacfb50)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
