<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Publication Floor Is Computed On Every Read And Shown

For: Engineers using local sessions or pairing their own machines.

Given a git repository open in the app, when the commit graph is read, then
the **publication floor** — `merge-base(HEAD, <published ref>)` — is computed
and reported alongside the commits, and every commit above it is marked a
draft with a sentence saying why.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/the-publication-floor-is-computed-and-shown.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/api/types.ts](../../../mechanisms/apps/web/src/api/types.ts.md#output-8a4c3188ff6c)
- [apps/web/src/views/GitPane.tsx](../../../mechanisms/apps/web/src/views/GitPane.tsx.md#output-f7cd01998027)
- [crates/hickory-cli/src/floor.rs](../../../mechanisms/crates/hickory-cli/src/floor.rs.md#output-e0386b43788a)
- [crates/hickory-cli/src/serve/git.rs](../../../mechanisms/crates/hickory-cli/src/serve/git.rs.md#output-3794196055eb)
- [crates/hickory-cli/src/serve/history.rs](../../../mechanisms/crates/hickory-cli/src/serve/history.rs.md#output-be949b7ea526)

### Tests and fixtures

- [apps/web/src/views/GitPane.test.tsx](../../../mechanisms/apps/web/src/views/GitPane.tsx.md#output-0e5613784fdf)
- [crates/hickory-cli/tests/replay_and_floor.rs](../../../mechanisms/crates/hickory-cli/tests/replay_and_floor.rs.md#output-652493fd98fd)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
