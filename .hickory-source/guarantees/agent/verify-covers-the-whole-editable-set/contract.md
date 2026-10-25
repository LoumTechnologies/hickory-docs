<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Verify Covers the Whole Editable Set

For: Authors using an agent to change documents while retaining review control.

Given an agent session whose primary document declares `hick:upstream` edges,
when the agent runs `verify`, then every document in that closure is executed
and its output files are written **in its own directory** — and the result says
how many upstream documents were re-woven. A `PASS` means the whole chain is
consistent, not just the document in front of the agent.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/verify-covers-the-whole-editable-set.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
