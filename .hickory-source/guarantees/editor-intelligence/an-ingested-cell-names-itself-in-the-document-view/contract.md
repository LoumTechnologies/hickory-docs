<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Ingested Cell Names Itself In The Document View

For: Authors using completion, diagnostics and refactoring inside literate source.

Given an exec cell that owns a `<hick:ingested>` child, when its card renders
in the Document view, then the card's status bar names the run — file
count, and its fingerprint and date on hover — the same fact the woven
markdown's "Ingested from …" caption already states for free.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/an-ingested-cell-names-itself-in-the-document-view.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
