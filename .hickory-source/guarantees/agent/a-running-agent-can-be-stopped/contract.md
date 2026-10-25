<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A running agent can be stopped

For: Authors using an agent to change documents while retaining review control.

Given a hosted Python turn, Stop cancels in-flight inference or review and
terminates the confined process group. A pending proposal is not approved by
cancellation. The session records the failure/cancellation, the turn has status
stopped and no successful answer, and the existing WebSocket terminal event
restores the Send control. Standalone Python also accepts an injectable cancel
callback and its development CLI handles Ctrl-C.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/a-running-agent-can-be-stopped.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
