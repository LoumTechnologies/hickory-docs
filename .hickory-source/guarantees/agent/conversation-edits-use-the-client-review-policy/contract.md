<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Conversation edits use the client's review policy

For: Authors using an agent to change documents while retaining review control.

Given a built-in or ACP conversation, when the user chooses **Review** or **Auto-accept**
under **Document edits**, then that choice belongs to this conversation and is
restored when its saved conversation is reopened. New conversations start with
Review. Changing it during a turn is refused.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
