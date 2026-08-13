//! An agent fixes a disagreement where it is RECORDED, not where it is felt.
//!
//! Guarantee: docs/guarantees/agent/the-editable-set-is-the-pipeline-closure.md
//!
//! The chain exists so a decision is written down once. An agent scoped to a
//! single document cannot honour that: told "the requirement is wrong", the
//! only edit available to it is a local one, so it restates the upstream fact
//! in a second place — manufacturing the exact contradiction the chain was
//! built to prevent. The editable set is therefore the pipeline closure the
//! documents declare themselves.

use std::sync::Arc;

use hickory_agent::{AgentConfig, AgentEvent, ScriptedLlmClient, run_agent};
use hickory_executor::{Executor, LocalExecutor};

/// Two hops: requirements -> domain -> decisions.
const DECISIONS: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="decisions.md">
# Decisions

<hick:paste select=".decision" />

<hick:copy id="d-retention" class="decision">
Records are retained for 30 days.
</hick:copy>
</hick:doc>
"##;

const DOMAIN: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="domain.md">
# Domain

<hick:upstream file="decisions.hick" />

<hick:paste select=".decision" />
</hick:doc>
"##;

const REQUIREMENTS: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="requirements.md">
# Requirements

<hick:upstream file="domain.hick" />

<hick:paste select=".decision" />
</hick:doc>
"##;

#[tokio::test(flavor = "multi_thread")]
async fn the_agent_edits_a_decision_two_hops_upstream() {
    let dir = tempfile::tempdir().unwrap();
    let path = |n: &str| dir.path().join(n);
    std::fs::write(path("decisions.hick"), DECISIONS).unwrap();
    std::fs::write(path("domain.hick"), DOMAIN).unwrap();
    std::fs::write(path("requirements.hick"), REQUIREMENTS).unwrap();

    let llm = ScriptedLlmClient::new(vec![
        // Read the primary: the result must name the reachable chain.
        "<hick:next>tool</hick:next>\nReading the primary.\n\
         <hick:tool name=\"read_doc\">\n</hick:tool>"
            .to_string(),
        // Read two hops up by file name alone.
        "<hick:next>tool</hick:next>\nReading the decisions.\n\
         <hick:tool name=\"read_doc\">\n\
         <hick:arg name=\"doc\">decisions.hick</hick:arg>\n</hick:tool>"
            .to_string(),
        // Fix it AT THE SOURCE, addressed by line content hash.
        "<hick:next>tool</hick:next>\nChanging the decision itself.\n\
         <hick:tool name=\"edit_doc\">\n\
         <hick:arg name=\"doc\">decisions.hick</hick:arg>\n\
         <hick:arg name=\"run\">RETENTION_HASH</hick:arg>\n\
         <hick:input>Records are retained for 90 days.</hick:input>\n</hick:tool>"
            .to_string(),
        "<hick:next>done</hick:next>\nChanged the decision where it is recorded.".to_string(),
    ]);

    // The scripted edit needs the real content hash of the line it targets,
    // which only the session can produce. Run once to read it, then script it.
    let executor: Arc<dyn Executor> = Arc::new(LocalExecutor::new().unwrap());
    let mut config = AgentConfig::new("Retention should be 90 days.", dir.path());
    config.doc_path = Some(path("requirements.hick"));
    config.max_turns = 6;

    let mut events = Vec::new();
    let mut on_event = |e: AgentEvent| events.push(e);
    let outcome = run_agent(&llm, executor.clone(), &config, &mut on_event)
        .await
        .expect("agent session");
    executor.shutdown().await.ok();

    let session = std::fs::read_to_string(&outcome.session_path).unwrap();

    // 1. The primary's read must ADVERTISE the chain, or the agent has no way
    //    to discover that the decision lives elsewhere.
    assert!(
        session.contains("decisions.hick") && session.contains("domain.hick"),
        "read_doc must name the upstream documents:\n{session}"
    );

    // 2. Reading two hops up by bare file name must work.
    assert!(
        session.contains("Records are retained for 30 days"),
        "read_doc doc=decisions.hick did not return the decision:\n{session}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn an_upstream_edit_rewrites_the_source_and_reaches_the_primary() {
    let dir = tempfile::tempdir().unwrap();
    let path = |n: &str| dir.path().join(n);
    std::fs::write(path("decisions.hick"), DECISIONS).unwrap();
    std::fs::write(path("domain.hick"), DOMAIN).unwrap();
    std::fs::write(path("requirements.hick"), REQUIREMENTS).unwrap();

    // Address the line by its own content hash, computed the same way the
    // session renders it, so the test does not depend on line numbers.
    let hash = hickory_agent::hashline::LineIndex::new(DECISIONS)
        .render()
        .lines()
        .find(|l| l.contains("retained for 30 days"))
        .and_then(|l| l.split('|').next())
        .expect("hashed line")
        .to_string();

    let llm = ScriptedLlmClient::new(vec![
        format!(
            "<hick:next>tool</hick:next>\nFixing the decision at its source.\n\
             <hick:tool name=\"edit_doc\">\n\
             <hick:arg name=\"doc\">decisions.hick</hick:arg>\n\
             <hick:arg name=\"run\">{hash}</hick:arg>\n\
             <hick:input>Records are retained for 90 days.</hick:input>\n</hick:tool>"
        ),
        "<hick:next>done</hick:next>\nDone.".to_string(),
    ]);

    let executor: Arc<dyn Executor> = Arc::new(LocalExecutor::new().unwrap());
    let mut config = AgentConfig::new("Retention should be 90 days.", dir.path());
    config.doc_path = Some(path("requirements.hick"));
    config.max_turns = 4;

    let mut on_event = |_: AgentEvent| {};
    run_agent(&llm, executor.clone(), &config, &mut on_event)
        .await
        .expect("agent session");
    executor.shutdown().await.ok();

    // The upstream SOURCE changed on disk — one place, not a local restatement.
    let decisions = std::fs::read_to_string(path("decisions.hick")).unwrap();
    assert!(
        decisions.contains("90 days"),
        "the decision document was not edited:\n{decisions}"
    );

    // And nothing was restated downstream: the requirement still only
    // references the fragment.
    let requirements = std::fs::read_to_string(path("requirements.hick")).unwrap();
    assert!(
        !requirements.contains("90 days") && !requirements.contains("30 days"),
        "the fact was copied downstream instead of referenced:\n{requirements}"
    );
}

/// `verify` must cover everything the agent can edit.
///
/// Guarantee: docs/guarantees/agent/verify-covers-the-whole-editable-set.md
///
/// Observed live: an agent edited a decision two hops upstream, ran verify,
/// got PASS, and reported success in good faith — while `hick test`
/// failed on six documents whose outputs had never been re-woven. A feedback
/// loop narrower than the edit scope does not just miss problems, it actively
/// certifies them.
#[tokio::test(flavor = "multi_thread")]
async fn verify_reweaves_upstream_outputs_too() {
    let dir = tempfile::tempdir().unwrap();
    let path = |n: &str| dir.path().join(n);
    std::fs::write(path("decisions.hick"), DECISIONS).unwrap();
    std::fs::write(path("domain.hick"), DOMAIN).unwrap();
    std::fs::write(path("requirements.hick"), REQUIREMENTS).unwrap();

    let hash = hickory_agent::hashline::LineIndex::new(DECISIONS)
        .render()
        .lines()
        .find(|l| l.contains("retained for 30 days"))
        .and_then(|l| l.split('|').next())
        .expect("hashed line")
        .to_string();

    let llm = ScriptedLlmClient::new(vec![
        format!(
            "<hick:next>tool</hick:next>\nFixing it at the source.\n\
             <hick:tool name=\"edit_doc\">\n\
             <hick:arg name=\"doc\">decisions.hick</hick:arg>\n\
             <hick:arg name=\"run\">{hash}</hick:arg>\n\
             <hick:input>Records are retained for 90 days.</hick:input>\n</hick:tool>"
        ),
        "<hick:next>tool</hick:next>\nVerifying.\n\
         <hick:tool name=\"verify\">\n</hick:tool>"
            .to_string(),
        "<hick:next>done</hick:next>\nDone.".to_string(),
    ]);

    let executor: Arc<dyn Executor> = Arc::new(LocalExecutor::new().unwrap());
    let mut config = AgentConfig::new("Retention should be 90 days.", dir.path());
    config.doc_path = Some(path("requirements.hick"));
    config.max_turns = 6;

    let mut on_event = |_: AgentEvent| {};
    let outcome = run_agent(&llm, executor.clone(), &config, &mut on_event)
        .await
        .expect("agent session");
    executor.shutdown().await.ok();

    let session = std::fs::read_to_string(&outcome.session_path).unwrap();
    assert!(
        session.contains("re-wove") && session.contains("upstream document"),
        "verify did not report re-weaving upstream:\n{session}"
    );

    // The actual point: every upstream document's committed output is now
    // current, so a `hick test` over the tree would pass.
    for name in ["decisions.md", "domain.md"] {
        let woven = std::fs::read_to_string(path(name))
            .unwrap_or_else(|e| panic!("{name} was never written: {e}"));
        assert!(
            woven.contains("90 days"),
            "{name} still carries the old decision — verify left it stale:\n{woven}"
        );
    }
}
