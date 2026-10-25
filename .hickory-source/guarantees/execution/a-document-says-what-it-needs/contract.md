<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Document Says What It Needs

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a `<hick:container>` carrying `<hick:needs bin="…" />` declarations,
when the document runs, then every declared program is looked for **before
any cell executes**, and a document missing one stops immediately — naming
every missing program at once, with the line each was declared on and the
reason its author gave.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-document-says-what-it-needs.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
