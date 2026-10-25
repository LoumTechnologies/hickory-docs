<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Measure Says What It Measures

For: Authors editing executable documents and the files those documents produce.

Given the ruler above a document editor, when it is measuring prose then it
names its unit (`chars/line`), shades the range that is comfortable to read
(45–75), and says in its tooltip that the number is an average of a
proportional face rather than a count of columns. Given a caret inside a
table, the same strip stops measuring prose entirely and names the table's
columns instead.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-measure-says-what-it-measures.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
