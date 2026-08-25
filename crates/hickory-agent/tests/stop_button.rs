//! Pulling the cord on a running agent.
//!
//! Guarantee: docs/guarantees/agent/a-running-agent-can-be-stopped.md
//!
//! The scenario that forced this: a model degenerated into emitting
//! `<sh:exec></sh:exec>` in an endless stream, and the only way to stop
//! paying for it was to kill the whole program. The cancel flag must cut a
//! run mid-stream — dropping the provider stream is what stops the spend —
//! not merely between turns.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use hickory_agent::{
    AgentConfig, AgentEvent, ChatChunk, ChatStream, LlmClient, Message, STOPPED_BY_USER,
    ScriptedLlmClient, run_agent,
};
use hickory_executor::{Executor, LocalExecutor};

/// The runaway: a stream that never finishes, degenerating exactly the way
/// the real incident did.
struct EndlessLlm;

#[async_trait::async_trait]
impl LlmClient for EndlessLlm {
    async fn complete(&self, _messages: Vec<Message>) -> anyhow::Result<String> {
        anyhow::bail!("the runaway only streams")
    }

    async fn complete_stream(&self, _messages: Vec<Message>) -> anyhow::Result<ChatStream> {
        let stream = futures::stream::unfold(0u64, |n| async move {
            tokio::time::sleep(Duration::from_millis(2)).await;
            Some((Ok(ChatChunk::text("<sh:exec></sh:exec>")), n + 1))
        });
        Ok(Box::pin(stream))
    }

    fn provider_name(&self) -> &str {
        "endless"
    }

    fn model_name(&self) -> &str {
        "endless"
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn stop_cuts_a_runaway_stream_mid_generation() {
    let project = tempfile::tempdir().unwrap();
    let executor: Arc<dyn Executor> = Arc::new(LocalExecutor::new().unwrap());

    let cancel = Arc::new(AtomicBool::new(false));
    let mut config = AgentConfig::new("do a thing", project.path());
    config.cancel = Some(cancel.clone());

    let stopper = {
        let cancel = cancel.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            cancel.store(true, Ordering::Relaxed);
        })
    };

    let mut events = Vec::new();
    let mut on_event = |e: AgentEvent| events.push(e);
    // Without the mid-stream check this future never completes; the timeout
    // is what fails the test instead of hanging CI.
    let outcome = tokio::time::timeout(
        Duration::from_secs(10),
        run_agent(&EndlessLlm, executor.clone(), &config, &mut on_event),
    )
    .await
    .expect("the stop must cut the stream, not wait for it to finish");
    executor.shutdown().await.ok();
    stopper.await.ok();

    let error = match outcome {
        Err(error) => error,
        Ok(_) => panic!("a stopped run is not a finished run"),
    };
    assert_eq!(
        error.to_string(),
        STOPPED_BY_USER,
        "callers match this exact message to render a stop, not a failure"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, AgentEvent::Error { message } if message == STOPPED_BY_USER)),
        "the stream watcher needs a terminal event to clear its state"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_stop_before_the_first_call_never_reaches_the_model() {
    let project = tempfile::tempdir().unwrap();
    let executor: Arc<dyn Executor> = Arc::new(LocalExecutor::new().unwrap());

    // No scripted responses: reaching the model at all would fail with
    // "ran out of responses" rather than the stop sentinel.
    let llm = ScriptedLlmClient::new(Vec::<String>::new());
    let cancel = Arc::new(AtomicBool::new(true));
    let mut config = AgentConfig::new("do a thing", project.path());
    config.cancel = Some(cancel);

    let mut on_event = |_e: AgentEvent| {};
    let outcome = run_agent(&llm, executor.clone(), &config, &mut on_event).await;
    executor.shutdown().await.ok();
    let error = match outcome {
        Err(error) => error,
        Ok(_) => panic!("already stopped — the run must not proceed"),
    };
    assert_eq!(error.to_string(), STOPPED_BY_USER);
}
