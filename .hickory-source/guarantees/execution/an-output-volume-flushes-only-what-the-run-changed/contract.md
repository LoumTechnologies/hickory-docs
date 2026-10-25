<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Output Volume Flushes Only What The Run Changed

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a volume declared with an `output=` path and seeded from a directory,
when the run finishes and the volume is flushed to disk, then only the files
a cell created or changed since the volume was seeded are written; a file
whose bytes are identical to its seeded bytes is left alone.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/an-output-volume-flushes-only-what-the-run-changed.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
