<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Sandboxed Cell Cannot Reach Past Its Workdir

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given `HICKORY_EXECUTOR=sandbox`, when a document's cells run, then each cell
may write only its own container workdir and a private `/tmp`; it sees an
empty home directory rather than yours; it cannot read another container's
workdir; and it has no network unless the document declared one with
`<hick:allow>`.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />
- [Requirements 3](requirements/03.md)
<book:include file="requirements/03.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-sandboxed-cell-cannot-reach-past-its-workdir.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
