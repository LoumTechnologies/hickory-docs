<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# `hick test` Never Spends The Reader's Tokens

For: Authors using an agent to change documents while retaining review control.

Given a document containing a `<hick:agent>` cell, when `hick test` runs it,
then **no model is called**, no matter what credentials the machine holds. The
cell is verified against its recording, or reported **unverifiable** (exit `2`)
with a message naming the cell, its prompt, and how to establish a baseline.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/test-never-spends-tokens.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)
- [crates/hickory-cli/src/agent_cell_runner.rs](../../../mechanisms/crates/hickory-cli/src/agent_cell_runner.rs.md#output-5fb6c450e7d2)
- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)

### Tests and fixtures

- [crates/hick-literate/tests/agent_cells.rs](../../../mechanisms/crates/hick-literate/tests/agent_cells.rs.md#output-d49b19009bed)
- [crates/hickory-cli/tests/agent_cell_vertex.rs](../../../mechanisms/crates/hickory-cli/tests/agent_cell_vertex.rs.md#output-932eec9d0689)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
