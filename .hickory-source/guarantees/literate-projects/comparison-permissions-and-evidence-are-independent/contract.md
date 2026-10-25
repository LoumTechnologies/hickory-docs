<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Comparison edit permissions do not establish replay evidence

For: Maintainers reorganizing source while preserving the complete generated program.

Given an N-source project comparison, when Old and New endpoints are selected,
then their identities, writable destinations and edit policy are separate from
replay verification. All four policies (neither, old, new, both) are supported
by the model. An immutable source without a writable fork cannot become editable.
An accepted endpoint edit clears equality evidence and leaves the opposite
endpoint unchanged. Destination trees must still equal the captured identity
before a host applies reviewed changes.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/literate-projects/comparison-permissions-and-evidence-are-independent.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
