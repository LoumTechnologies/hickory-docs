<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Unsent responses survive restart

For: Authors using an agent to change documents while retaining review control.

Given text in **Your response**, editing saves recovery to personal workspace
storage. Closing the Agent pane retains it. Reopening the pane or restarting the
app restores the response to its workspace, window slot, editor subject and
conversation thread. Recovery never creates a repository file, names an untitled
editor, or writes response text into an immutable recorded session.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/unsent-responses-survive-restart.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
