<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A project environment offers its own sync

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a host project identified as uv, npm or pnpm by its manifest,
configuration or lockfile, when Hickory checks its environment, then it
reports a typed finding and offers only a repair the selected provider can
plan. An ambiguous manager offers an explicit choice, saved in the user's
workspace state, before it offers installation.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/a-project-environment-offers-its-own-sync.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
