<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Anchored Terminal Writes What You Typed

For: Authors running interactive tools alongside documents without losing session state.

Given a terminal anchored to a container in a document, when you type a
command into it, then that line becomes part of that container's cell — and
the cell is always **a prefix of the session that reproduces**:

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/terminal/an-anchored-terminal-writes-what-you-typed.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
