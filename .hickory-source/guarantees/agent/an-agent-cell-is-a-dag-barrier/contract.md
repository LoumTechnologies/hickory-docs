<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Agent Cell Is A DAG Vertex, And Its Edges Are A Barrier

For: Authors using an agent to change documents while retaining review control.

Given a document containing `<hick:agent>`, when the pipeline builds the
information-flow DAG, then the cell is a **vertex** — scheduled by the same
topological loop as `<hick:exec>`, covered by the same freeze, recording,
`never_run`, and `hick:expect` machinery — and its edges are a **barrier**:
every cell declared before it precedes it, and every cell declared after it
follows it.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/an-agent-cell-is-a-dag-barrier.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
