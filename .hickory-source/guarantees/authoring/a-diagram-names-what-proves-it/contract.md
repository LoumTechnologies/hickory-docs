<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Diagram Names What Proves It

For: Authors editing executable documents and the files those documents produce.

Given a `<hick:diagram>` in a document, when the document is run or woven,
then the diagram weaves into a fenced block tagged with its `renderer` — so
the woven markdown renders wherever markdown renders — and its `asserts`
attribute is checked against the document: a reference to an id no tag carries
is a warning naming the missing id, and a diagram with no `asserts` at all is
a warning saying that nothing will fail when the picture stops being true.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-diagram-names-what-proves-it.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
