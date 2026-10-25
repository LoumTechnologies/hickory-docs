<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Session Appears Where It Is Working

For: Authors running interactive tools alongside documents without losing session state.

Given terminal sessions and an open folder, when the workspace tree is shown,
then each session appears as a visible child node of the directory it is working
in; clicking that node shows the session's terminal; a collapsed directory says how many sessions
are hidden inside it; and a session working outside the folder does not appear
at all.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/terminal/a-session-appears-where-it-is-working.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
