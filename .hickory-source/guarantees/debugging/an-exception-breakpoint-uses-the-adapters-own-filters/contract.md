<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Exception Breakpoint Uses The Adapter's Own Filters

For: Authors debugging generated code while navigating its document source.

Given a debug session, when the adapter reports
`exceptionBreakpointFilters`, then the strip draws one switch per filter, in
the adapter's own words, and toggling one sends `setExceptionBreakpoints`
with the ids that are on. An adapter reporting none draws nothing.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/an-exception-breakpoint-uses-the-adapters-own-filters.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
