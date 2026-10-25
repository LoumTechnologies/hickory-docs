<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Ingested Bytes Are Editable And Never Report As Yours

For: Authors editing generated text and reviewers tracing a result to its source.

Given generated output whose bytes came from an `<hick:ingested>` block, when
a reader asks for its lineage, then every such byte reports
`kind: "ingested"` carrying the document span AND the run fingerprint — never
`literal`, which would say a person here typed a scaffolder's forty files, and
never `synthetic`, which would make the bytes uneditable.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/ingested-bytes-are-not-yours.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
