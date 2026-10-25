<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Published executable results identify reviewed inputs

For: Host developers embedding the document editor without a native execution service.

Given the explicit publication selection,
when CI generates public verification artifacts, then it copies only selected
source/dependencies/assets whose SHA-256 values match their recorded review.
It runs the existing native weave/test commands in an isolated temporary input
tree with an isolated home and no inherited provider credentials. It preserves
failed command details and still generates the static index and snapshot.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/embedding/publication-is-bound-to-reviewed-inputs.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
