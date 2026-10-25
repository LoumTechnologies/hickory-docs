<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Missing Key Degrades to a Note, Never a Crash

For: Authors using an agent to change documents while retaining review control.

Given a local session (the desktop app's server) on a machine with no LLM
provider key in the environment, when the user sends a prompt to the in-app
agent, then `POST /api/docs/:id/agent` answers `503` with an error message
that **starts with** `agent not available` and goes on to name every
environment variable that would fix it — and nothing else about the session
degrades: documents still open, edit, weave, and run.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/a-missing-key-degrades-to-a-note.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
