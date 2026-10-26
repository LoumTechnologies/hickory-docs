<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
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


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/one-verb-brings-bytes-into-a-document.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hick-literate/src/promote.rs](../../../mechanisms/crates/hick-literate/src/promote.rs.md#output-36fb154eeee4)
- [crates/hickory-cli/src/adopt.rs](../../../mechanisms/crates/hickory-cli/src/adopt.rs.md#output-b820c141d77c)
- [crates/hickory-cli/src/carry.rs](../../agent/rewind-and-re-run-are-two-different-acts/contract.md#output-79fc41424ae4)
- [crates/hickory-cli/src/claude_code.rs](../../agent/a-claude-code-transcript-imports-as-a-session/contract.md#output-d087b883d22c)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)

### Tests and fixtures

- [crates/hickory-cli/tests/ingest.rs](../../../mechanisms/crates/hickory-cli/tests/ingest.rs.md#output-a492c9d4e3cc)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
