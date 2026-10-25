<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Claim Says Who Is Asserting It, And Never Looks Verified

For: Authors editing executable documents and the files those documents produce.

Given a `<hick:claim>` in a document, when the document is woven, then it
emits an attribution naming who made the claim, on what standing, and about
what scope — and the claim's own prose passes through unchanged. When the
marking is incomplete or uses a standing the document does not know, then a
**warning** names the problem, the value, and the vocabulary; never an error.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-claim-says-who-is-asserting-it.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
