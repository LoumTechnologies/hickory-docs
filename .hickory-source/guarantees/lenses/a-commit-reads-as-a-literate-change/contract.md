<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A commit reads as a literate change

For: Authors reviewing history and choosing where an edit should land.

Given a commit in History or Story, **Read commit** opens a read-only document
reading with its complete message above its changed files. Markdown reads as
prose; other text files read as code. Additions and removals use the same
`comparisonField` as literate Git comparisons and agent proposals. Removed
passages render through the literate formatting layer and are expanded in a
read-only comparison. They never enter the current buffer or its saved bytes.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lenses/a-commit-reads-as-a-literate-change.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
