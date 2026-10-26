<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Plain File Opens, Edits, And Never Silently Loses A Byte

For: Authors editing executable documents and the files those documents produce.

Given a folder open in the desktop app, when the tree shows a text file that
is neither a `.md` document nor a woven output of an open document — a
README, a workflow, a justfile — then clicking it opens an editable pane, and
edits save back to the same path on disk; and when the file was rewritten on
disk between the load and the save, then the save is refused and the person
chooses — reload the disk copy or overwrite it — rather than either version
being lost silently.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-plain-file-opens-and-saves.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/components/PlainFilePane.tsx](../../../mechanisms/apps/web/src/components/PlainFilePane.tsx.md#output-2f891a49445f)
- [apps/web/src/lib/plainFileSave.ts](../../../mechanisms/apps/web/src/lib/plainFileSave.ts.md#output-07b4144df7a0)
- [apps/web/src/shell/FolderTreePane.tsx](../../../mechanisms/apps/web/src/shell/FolderTreePane.tsx.md#output-9c0af3061e25)
- [apps/web/src/views/workspaceState.ts](../../../mechanisms/apps/web/src/views/workspaceState.ts.md#output-efd800f9ed5a)
- [crates/hickory-cli/src/serve/mod.rs](../../../mechanisms/crates/hickory-cli/src/serve/mod.rs.md#output-09ea558208cd)
- [crates/hickory-cli/src/serve/plain_file.rs](../../../mechanisms/crates/hickory-cli/src/serve/plain_file.rs.md#output-128423f88fa6)

### Tests and fixtures

- [apps/web/src/lib/plainFileSave.test.ts](../../../mechanisms/apps/web/src/lib/plainFileSave.ts.md#output-94afcad10b27)
- [apps/web/src/shell/FolderTreePane.test.tsx](../../../mechanisms/apps/web/src/shell/FolderTreePane.tsx.md#output-508e32e299fd)
- [apps/web/src/views/workspaceState.test.ts](../../../mechanisms/apps/web/src/views/workspaceState.ts.md#output-14d3dbc939ff)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
