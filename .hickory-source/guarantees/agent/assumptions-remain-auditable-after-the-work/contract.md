<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Assumptions remain auditable after the work

For: Authors using an agent to change documents while retaining review control.

Given a retained agent session, when an agent records assumptions, decisions or
guarantee reviews, then each entry retains its statement, reason, evidence
claimed, affected work, alternatives and falsifier. Guarantee reviews require
the originating requirement and a falsifier; exclusions, witnesses, challenges
and dependency snapshots are separate fields.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/assumptions-remain-auditable-after-the-work.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
