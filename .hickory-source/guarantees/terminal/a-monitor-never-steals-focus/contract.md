<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Monitor Never Steals Focus

For: Authors running interactive tools alongside documents without losing session state.

Given a session opened with `monitor: true` — a dev server, a test watcher, a
log tail — when it prints, restarts, or dies, then it never enters the
attention queue, never opens a tab by itself, and never takes focus from
whatever pane is in front. It appears in the dock, where it shows its name and
its last line, and the dock turns amber when any monitor has stopped running.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/terminal/a-monitor-never-steals-focus.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
