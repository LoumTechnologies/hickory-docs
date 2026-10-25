<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Generated Text Refuses An Edit, And Says So Before You Type

For: Authors editing executable documents and the files those documents produce.

Given `hick up` running over a folder, when a generated output file contains
no byte that maps back to a document, then that file is marked read-only on
disk for as long as the loop runs; and when an edit is forced into generated
text anyway, then the file is restored to the woven bytes, the document is
left untouched, and the refusal names the line, the reason, and the file to
edit instead.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-generated-file-refuses-an-edit.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
