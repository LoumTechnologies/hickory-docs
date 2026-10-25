<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
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
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-file-is-ingested-from-the-tree.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
