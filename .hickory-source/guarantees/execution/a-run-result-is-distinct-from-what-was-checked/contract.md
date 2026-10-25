<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A run result is distinct from what was checked

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a cell, diagram, or source-backed AI explanation, when status is
rendered, then successful execution does not alone claim verified output or
semantic correctness. Matching expected stdout is reported separately. Diagrams
report passed, failed, or not-yet-verified assertions; stale or absent execution
evidence cannot report passing assertions. Stale explanations identify source
change separately from execution verdicts. The action rail distinguishes stale
recordings from cells that have never run.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-run-result-is-distinct-from-what-was-checked.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
