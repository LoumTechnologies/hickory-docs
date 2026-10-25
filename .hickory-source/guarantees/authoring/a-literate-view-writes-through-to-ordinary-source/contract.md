<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A literate view writes through to ordinary source

For: Authors editing executable documents and the files those documents produce.

Given ordinary UTF-8 files in a repository, opening a source-backed literate view
or organizing its exact code slices changes no repository files. Every selected
file reconstructs byte-for-byte. A view's source does not become a document in
the repository index.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-literate-view-writes-through-to-ordinary-source.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
