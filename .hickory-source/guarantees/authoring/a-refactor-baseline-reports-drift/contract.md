<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Refactor Baseline Reports The Moment An Output Would Move

For: Authors editing executable documents and the files those documents produce.

Given a document whose woven outputs were pinned as a refactor baseline
(the toolbar's "Refactor", `POST /api/docs/:id/refactor/begin`), when the
document is restructured — blocks split, prose interleaved, sections
reordered — then the live verdict (`GET …/refactor/status`, the toolbar
badge) says either that every output still matches the baseline or exactly
which outputs differ and how (changed, added, removed); and checking never
executes a cell.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-refactor-baseline-reports-drift.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
