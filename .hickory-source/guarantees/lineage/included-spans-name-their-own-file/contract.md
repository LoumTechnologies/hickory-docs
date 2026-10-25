<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Included Span Names Its Own File, And An Edit Through It Lands There

For: Authors editing generated text and reviewers tracing a result to its source.

Given a document that splices text from other files (`<hick:include>`,
`<hick:upstream>`), when provenance is built for any output those spans feed,
then each entry's `doc_path` names the file whose bytes the span offsets
actually index — the included file, canonically — and when an edit in a
generated output maps onto such a span, then the edit is applied to that
file, byte-exactly, while the including document is untouched.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/included-spans-name-their-own-file.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
