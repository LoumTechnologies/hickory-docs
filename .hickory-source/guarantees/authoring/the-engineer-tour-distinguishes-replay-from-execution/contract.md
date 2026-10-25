<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The engineer tour distinguishes replay from execution

For: Authors editing executable documents and the files those documents produce.

Given the AI engineer tour on the homepage or in the application, the same
scenario identifies meeting evidence, a requirement, a recorded agent mistake,
a precise objection, a failing counterexample and a corrected implementation.
Agent proposals are labeled recorded examples. Browser execution uses the product
browser debugger. The counterexample step requires observed failure and the
correction step requires observed success before Continue is enabled.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
