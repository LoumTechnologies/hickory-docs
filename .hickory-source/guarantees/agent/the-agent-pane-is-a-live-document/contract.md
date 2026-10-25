<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Agent pane is a live document

For: Authors using an agent to change documents while retaining review control.

Given an agent conversation, the Agent pane uses `DocumentEditor`, the same
literate editor as an ordinary note. Recorded turns on the selected branch and
the running turn appear in one buffer, followed by **Your response**, an editable
region. Recorded and streaming content is protected against keyboard edits,
paste, undo, and programmatic widget edits. Only the conversation owner can
replace that prefix. A streaming update preserves the user's response draft,
including its line breaks.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/the-agent-pane-is-a-live-document.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
