<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Runtime history keeps runs and invocation occurrences separate

For: Authors running documents on their own machine and reviewers assessing their side effects.

The initial display policy selects the latest captured run, with explicit
history, pin/unpin and separate comparison. A newer failed/partial run exposes
its actual observations rather than retaining an older green result as current.
Historical selections remain explicit. Editing source preserves observations
but marks them stale and removes their live-editor decorations.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/runtime-history-keeps-runs-and-occurrences-separate.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
