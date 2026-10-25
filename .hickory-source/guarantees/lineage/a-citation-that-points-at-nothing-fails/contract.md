<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Citation That Points At Nothing Fails

For: Authors editing generated text and reviewers tracing a result to its source.

Given a document with `cites="…"`, when `hick test` runs, then every selector
in that list is resolved **one at a time**, and any selector matching nothing
fails the run — `EXPECTATION FAILED`, exit 3 — naming the document, the line,
and the selector. `hick cites` marks the same selector in its listing.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/a-citation-that-points-at-nothing-fails.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
