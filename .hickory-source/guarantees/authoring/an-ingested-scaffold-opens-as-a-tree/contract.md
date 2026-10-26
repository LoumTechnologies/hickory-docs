<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Ingested Scaffold Opens As A Tree

For: Authors editing executable documents and the files those documents produce.

Given a document holding a `<hick:ingested>` block — the forty files
`dotnet new webapi` wrote, sitting between two paragraphs somebody typed —
when it opens in the Document view, then **every ingested file starts
folded**, so what is on screen is one line per file: that file's own
`<hick:file path="…">` tag, the first line of its content, and a placeholder
saying how many lines it holds. The block itself folds to `8 files`, a
directory of two or more files folds to `Controllers/ · 4 files, 96 lines`,
and none of it is a second rendering: the fold headers are the document's own
bytes and the document is unchanged.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/an-ingested-scaffold-opens-as-a-tree.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/editor/DocumentEditor.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.tsx.md#output-aab82870e517)
- [apps/web/src/editor/folding.ts](../../../mechanisms/apps/web/src/editor/folding.ts.md#output-eee9d1963829)
- [apps/web/src/styles.css](../../../mechanisms/apps/web/src/styles.css.md#output-8695905e2358)

### Tests and fixtures

- [apps/web/src/editor/DocumentEditor.test.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.test.tsx.md#output-71bbcfe029bb)
- [apps/web/src/editor/folding.test.ts](../../../mechanisms/apps/web/src/editor/folding.ts.md#output-92a9cd526370)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
