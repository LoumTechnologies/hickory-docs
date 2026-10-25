<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
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
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/document-text-never-duplicates.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
