<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Agent Can Author A Diagram

For: Authors using an agent to change documents while retaining review control.

Given the built-in agent editing a document, when a diagram is the right form
for what it has to say, then the document language its system prompt teaches
includes `<hick:diagram>` — with the discipline attached: name the proving
cell in `asserts` or be a deliberate sketch; for `renderer="graph"`, emit
semantic node ids, omit `layout` (arranging is the person's half; auto-layout
fills in), and paste an existing topology fragment rather than restating it.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/the-agent-can-author-a-diagram.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
