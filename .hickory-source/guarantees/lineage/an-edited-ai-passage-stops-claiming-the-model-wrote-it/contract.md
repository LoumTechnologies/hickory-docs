<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Edited AI Passage Stops Claiming The Model Wrote It

For: Authors editing generated text and reviewers tracing a result to its source.

Given a `hick:transform` carrying `wrote=` — a fingerprint of the passage as
the model produced it — when the passage's bytes are no longer those bytes,
then the weave says *written by a model, and edited by hand since*, and
`hick test` reports it, naming the model.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/an-edited-ai-passage-stops-claiming-the-model-wrote-it.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
