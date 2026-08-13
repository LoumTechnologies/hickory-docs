//! End-to-end proof that "agent output is literate programming state in git":
//!
//! a canned LLM drives the ReAct loop (no network) → the session is written
//! as a `hick:session` file → `hick-lang` parses it → `hick promote`
//! produces a clean `hick:doc` pipeline → `hick test` accepts it.

use hick_lang::SessionNode;
use hick_literate::promote::{PromoteOpts, promote};
use hickory_agent::{AgentConfig, AgentEvent, ScriptedLlmClient, run_agent};
use hickory_cli::{ExecutorChoice, RunMode, check_failures, run_doc, write_outputs};
use hickory_executor::LocalExecutor;

/// Turn 1: run a python script that registers a new pipeline file. The
/// `hick.pipeline_new_file(...)` call is what `hick promote` extracts;
/// the local shim keeps the script runnable without any real `hick` module.
const TURN_CODE: &str = r#"<hick:next>code</hick:next>
I'll register the greeting file with the pipeline.
```python
def _register(*args):
    pass

class hick:
    pipeline_new_file = _register

hick.pipeline_new_file('greeting.txt', 'hello from hick agent')
print('registered greeting.txt')
```"#;

const TURN_DONE: &str = "<hick:next>done</hick:next>\nRegistered greeting.txt with the pipeline.";

#[tokio::test(flavor = "multi_thread")]
async fn agent_session_promotes_to_checked_pipeline() {
    let project = tempfile::tempdir().unwrap();
    let project_dir = project.path();

    // 1. Drive the ReAct loop with scripted responses — no network.
    let llm = ScriptedLlmClient::new([TURN_CODE, TURN_DONE]);
    let executor: std::sync::Arc<dyn hickory_executor::Executor> =
        std::sync::Arc::new(LocalExecutor::new().unwrap());
    let config = AgentConfig::new("create a greeting file", project_dir);

    let mut events = Vec::new();
    let outcome = run_agent(&llm, executor.clone(), &config, &mut |e: AgentEvent| {
        events.push(e);
    })
    .await
    .expect("agent run failed");

    assert_eq!(outcome.turns, 2);
    assert_eq!(
        outcome.summary,
        "Registered greeting.txt with the pipeline."
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, AgentEvent::ScriptFinished { .. }))
    );

    // 2. The session file landed in sessions/ and parses as a SessionDocument.
    assert!(
        outcome
            .session_path
            .starts_with(project_dir.join("sessions"))
    );
    let session_source = std::fs::read_to_string(&outcome.session_path).unwrap();
    let session = hick_lang::parse_session(&session_source).expect("session must parse");
    assert!(matches!(session.nodes[0], SessionNode::User { .. }));
    let has_action = session
        .nodes
        .iter()
        .any(|n| matches!(n, SessionNode::Assistant { actions, .. } if !actions.is_empty()));
    assert!(has_action, "session must contain the executed action");
    assert!(
        session
            .nodes
            .iter()
            .any(|n| matches!(n, SessionNode::Observation { .. })),
        "session must contain the captured observation"
    );

    // 3. Promote the session into a clean pipeline document.
    let promoted = promote(&PromoteOpts {
        session_source: &session_source,
        project_dir,
        session_name: "e2e-session.hick",
    })
    .expect("promotion failed");
    assert_eq!(promoted.surviving_writes, 1);
    assert!(promoted.promoted_source.contains("greeting.txt"));
    assert!(promoted.promoted_source.contains("hello from hick agent"));

    // 4. The promoted document is a valid hick:doc that `hick test`
    //    accepts once its outputs are committed.
    let doc_path = project_dir.join("promoted.hick");
    std::fs::write(&doc_path, &promoted.promoted_source).unwrap();

    let run = run_doc(&doc_path, &[], RunMode::Execute, ExecutorChoice::Local)
        .await
        .expect("promoted doc must execute");
    let written = write_outputs(&run, None).unwrap();
    assert!(
        written
            .iter()
            .any(|p| p.file_name().is_some_and(|n| n == "greeting.txt")),
        "promoted doc must produce greeting.txt, wrote: {written:?}"
    );

    // Re-run and verify: no expectation failures, no drift — `hick test`
    // exits successfully on this document.
    let check_run = run_doc(&doc_path, &[], RunMode::Execute, ExecutorChoice::Local)
        .await
        .expect("check run must execute");
    let failures = check_failures(&check_run, None).unwrap();
    assert!(failures.is_empty(), "hick test failures: {failures:?}");
}
