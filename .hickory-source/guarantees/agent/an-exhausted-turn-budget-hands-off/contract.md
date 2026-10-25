<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An exhausted Python iteration budget retains incomplete work

For: Authors using an agent to change documents while retaining review control.

Given a Python run that spends its configured iteration budget without current
accepted completion, the Python graph stops and reports an error. Hickory keeps
actual applied writes, proposals, model calls and tool evidence in the session.
It does not mark the unverified work successful or make a hidden Rust handoff
model call. A later turn can use the retained session to continue deliberately.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/an-exhausted-turn-budget-hands-off.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
