<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Agent-Authored Bytes Always Report Session, Turn, And A Git-Derived Author — Even When The Session Cannot Be Read

For: Authors editing generated text and reviewers tracing a result to its source.

Given a byte of generated output whose provenance origin is
`SourceOrigin::Agent { session, turn }`, when a reader asks for its lineage,
then the report always names the session id and the turn, and names the author
by deriving it from `git` rather than from any stored field — and when the
session file is missing, unreadable, unparseable, or does not record that turn,
the report says so with the path it looked for and why it failed, instead of
erroring, panicking, or degrading the origin to `synthetic`.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/agent-lineage-degrades-without-a-session.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
