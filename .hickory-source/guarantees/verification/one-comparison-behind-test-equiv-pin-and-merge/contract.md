<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# One Comparison Behind Test, Equiv, The Pin, And The Merge Check

For: Reviewers deciding what an output, recording or prediction actually establishes.

Given two sets of woven outputs — a document's against what is on disk
(`hick test`), one document's against another's (`hick equiv`), a document's
against its pinned baseline (the refactor pin in the app), a document with
its recordings inside it against the cache-backed weave (the ingest gate),
or a merged document against its parents (the merge driver, through
`hick test`) — when they are compared, then it is one function,
`hick_literate::equiv::compare_outputs`, and one report shape: per file,
*only in the first*, *only in the second*, or *content differs* with the
one sentence every comparison ends with — where the bytes first part company
and how the lengths compare (`describe_difference`), followed by the
line-by-line difference.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/one-comparison-behind-test-equiv-pin-and-merge.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
