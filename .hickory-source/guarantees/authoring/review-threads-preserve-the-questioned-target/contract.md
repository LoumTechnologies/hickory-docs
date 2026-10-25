<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Review threads preserve the questioned target

For: Authors editing executable documents and the files those documents produce.

Given selected text in a saved-path editor or a drawn provenance connection,
when the user opens a review and saves a comment, then its target retains the
quoted text, source SHA-256, path and line range, plus the relationship descriptor
when applicable. Replies cannot replace that target. Comments, resolutions and
reopenings append local workspace events; earlier events remain. Resolution is
a user declaration, separate from executed verification. Changed or unavailable
saved source is shown rather than silently relocating the original quotation.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/review-threads-preserve-the-questioned-target.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
