<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Empty Response Never Kills a Session

For: Authors using an agent to change documents while retaining review control.

Given a model that returns an empty or whitespace-only response, when the
agent loop re-prompts it, then the correction turn succeeds: the blank turn is
recorded as `(no output)` rather than replayed verbatim, and no request built
by either client ever carries an empty content block.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/an-empty-response-never-kills-a-session.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
