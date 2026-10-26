<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A File Is Ingested From The Tree

For: Authors editing executable documents and the files those documents produce.

Given a plain text file in the tree — not a document, not a file some
document already writes, not a binary — when a person right-clicks it,
then the menu offers **Make literate**, which writes a new `.md` document beside
the file owning its bytes exactly, and, when a document is focused,
**Ingest into <that document>**, which appends the file's bytes to it as a
block; either way the document it landed in opens. Both are the same
adoption `hick ingest --from file` performs, through `POST /api/adopt`,
and both are refused for anything that is not a plain text file, so the
verb never appears where it cannot be honoured.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-file-is-ingested-from-the-tree.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/shell/FolderTreePane.tsx](../../../mechanisms/apps/web/src/shell/FolderTreePane.tsx.md#output-9c0af3061e25)
- [apps/web/src/shell/treeMenu.ts](../../../mechanisms/apps/web/src/shell/treeMenu.ts.md#output-19605cc8cff0)
- [crates/hickory-cli/src/serve/refactor.rs](../../../mechanisms/crates/hickory-cli/src/serve/refactor.rs.md#output-3a011c557915)

### Tests and fixtures

- [apps/web/src/shell/FolderTreePane.test.tsx](../../../mechanisms/apps/web/src/shell/FolderTreePane.tsx.md#output-508e32e299fd)
- [apps/web/src/shell/treeMenu.test.ts](../../../mechanisms/apps/web/src/shell/treeMenu.ts.md#output-f48c486ac3c1)
- [crates/hickory-cli/tests/serve_files.rs](../../../mechanisms/crates/hickory-cli/tests/serve_files.rs.md#output-44963d50453a)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
