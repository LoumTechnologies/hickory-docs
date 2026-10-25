<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Re-Preparation After An Agent Edit Terminates, Under A Declared Bound

For: Authors using an agent to change documents while retaining review control.

Given a document whose `<hick:agent>` cell edits that document's own source,
when the pipeline re-prepares the document so the rest of the pass runs against
what the agent wrote, then re-preparation reaches a **declared fixed point**
under a **declared bound**, and a document with no fixed point **fails** rather
than looping.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/re-preparation-terminates.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
