<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Answer Does Not Cost You Your Place

For: Authors running interactive tools alongside documents without losing session state.

Given a session at the top of the attention queue, when its prompt is answered
from the attention card, then the answer reaches that session's terminal, the
session leaves the queue on the same request, and the pane the person was
working in is not closed, rearranged, or unfocused.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/terminal/an-answer-does-not-cost-you-your-place.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/terminal/AttentionCard.tsx](../../../mechanisms/apps/web/src/terminal/AttentionCard.tsx.md#output-da1d3639d06b)
- [crates/hickory-cli/src/serve/terminal.rs](../../../mechanisms/crates/hickory-cli/src/serve/terminal.rs.md#output-e67fa7430800)

### Tests and fixtures

- [apps/web/src/terminal/AttentionCard.test.tsx](../../../mechanisms/apps/web/src/terminal/AttentionCard.tsx.md#output-b44600c6bdc4)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
