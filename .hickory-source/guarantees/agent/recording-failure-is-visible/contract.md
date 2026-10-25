<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Recording failure is visible

For: Authors using an agent to change documents while retaining review control.

Given a Hickory-handled agent action, when its intent cannot be recorded,
then that action is refused before execution. When recording an already executed
result fails, the caller receives an explicit incomplete-record error and must
inspect actual files; the system does not claim rollback.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/recording-failure-is-visible.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
