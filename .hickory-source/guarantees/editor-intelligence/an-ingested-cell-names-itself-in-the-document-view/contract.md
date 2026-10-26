<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Ingested Cell Names Itself In The Document View

For: Authors using completion, diagnostics and refactoring inside literate source.

Given an exec cell that owns a `<hick:ingested>` child, when its card renders
in the Document view, then the card's status bar names the run — file
count, and its fingerprint and date on hover — the same fact the woven
markdown's "Ingested from …" caption already states for free.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/an-ingested-cell-names-itself-in-the-document-view.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/api/types.ts](../../../mechanisms/apps/web/src/api/types.ts.md#output-8a4c3188ff6c)
- [apps/web/src/components/CellPanel.tsx](../../../mechanisms/apps/web/src/components/CellPanel.tsx.md#output-c3c409b02e61)
- [apps/web/src/editor/rendered.ts](../../../mechanisms/apps/web/src/editor/rendered.ts.md#output-4f5444d21411)
- [crates/hick-literate/src/render.rs](../../../mechanisms/crates/hick-literate/src/render.rs.md#output-4a7f39f68eb8)

### Tests and fixtures

- [apps/web/src/components/CellPanel.test.tsx](../../../mechanisms/apps/web/src/components/CellPanel.tsx.md#output-85e341faa089)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
