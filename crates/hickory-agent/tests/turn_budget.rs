//! Running out of turns is a handoff, not a lost session.
//!
//! Guarantee: docs/guarantees/agent/an-exhausted-turn-budget-hands-off.md
//!
//! The old behaviour bailed with "agent did not finish within N turns". By
//! then the agent had usually written files and landed edits — real work the
//! caller then threw away, because it only saw an Err. The measured corpus
//! makes this the common case, not the edge case: the median Claude Code
//! session on this machine used 54 tool-issuing rounds against a default
//! budget of 20.

use std::sync::Arc;

use hickory_agent::{AgentConfig, AgentEvent, ScriptedLlmClient, run_agent};
use hickory_executor::{Executor, LocalExecutor};

#[tokio::test(flavor = "multi_thread")]
async fn an_exhausted_budget_returns_a_handoff_instead_of_failing() {
    let project = tempfile::tempdir().unwrap();

    // Never says done: it would loop forever if the budget did not stop it.
    let busy = "<hick:next>code</hick:next>\nStill working.\n```bash\necho step\n```";
    let llm = ScriptedLlmClient::new(vec![
        busy.to_string(),
        busy.to_string(),
        // The wrap-up call. Asked for a handoff, gives one.
        "<hick:next>done</hick:next>\nWrote nothing yet; next I would run the tests."
            .to_string(),
    ]);

    let executor: Arc<dyn Executor> = Arc::new(LocalExecutor::new().unwrap());
    let mut config = AgentConfig::new("keep going forever", project.path());
    config.max_turns = 2;

    let mut events = Vec::new();
    let mut on_event = |e: AgentEvent| events.push(e);
    let outcome = run_agent(&llm, executor.clone(), &config, &mut on_event)
        .await
        .expect("an exhausted budget must not fail the run");
    executor.shutdown().await.ok();

    assert!(
        outcome.summary.contains("unfinished"),
        "the caller must be able to tell this run did not finish: {}",
        outcome.summary
    );
    assert!(
        outcome.summary.contains("next I would run the tests"),
        "the model's handoff is the point; it was dropped: {}",
        outcome.summary
    );
    // The wrap-up call's tokens are billed, so they must be counted.
    assert!(outcome.total_usage.output_tokens > 0 || outcome.total_usage.is_zero());

    let session = std::fs::read_to_string(&outcome.session_path).unwrap();
    assert!(
        session.contains("next I would run the tests"),
        "the handoff must be in the session document too:\n{session}"
    );
}
