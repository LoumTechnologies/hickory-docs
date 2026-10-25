<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Inserting An Element Writes Hick You Could Have Typed Yourself

For: Authors editing executable documents and the files those documents produce.

Given a document open in the app, when a person chooses an element from the
Insert menu and fills in its attributes, then the exact bytes the insert will
write are shown before it happens, and what lands in the buffer is ordinary
`.md` source — correctly quoted, placed where that element belongs, with the
body left selected so the next keystroke replaces it — indistinguishable from
the same element typed by hand, and undoable with one Ctrl+Z.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/inserting-an-element-writes-hick-you-could-have-typed.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
