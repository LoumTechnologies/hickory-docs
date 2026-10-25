<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Command The App Runs On Your Behalf Is Watched In A Terminal

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a button, menu item or gutter mark in the app whose job is to run an
**external command line program** on this machine — `dotnet new`, `cargo
build`, `pytest`, `dotnet test`, `go test` — when a person triggers it, then
that program runs in a terminal session they can read, scroll and keep, and
its stdout and stderr are the program's own, byte for byte. The app reports
*around* the output; it never reports *instead of* it.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-command-the-app-runs-is-watched-in-a-terminal.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
