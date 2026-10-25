<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Launch profiles choose targets within the viewed project

For: Authors debugging generated code while navigating its document source.

Given a Debug gesture beside displayed source, Hickory resolves the latest
compatible atomic project before discovering launch profiles. The viewed file
remains the breakpoint coordinate space; a selected application's entry point
may be another file. Definition-only library modules are not automatically
applications. Discovery reads project inputs without running them.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
