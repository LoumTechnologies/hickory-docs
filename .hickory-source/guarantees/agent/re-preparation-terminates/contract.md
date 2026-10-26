<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Re-Preparation After An Agent Edit Terminates, Under A Declared Bound

For: Authors using an agent to change documents while retaining review control.

Given a document whose `<hick:agent>` cell edits that document's own source,
when the pipeline re-prepares the document so the rest of the pass runs against
what the agent wrote, then re-preparation reaches a **declared fixed point**
under a **declared bound**, and a document with no fixed point **fails** rather
than looping.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/re-preparation-terminates.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)

### Tests and fixtures

- [crates/hick-literate/tests/agent_cells.rs](../../../mechanisms/crates/hick-literate/tests/agent_cells.rs.md#output-d49b19009bed)
- [crates/hickory-cli/tests/agent_cell_vertex.rs](../../../mechanisms/crates/hickory-cli/tests/agent_cell_vertex.rs.md#output-932eec9d0689)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
