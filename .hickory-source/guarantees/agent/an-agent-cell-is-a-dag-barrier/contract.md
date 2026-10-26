<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Agent Cell Is A DAG Vertex, And Its Edges Are A Barrier

For: Authors using an agent to change documents while retaining review control.

Given a document containing `<hick:agent>`, when the pipeline builds the
information-flow DAG, then the cell is a **vertex** — scheduled by the same
topological loop as `<hick:exec>`, covered by the same freeze, recording,
`never_run`, and `hick:expect` machinery — and its edges are a **barrier**:
every cell declared before it precedes it, and every cell declared after it
follows it.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/an-agent-cell-is-a-dag-barrier.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

## Test the behavior: `agent_cell_is_a_dag_vertex_with_a_reserved_container`

This is the maintained test fragment from `crates/hick-exec/src/dag/tests/tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-cc538e30a76b-agent_cell_is_a_dag_vertex_with_a_reserved_container"></a>
### `agent_cell_is_a_dag_vertex_with_a_reserved_container`

<storycode:copy id="f-cc538e30a76b-agent_cell_is_a_dag_vertex_with_a_reserved_container">
// The five tests below protect
// docs/guarantees/agent/an-agent-cell-is-a-dag-barrier.md.

#[test]
fn agent_cell_is_a_dag_vertex_with_a_reserved_container() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:agent id="a1" max-turns="7" model="claude-sonnet-5">
<hick:prompt>write the greeting</hick:prompt>
</hick:agent>
</hick:doc>"#;
    let dag = parse_and_build(src).unwrap();
    assert_eq!(dag.execs.len(), 1, "an agent cell is a DAG vertex");
    let cell = &dag.execs[0];
    assert!(cell.is_agent());
    assert!(
        is_agent_container(&cell.container),
        "reserved synthetic container name, not a declarable one: {}",
        cell.container
    );
    let agent = cell.agent.as_ref().unwrap();
    assert_eq!(agent.id.as_deref(), Some("a1"));
    assert_eq!(agent.max_turns, Some(7));
    assert_eq!(agent.model.as_deref(), Some("claude-sonnet-5"));
    assert_eq!(agent.prompt.trim(), "write the greeting");
    assert_eq!(
        cell.command.trim(),
        "write the greeting",
        "command carries the prompt so consumers keyed on it keep working"
    );
}

</storycode:copy>

## Implementation review

### Implementation

- [crates/hick-exec/src/dag.rs](../../../mechanisms/crates/hick-exec/src/dag.rs.md#output-e6f480388088)
- [crates/hick-literate/src/expect.rs](../../../mechanisms/crates/hick-literate/src/expect.rs.md#output-162ca50044aa)
- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)

### Tests and fixtures

- [crates/hick-exec/src/dag/tests/tests.rs](../../../mechanisms/crates/hick-exec/src/dag/tests/tests.rs.md#output-cc538e30a76b)
- [crates/hick-literate/tests/agent_cells.rs](../../../mechanisms/crates/hick-literate/tests/agent_cells.rs.md#output-d49b19009bed)
- [crates/hickory-cli/tests/agent_cell_vertex.rs](../../../mechanisms/crates/hickory-cli/tests/agent_cell_vertex.rs.md#output-932eec9d0689)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
