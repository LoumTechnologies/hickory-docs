<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Reject a patch that does not fit its exact input

For: Reviewers applying an agent explanation or maintaining a patch-based construction.

A patch expecting old text must fail if that text has changed. It must leave the input tree intact, including when a later hunk fails.

## What must happen

Given a virtual input tree and an explicitly declared UTF-8 unified patch, every hunk applies at its stated offsets with exact context. Invalid counts, overlaps, incorrect context, path collisions, unsupported headers, and incomplete deletions reject the whole act. The input tree is unchanged. BOM, CRLF, Unicode and final-newline differences remain data. Untouched bytes retain their origins; inserted bytes refer to the patch payload. Ordinary diff fences are display text.


## Limits of the evidence

Binary, Git extended-header and mode-only patch operations are unsupported. Explicit entries can construct binary content and metadata.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/literate-projects/replay-is-exact-and-atomic.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: State atomicity for the in-memory patch act; do not imply a filesystem transaction across all generated files.

Implementation review: partially verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
