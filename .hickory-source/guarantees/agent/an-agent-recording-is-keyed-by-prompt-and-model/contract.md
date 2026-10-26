<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Agent Recording Is Keyed By The Prompt And The Model

For: Authors using an agent to change documents while retaining review control.

Given a `<hick:agent>` cell whose answer is recorded in
`.hick-cache/transcripts/`, when that recording is looked up, then the key is a
hash of **the prompt and the model together**. Editing either retires the
recording; a retired recording is a missing baseline, never a stale answer
served as a current one.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/an-agent-recording-is-keyed-by-prompt-and-model.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hick-literate/src/cache.rs](../../../mechanisms/crates/hick-literate/src/cache.rs.md#output-4b4919c14d46)
- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)
- [crates/hickory-cli/src/agent_cell_runner.rs](../../../mechanisms/crates/hickory-cli/src/agent_cell_runner.rs.md#output-5fb6c450e7d2)

### Tests and fixtures

- [crates/hick-literate/tests/agent_cells.rs](../../../mechanisms/crates/hick-literate/tests/agent_cells.rs.md#output-d49b19009bed)
- [crates/hickory-cli/tests/agent_cell_vertex.rs](../../../mechanisms/crates/hickory-cli/tests/agent_cell_vertex.rs.md#output-932eec9d0689)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
