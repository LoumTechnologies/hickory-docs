<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Private edit proposals open review before writing

For: Reviewers deciding what an output, recording or prediction actually establishes.

An advertised private edit_doc call previews the captured source and waits for
independent host review before applying a source-bound edit. A prose proposal or
completion summary does not open that queue. The agent cannot supply its own
approval. The private catalogue advertises the supported whole="true" option,
latest whole-surface snapshot and complete replacement input, and identifies
run/after anchors as SHA-256 line hashes rather than source text. Invalid/stale
preview recovery exposes only fixed source/schema guidance, never raw host errors
or held-out target diagnostics. Existing exact-proposal freshness checks remain.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/private-proposals-wait-for-review.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
