<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# One Verb Brings Bytes Into A Document

For: Authors editing executable documents and the files those documents produce.

Given bytes a document did not write — a cell's output volume, a plain file,
a session, what a session carries forward, another harness's session log,
an inbox transcript — when they are to become a document's own, then the
command is `hick ingest`, with `--from` naming the source (`'#cell'`,
`file`, `session`, `carry`, `claude-code`; absent for the inbox), and every
source does the same three things: gives the bytes a home in a document,
records where they came from, and — where the bytes are generated — proves
the weave is unchanged before writing anything. `adopt`, `promote`, `carry`
and `import` no longer exist as commands.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/one-verb-brings-bytes-into-a-document.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
