<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Capture Records Values From Inside A Function, On Every Hit

For: Authors debugging generated code while navigating its document source.

Given `<hick:capture at="file.py:14" of="a, b" when="…" max="N" />` inside a
`<hick:exec>` cell, when the document runs, then each time that line is
reached the same `evaluate` request the interactive debugger issues is sent
for each expression, and the values are woven into the cell's output as a
table ordered by hit index — which `<hick:expect>` can then pin, making a
value *inside a function* something a document can assert about, re-derived on
every run.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/a-capture-records-values-from-inside-a-function.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
