<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Context Provenance Is Derived From The Session, Never From The Model

For: Authors using an agent to change documents while retaining review control.

Given an agent session, when a tool shows the model a file (`read_doc`,
`read_output`, `read_file`), then the session records `<hick:read>` — the
path, the content's SHA-256, the repository commit when there is one, and the
lines shown; when an edit tool writes, then the session records
`<hick:wrote>` — the file, the lines as they stand after the edit, and their
hashline hashes; and when anyone asks (`hick context`, `GET /api/docs/:id/context`,
the app's Context layer), native writes use the recorded message envelope supplied to the provider client and
reads from this turn. Adapter writes use recorded observations from this turn,
with opaque adapter-managed context explicitly outside that scope. Abandoned
branches are retained historically but are never counted as supplied inputs.
Availability is not evidence that a claim follows from a source.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/context-provenance-is-derived-from-the-session.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
