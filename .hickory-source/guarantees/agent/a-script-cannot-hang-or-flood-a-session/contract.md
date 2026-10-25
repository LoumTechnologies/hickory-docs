<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Model snippets cannot hang or flood an agent session indefinitely

For: Authors using an agent to change documents while retaining review control.

Given a model snippet interpreted in Monty, memory, execution duration, callback
count, recursion, sleep and captured output have explicit bounds. Exceeding a
bound produces a failed observation that Python can handle within its iteration
limit. Snippets have no shell or CPython execution fallback. Hosted process
lifetime, frame size and tool requests are independently bounded by Hickory.
Output beyond 64 KiB is refused rather than silently treated as complete.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/a-script-cannot-hang-or-flood-a-session.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
