<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Re-Emission Is Shown Before It Is Done, And Refuses To Rewrite What Is Published

For: Engineers using local sessions or pairing their own machines.

Given a folder of stage documents in a git repository, when `hick emit` runs,
then it reports the commits a re-emission **would** produce — one stage, one
commit — and emits nothing; and when producing them would rewrite a commit
below the publication floor, then it refuses and names it.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/emission-shows-what-it-would-do.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
