<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Output Path Declared Twice Warns Before It Runs

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a document that has already ingested a path — a `hick:ingested`
block's child `hick:file` records it — and a separate, top-level
`<hick:file path="P">` declares that exact same path again, when the
document is run, then a warning names both declarations and their source
lines before any cell executes — never a silent clobber or a competing
declaration nobody notices disagrees with the other.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/an-output-path-declared-twice-warns-before-it-runs.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
