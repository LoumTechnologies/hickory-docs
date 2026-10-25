<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Fence Becomes A Cell That Runs

For: Authors editing executable documents and the files those documents produce.

Given a `.md` document whose prose contains a closed fenced code block, when
its rail icon is clicked, then a panel shows the exact `<hick:exec>` that
would replace it — the container it will run in, and a body derived from the
fence's language — and confirming replaces exactly the fence, leaving every
byte around it untouched.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-fence-becomes-a-cell-that-runs.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
