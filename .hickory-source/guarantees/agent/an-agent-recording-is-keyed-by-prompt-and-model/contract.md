<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Agent Recording Is Keyed By The Prompt And The Model

For: Authors using an agent to change documents while retaining review control.

Given a `<hick:agent>` cell whose answer is recorded in
`.hick-cache/transcripts/`, when that recording is looked up, then the key is a
hash of **the prompt and the model together**. Editing either retires the
recording; a retired recording is a missing baseline, never a stale answer
served as a current one.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/an-agent-recording-is-keyed-by-prompt-and-model.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
