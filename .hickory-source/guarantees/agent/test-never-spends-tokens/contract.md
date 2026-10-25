<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# `hick test` Never Spends The Reader's Tokens

For: Authors using an agent to change documents while retaining review control.

Given a document containing a `<hick:agent>` cell, when `hick test` runs it,
then **no model is called**, no matter what credentials the machine holds. The
cell is verified against its recording, or reported **unverifiable** (exit `2`)
with a message naming the cell, its prompt, and how to establish a baseline.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/test-never-spends-tokens.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
