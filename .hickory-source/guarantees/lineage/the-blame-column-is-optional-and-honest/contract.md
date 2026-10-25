<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Blame Column Is Optional And Honest

For: Authors editing generated text and reviewers tracing a result to its source.

Given any editor in the app, when the blame column is off — which it is by
default — then nothing is asked of git at all; and when it is turned on, then
every line shows who last committed it, uncommitted lines say so rather than
borrowing an author, and a folder with no history shows an empty column rather
than failing to open a file.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/the-blame-column-is-optional-and-honest.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
