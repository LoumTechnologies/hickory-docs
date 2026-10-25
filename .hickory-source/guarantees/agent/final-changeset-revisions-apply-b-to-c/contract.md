<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A final changeset revision applies B to C

For: Authors using an agent to change documents while retaining review control.

Given a finalized agent artifact constructing accepted B from captured A,
when the user chooses Revise New, then the accepted closure and snapshots remain
immutable. A separately labeled authored draft permits New code edits with Old
fixed. Editing uses ordinary patch regeneration with captured A, re-evaluates the
entire supplied closure, and writes no destination. A downstream-breaking edit
remains unchecked and disables Apply. Discard removes the draft without writes.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/final-changeset-revisions-apply-b-to-c.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
