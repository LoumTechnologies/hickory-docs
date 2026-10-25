<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Table Is A Dataset And A Paragraph

For: Authors editing executable documents and the files those documents produce.

Given a `<hick:table>` in a document, when the document is woven, then the
reader sees a markdown table; and when the tag carries a `path`, then those
same rows are written there as CSV, byte for byte, for anything that is not
this app to read.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />
- [Requirements 3](requirements/03.md)
<book:include file="requirements/03.md" />
- [Requirements 4](requirements/04.md)
<book:include file="requirements/04.md" />
- [Requirements 5](requirements/05.md)
<book:include file="requirements/05.md" />
- [Requirements 6](requirements/06.md)
<book:include file="requirements/06.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-table-is-a-dataset-and-a-paragraph.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
