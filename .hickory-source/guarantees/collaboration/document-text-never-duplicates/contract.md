<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Document's Text Is Never Duplicated Or Silently Rewritten By Sync

For: Engineers using local sessions or pairing their own machines.

Given a document open in one or more editors, when the server rebuilds its
room, reconciles the CRDT against a source rewritten out of band (an
`/outputs/edit` resolved through provenance), or receives client updates, then
the room's text equals the intended text exactly — never the intended text
concatenated with a rival copy of itself, and never spliced at an offset that
lands mid-character. A document that nevertheless exceeds
`MAX_DOC_BYTES` (4 MB) is refused rather than persisted.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/document-text-never-duplicates.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/editor/DocumentEditor.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.tsx.md#output-aab82870e517)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve.

</storycode:doc>
