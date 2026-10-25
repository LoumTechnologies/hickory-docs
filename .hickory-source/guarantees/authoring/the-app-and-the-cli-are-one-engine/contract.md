<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Desktop App And The CLI Are One Engine

For: Authors editing executable documents and the files those documents produce.

Given the desktop app open on a folder, when it weaves a document, computes
lineage, or carries an edit from a generated file back into its source, then it
runs the same code `hick up` runs — not a second implementation of it — and it
holds the same directory lock, so one folder is never open in two writers at
once.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-app-and-the-cli-are-one-engine.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
