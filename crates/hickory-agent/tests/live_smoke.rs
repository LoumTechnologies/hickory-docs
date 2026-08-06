//! Live smoke test against the real Anthropic API.
//!
//! Ignored by default; run with:
//! `HICKORY_AGENT_LIVE=1 ANTHROPIC_API_KEY=... cargo test -p hickory-agent -- --ignored live_smoke`

use hickory_agent::{AgentConfig, AgentEvent, AnthropicClient, run_agent};
use hickory_executor::LocalExecutor;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "live network test — set HICKORY_AGENT_LIVE=1 and ANTHROPIC_API_KEY to run"]
async fn live_smoke() {
    if std::env::var("HICKORY_AGENT_LIVE").as_deref() != Ok("1") {
        eprintln!("HICKORY_AGENT_LIVE != 1 — skipping live smoke test");
        return;
    }
    assert!(
        std::env::var("ANTHROPIC_API_KEY").is_ok(),
        "ANTHROPIC_API_KEY must be set for the live smoke test"
    );

    let dir = tempfile::tempdir().unwrap();
    let llm = AnthropicClient::new().with_max_tokens(1024);
    let executor: std::sync::Arc<dyn hickory_executor::Executor> =
        std::sync::Arc::new(LocalExecutor::new().unwrap());
    let mut config = AgentConfig::new(
        "Run one shell command that prints the word smoke, then finish.",
        dir.path(),
    );
    config.max_turns = 4;

    let mut events = Vec::new();
    let outcome = run_agent(&llm, executor.clone(), &config, &mut |e: AgentEvent| {
        events.push(e);
    })
    .await
    .expect("live agent run failed");

    assert!(outcome.session_path.exists());
    let source = std::fs::read_to_string(&outcome.session_path).unwrap();
    hick_lang::parse_session(&source).expect("live session must parse");
    eprintln!(
        "live smoke ok: {} turns, session {}",
        outcome.turns,
        outcome.session_path.display()
    );
}
