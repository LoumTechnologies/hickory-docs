<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Replay Recomputes Exact Lineage At Any Commit, And Names Its One Limit

For: Authors editing generated text and reviewers tracing a result to its source.

Given a `.md` document in a git repository, when a reader asks for its
lineage at an earlier commit — `hick lineage --at <commit>`, or the time
slider on the lineage view — then that commit's document is read out of git,
woven **without executing anything**, and its provenance reported exactly;
and when the old document does not parse with today's binary, then the answer
says *replay works back to the last grammar change* rather than surfacing a
parse error.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/replay-recomputes-lineage-at-a-commit.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
