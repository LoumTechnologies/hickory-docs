<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A view declares where edits land

For: Authors reviewing history and choosing where an edit should land.

Given a document, plain or generated file, literate view, change review,
commit reading, conversation history, worktree comparison, or history story,
when the working surface is shown, then its identity, editability, and write
destination appear together. Historical comparisons disable writes. Generated
regions retain their existing protection; declaring an editable surface does
not make every byte editable or grant a new engine capability.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lenses/a-view-declares-where-edits-land.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
