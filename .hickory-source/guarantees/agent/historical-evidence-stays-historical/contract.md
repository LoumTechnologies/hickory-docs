<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Historical evidence stays historical

For: Authors using an agent to change documents while retaining review control.

Given a new completed read receipt, when its session is inspected later, then
the audit retains the exact returned source passage, its original range and
whole-source SHA-256. The passage does not become today's file contents. A
separate current-source action opens today's file; current source is labeled
unchanged, changed or unavailable by a confined mechanical digest comparison.
Guarantee dependency snapshots use the same independent current check.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/historical-evidence-stays-historical.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
