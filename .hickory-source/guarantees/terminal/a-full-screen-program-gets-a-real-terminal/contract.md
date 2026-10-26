<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Full-Screen Program Gets A Real Terminal

For: Authors running interactive tools alongside documents without losing session state.

Given a session running a program that takes over the screen — an editor, a
process monitor, a pager, another vendor's coding agent — when it runs in one
of these terminals, then it gets everything it needs to draw itself:

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/terminal/a-full-screen-program-gets-a-real-terminal.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Tests and fixtures

- [crates/hick-term/tests/full_screen_apps.rs](../../../mechanisms/crates/hick-term/tests/full_screen_apps.rs.md#output-31c9ccf64945)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
