<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Anchored Terminal Writes What You Typed

For: Authors running interactive tools alongside documents without losing session state.

Given a terminal anchored to a container in a document, when you type a
command into it, then that line becomes part of that container's cell — and
the cell is always **a prefix of the session that reproduces**:

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/terminal/an-anchored-terminal-writes-what-you-typed.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation chapters

These chapters keep this behavior's larger implementation readable without duplicating shared code.

- [`apps/web/src/terminal/AnchorBar.tsx`](implementation/apps/web/src/terminal/AnchorBar.tsx.md#output-bdb1fbadcd32)
- [`crates/hick-term/src/anchor.rs`](implementation/crates/hick-term/src/anchor.rs.md#output-ea39feed7f90)
- [`crates/hick-term/src/command.rs`](implementation/crates/hick-term/src/command.rs.md#output-764182715548)
- [`crates/hick-term/tests/typed_commands.rs`](implementation/crates/hick-term/tests/typed_commands.rs.md#output-67c746c3413a)
- [`crates/hickory-cli/src/anchor.rs`](implementation/crates/hickory-cli/src/anchor.rs.md#output-0234532ad992)
- [`crates/hickory-cli/src/serve/anchored.rs`](implementation/crates/hickory-cli/src/serve/anchored.rs.md#output-7e6ee30b8619)
- [`crates/hickory-cli/tests/anchored_terminal.rs`](implementation/crates/hickory-cli/tests/anchored_terminal.rs.md#output-3d6fdffbcf9a)
<storycode:include file="implementation/apps/web/src/terminal/AnchorBar.tsx.md" />
<storycode:include file="implementation/crates/hick-term/src/anchor.rs.md" />
<storycode:include file="implementation/crates/hick-term/src/command.rs.md" />
<storycode:include file="implementation/crates/hick-term/tests/typed_commands.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/src/anchor.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/src/serve/anchored.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/tests/anchored_terminal.rs.md" />

## Implementation review

### Implementation

- [apps/web/src/terminal/AnchorBar.tsx](implementation/apps/web/src/terminal/AnchorBar.tsx.md#output-bdb1fbadcd32)
- [crates/hick-term/src/anchor.rs](implementation/crates/hick-term/src/anchor.rs.md#output-ea39feed7f90)
- [crates/hick-term/src/command.rs](implementation/crates/hick-term/src/command.rs.md#output-764182715548)
- [crates/hick-term/src/session.rs](../../../mechanisms/crates/hick-term/src/session.rs.md#output-5705d2b8acc0)
- [crates/hick-term/src/shell_integration.rs](../../../mechanisms/crates/hick-term/src/shell_integration.rs.md#output-3465af886d7a)
- [crates/hickory-cli/src/anchor.rs](implementation/crates/hickory-cli/src/anchor.rs.md#output-0234532ad992)
- [crates/hickory-cli/src/serve/anchored.rs](implementation/crates/hickory-cli/src/serve/anchored.rs.md#output-7e6ee30b8619)

### Tests and fixtures

- [crates/hick-term/tests/typed_commands.rs](implementation/crates/hick-term/tests/typed_commands.rs.md#output-67c746c3413a)
- [crates/hickory-cli/tests/anchored_terminal.rs](implementation/crates/hickory-cli/tests/anchored_terminal.rs.md#output-3d6fdffbcf9a)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
