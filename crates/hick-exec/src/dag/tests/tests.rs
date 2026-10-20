use super::*;

fn parse_and_build(src: &str) -> Result<FlowDag, DagValidationError> {
    let doc = hick_lang::parse(src).expect("parse failed");
    build_dag(&doc)
}

// The three tests below protect
// docs/guarantees/verification/freeze-is-declared-per-cell.md — the
// attribute-parsing half of it.

#[test]
fn freeze_attribute_parses_both_values() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="frozen" image="alpine" freeze="true">
cat lockfile
</hick:exec>
<hick:exec container="live" image="alpine" freeze="FALSE">
run the integration test
</hick:exec>
</hick:doc>"#;
    let dag = parse_and_build(src).unwrap();
    assert_eq!(dag.execs[0].freeze, Some(true));
    assert_eq!(dag.execs[1].freeze, Some(false));
}

#[test]
fn freeze_attribute_defaults_to_inherit() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine">
echo hello
</hick:exec>
</hick:doc>"#;
    let dag = parse_and_build(src).unwrap();
    assert_eq!(
        dag.execs[0].freeze, None,
        "an absent attribute must mean 'inherit the run-wide default', \
             not 'live' — otherwise --freeze would stop working"
    );
}

#[test]
fn freeze_attribute_rejects_non_boolean() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine" freeze="yes">
echo hello
</hick:exec>
</hick:doc>"#;
    let err = parse_and_build(src).expect_err("freeze=\"yes\" must not be accepted");
    assert!(
        matches!(err, DagValidationError::InvalidFreeze { ref value, .. } if value == "yes"),
        "unexpected error: {err}"
    );
    let msg = err.to_string();
    assert!(msg.contains("freeze=\"true\"") && msg.contains("freeze=\"false\""));
}

// The three tests below protect
// docs/guarantees/execution/a-cell-cannot-hang-a-run.md — the
// attribute-parsing half of it.

#[test]
fn timeout_attribute_parses_seconds_and_zero() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="slow" image="alpine" timeout="300">
run the long benchmark
</hick:exec>
<hick:exec container="endless" image="alpine" timeout="0">
serve until killed by hand
</hick:exec>
</hick:doc>"#;
    let dag = parse_and_build(src).unwrap();
    assert_eq!(dag.execs[0].timeout_secs, Some(300));
    assert_eq!(
        dag.execs[1].timeout_secs,
        Some(0),
        "timeout=\"0\" is an explicit 'unbounded', distinct from the attribute being absent"
    );
}

#[test]
fn timeout_attribute_defaults_to_inherit() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine">
echo hello
</hick:exec>
</hick:doc>"#;
    let dag = parse_and_build(src).unwrap();
    assert_eq!(
        dag.execs[0].timeout_secs, None,
        "an absent attribute must mean 'inherit the run-wide default', \
             not 'unbounded' — otherwise the default timeout would stop working"
    );
}

#[test]
fn timeout_attribute_rejects_non_integer_values() {
    for bad in ["1.5", "2m", "-1", "", "unbounded"] {
        let src = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine" timeout="{bad}">
echo hello
</hick:exec>
</hick:doc>"#
        );
        let err = parse_and_build(&src).expect_err("a malformed timeout must not be accepted");
        assert!(
            matches!(err, DagValidationError::InvalidTimeout { ref value, .. } if value == bad),
            "timeout=\"{bad}\": unexpected error: {err}"
        );
        let msg = err.to_string();
        assert!(
            msg.contains("timeout=\"0\"") && msg.contains("HICKORY_CELL_TIMEOUT"),
            "the error must name the valid shapes and the env default: {msg}"
        );
    }
}

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

#[test]
fn an_agent_cell_is_a_barrier_in_both_directions() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="a" image="alpine" />
<hick:container name="b" image="alpine" />
<hick:exec container="a">echo before</hick:exec>
<hick:agent id="a1"><hick:prompt>do the thing</hick:prompt></hick:agent>
<hick:exec container="b">echo after</hick:exec>
</hick:doc>"#;
    let dag = parse_and_build(src).unwrap();
    // Without the barrier these two execs are in different containers and
    // would be roots in parallel.
    let barrier: Vec<_> = dag
        .edges
        .iter()
        .filter(|e| matches!(&e.reason, DependencyReason::AgentBarrier { .. }))
        .map(|e| (e.from, e.to))
        .collect();
    assert!(barrier.contains(&(ExecId(0), ExecId(1))), "before → agent");
    assert!(barrier.contains(&(ExecId(1), ExecId(2))), "agent → after");
    let order = dag.topological_order();
    assert_eq!(order, vec![ExecId(0), ExecId(1), ExecId(2)]);
    assert_eq!(
        dag.roots,
        vec![ExecId(0)],
        "the barrier leaves exactly one root"
    );
}

#[test]
fn two_agent_cells_are_ordered_by_barriers_not_container_state() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:agent id="first"><hick:prompt>one</hick:prompt></hick:agent>
<hick:agent id="second"><hick:prompt>two</hick:prompt></hick:agent>
</hick:doc>"#;
    let dag = parse_and_build(src).unwrap();
    assert_ne!(
        dag.execs[0].container, dag.execs[1].container,
        "each agent cell gets its own reserved name"
    );
    assert!(
        dag.edges
            .iter()
            .all(|e| matches!(&e.reason, DependencyReason::AgentBarrier { .. })),
        "ordering comes from the barrier, never from container state"
    );
    assert_eq!(dag.topological_order(), vec![ExecId(0), ExecId(1)]);
    assert_eq!(dag.execs[0].agent.as_ref().unwrap().ordinal, 0);
    assert_eq!(dag.execs[1].agent.as_ref().unwrap().ordinal, 1);
}

#[test]
fn an_agent_cell_without_a_prompt_is_rejected() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:agent id="a1" />
</hick:doc>"#;
    let err = parse_and_build(src).expect_err("a promptless agent cell must not build");
    assert!(matches!(err, DagValidationError::AgentWithoutPrompt { .. }));
    assert!(err.to_string().contains("<hick:prompt>"));
}

#[test]
fn a_non_numeric_max_turns_is_rejected() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:agent max-turns="lots"><hick:prompt>go</hick:prompt></hick:agent>
</hick:doc>"#;
    let err = parse_and_build(src).expect_err("max-turns=\"lots\" must not be accepted");
    assert!(
        matches!(err, DagValidationError::InvalidMaxTurns { ref value, .. } if value == "lots"),
        "unexpected error: {err}"
    );
}

#[test]
fn single_exec_no_deps() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine">
echo hello
</hick:exec>
</hick:doc>"#;
    let dag = parse_and_build(src).unwrap();
    assert_eq!(dag.execs.len(), 1);
    assert_eq!(dag.edges.len(), 0);
    assert_eq!(dag.roots.len(), 1);
}

#[test]
fn sequential_execs_same_container() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine">
apk add curl
</hick:exec>
<hick:exec container="demo">
curl --version
</hick:exec>
</hick:doc>"#;
    let dag = parse_and_build(src).unwrap();
    assert_eq!(dag.execs.len(), 2);
    assert_eq!(dag.edges.len(), 1);
    assert_eq!(dag.edges[0].from, ExecId(0));
    assert_eq!(dag.edges[0].to, ExecId(1));
    assert!(matches!(
        dag.edges[0].reason,
        DependencyReason::ContainerState { .. }
    ));
}

#[test]
fn parallel_execs_different_containers() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="a" image="alpine" />
<hick:container name="b" image="alpine" />
<hick:exec container="a">
echo a
</hick:exec>
<hick:exec container="b">
echo b
</hick:exec>
</hick:doc>"#;
    let dag = parse_and_build(src).unwrap();
    assert_eq!(dag.execs.len(), 2);
    assert_eq!(dag.edges.len(), 0);
    assert_eq!(dag.roots.len(), 2);
}

#[test]
fn volume_dependency() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="writer" image="alpine" />
<hick:container name="reader" image="alpine" />
<hick:volume name="shared" />
<hick:exec container="writer" mount="shared:/output">
echo data > /output/file.txt
</hick:exec>
<hick:exec container="reader" mount="shared:/input">
cat /input/file.txt
</hick:exec>
</hick:doc>"#;
    let dag = parse_and_build(src).unwrap();
    assert_eq!(dag.execs.len(), 2);
    assert!(!dag.edges.is_empty());
    let volume_edge = dag
        .edges
        .iter()
        .find(|e| matches!(&e.reason, DependencyReason::VolumeFlow { .. }));
    assert!(volume_edge.is_some());
}

#[test]
fn fork_dependency() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="base" image="ubuntu:22.04" />
<hick:exec container="base">
apt-get update
</hick:exec>
<hick:fork from="base" to="analyzer" />
<hick:exec container="analyzer">
./run-analysis
</hick:exec>
</hick:doc>"#;
    let dag = parse_and_build(src).unwrap();
    assert_eq!(dag.execs.len(), 2);
    let fork_edge = dag
        .edges
        .iter()
        .find(|e| matches!(&e.reason, DependencyReason::Fork { .. }));
    assert!(fork_edge.is_some());
}

#[test]
fn topological_order() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine">
step 1
</hick:exec>
<hick:exec container="demo">
step 2
</hick:exec>
<hick:exec container="demo">
step 3
</hick:exec>
</hick:doc>"#;
    let dag = parse_and_build(src).unwrap();
    let order = dag.topological_order();
    assert_eq!(order.len(), 3);
    assert_eq!(order[0], ExecId(0));
    assert_eq!(order[1], ExecId(1));
    assert_eq!(order[2], ExecId(2));
}

#[test]
fn nested_exec_in_file() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="out.md">
<hick:exec container="demo" image="alpine">
cat /etc/alpine-release
</hick:exec>
</hick:file>
</hick:doc>"#;
    let dag = parse_and_build(src).unwrap();
    assert_eq!(dag.execs.len(), 1);
    assert_eq!(dag.execs[0].container, "demo");
}

#[test]
fn volume_with_explicit_access_rules() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="scaffolder" image="alpine" />
<hick:container name="linter" image="alpine" />
<hick:volume name="project" output="src/MyApi/">
  <hick:allow container="scaffolder" write="**" />
  <hick:allow container="linter" read="**" />
</hick:volume>
<hick:exec container="scaffolder" mount="project:/out">echo scaffold</hick:exec>
<hick:exec container="linter" mount="project:/in">echo lint</hick:exec>
</hick:doc>"#;
    let dag = parse_and_build(src).unwrap();
    assert_eq!(dag.execs.len(), 2);

    // scaffolder is writer, linter is reader → writer→reader edge
    let volume_edge = dag
        .edges
        .iter()
        .find(|e| matches!(&e.reason, DependencyReason::VolumeFlow { .. }));
    assert!(
        volume_edge.is_some(),
        "expected VolumeFlow edge from writer to reader"
    );
    let ve = volume_edge.unwrap();
    assert_eq!(ve.from, ExecId(0)); // scaffolder
    assert_eq!(ve.to, ExecId(1)); // linter

    // Volume declaration is stored
    assert!(dag.volumes.contains_key("project"));
    let vol = &dag.volumes["project"];
    assert_eq!(vol.access_rules.len(), 2);
    assert!(matches!(vol.kind, VolumeKind::Output { .. }));
}

#[test]
fn volume_backward_compat_no_access_rules() {
    // Volume with no <allow> children uses heuristic: first mounter = writer
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="a" image="alpine" />
<hick:container name="b" image="alpine" />
<hick:volume name="scratch" />
<hick:exec container="a" mount="scratch:/data">echo write</hick:exec>
<hick:exec container="b" mount="scratch:/data">echo read</hick:exec>
</hick:doc>"#;
    let dag = parse_and_build(src).unwrap();
    let volume_edge = dag
        .edges
        .iter()
        .find(|e| matches!(&e.reason, DependencyReason::VolumeFlow { .. }));
    assert!(
        volume_edge.is_some(),
        "expected VolumeFlow edge via heuristic"
    );
}

#[test]
fn volume_declaration_parsing() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:volume name="input-vol" input="." />
<hick:volume name="output-vol" output="dist/" />
<hick:volume name="both-vol" input="." output="." />
<hick:volume name="ephemeral" />
</hick:doc>"#;
    let dag = parse_and_build(src).unwrap();
    assert_eq!(dag.volumes.len(), 4);

    assert!(matches!(dag.volumes["input-vol"].kind, VolumeKind::Input { ref path } if path == "."));
    assert!(
        matches!(dag.volumes["output-vol"].kind, VolumeKind::Output { ref path } if path == "dist/")
    );
    assert!(matches!(
        dag.volumes["both-vol"].kind,
        VolumeKind::InputOutput { .. }
    ));
    assert!(matches!(
        dag.volumes["ephemeral"].kind,
        VolumeKind::Ephemeral
    ));
}

#[test]
fn volume_multiple_writers_sequential() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="step1" image="alpine" />
<hick:container name="step2" image="alpine" />
<hick:container name="consumer" image="alpine" />
<hick:volume name="shared">
  <hick:allow container="step1" write="**" />
  <hick:allow container="step2" write="**" />
  <hick:allow container="consumer" read="**" />
</hick:volume>
<hick:exec container="step1" mount="shared:/out">echo step1</hick:exec>
<hick:exec container="step2" mount="shared:/out">echo step2</hick:exec>
<hick:exec container="consumer" mount="shared:/in">echo consume</hick:exec>
</hick:doc>"#;
    let dag = parse_and_build(src).unwrap();

    // Both writers should have edges to consumer
    let writer_to_consumer: Vec<_> = dag
        .edges
        .iter()
        .filter(|e| e.to == ExecId(2) && matches!(&e.reason, DependencyReason::VolumeFlow { .. }))
        .collect();
    assert_eq!(
        writer_to_consumer.len(),
        2,
        "both writers should have edge to consumer"
    );

    // Writers should be sequentially ordered
    let writer_seq: Vec<_> = dag
        .edges
        .iter()
        .filter(|e| {
            e.from == ExecId(0)
                && e.to == ExecId(1)
                && matches!(&e.reason, DependencyReason::VolumeFlow { .. })
        })
        .collect();
    assert_eq!(
        writer_seq.len(),
        1,
        "writers should be sequentially ordered"
    );
}
