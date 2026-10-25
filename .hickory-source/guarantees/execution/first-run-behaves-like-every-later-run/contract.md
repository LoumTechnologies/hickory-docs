<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Document's First Run Behaves Like Every Later Run

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a document that assembles a file with `hick:file` and then executes that
file from a volume, when it is run for the FIRST time — with nothing previously
committed — then the cell finds the file and the run succeeds, exactly as it
would on any later run.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/first-run-behaves-like-every-later-run.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
