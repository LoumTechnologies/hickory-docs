<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Derived Diagram Keeps Your Layout

For: Authors editing executable documents and the files those documents produce.

Given a `<hick:diagram renderer="graph">` whose body takes its topology
through a `<hick:paste>` (`{"topology": <hick:paste select="#…"/>, "layout":
{…}}`), when the reader arranges it in the graph editor, then commits rewrite
**only** the `layout` object and write the paste tag back byte-for-byte —
adding or removing nodes and edges is refused, and the panel says the topology
is derived — and when the fragment is regenerated, nodes whose ids survive
keep their recorded places, new ids appear beside the layout rather than over
it, and layout entries for departed ids are dropped on the next commit.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-derived-diagram-keeps-your-layout.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
