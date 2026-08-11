//! The `hick:agent` DAG vertex driven by the **real** agent loop.
//!
//! Guarantees protected here:
//! - `docs/guarantees/agent/an-agent-cell-is-a-dag-barrier.md`
//! - `docs/guarantees/agent/an-agent-recording-is-keyed-by-prompt-and-model.md`
//! - `docs/guarantees/agent/test-never-spends-tokens.md`
//!
//! **Zero API calls**: every turn comes from `ScriptedLlmClient`. This is the
//! companion to `crates/hick-literate/tests/agent_cells.rs`, which covers the
//! same vertex with a stub runner; here the runner is the shipped
//! `LlmAgentRunner` over the shipped ReAct loop, so the `edit_doc`-only write
//! channel is exercised for real.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use hick_literate::agent_cell::AgentRunner;
use hick_literate::{CellId, LocalExecutor, NoBaseline, PipelineConfig, cache, run_pipeline_live};
use hickory_agent::{LlmClient, ScriptedLlmClient, hashline};
use hickory_cli::{
    CheckFailure, CheckOutcome, ExecutorChoice, LlmAgentRunner, RunMode, check_failures,
    check_outcome, run_doc, unverifiable_message,
};

const MODEL: &str = "claude-sonnet-5";
const ANCHOR: &str = "The agent writes below this line.";
const PAYLOAD: &str = "<hick:file path=\"greeting.txt\">\nhello from the agent\n</hick:file>";

fn doc_source() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
# Agent vertex

{ANCHOR}

<hick:agent id="a1" model="{MODEL}" max-turns="4">
<hick:prompt>write the greeting</hick:prompt>
</hick:agent>
</hick:doc>
"#
    )
}

/// One `edit_doc` turn, then `done` — the smallest run that proves the write
/// channel and the settle.
fn scripted() -> Arc<dyn LlmClient> {
    let anchor = hashline::line_hash(ANCHOR);
    let tool_turn = format!(
        "<hick:next>tool</hick:next>\n\
         <hick:tool name=\"edit_doc\">\n\
         <hick:arg name=\"after\">{anchor}</hick:arg>\n\
         <hick:input>\n{PAYLOAD}\n</hick:input>\n\
         </hick:tool>"
    );
    Arc::new(
        ScriptedLlmClient::new([
            tool_turn,
            "<hick:next>done</hick:next>\nWrote greeting.txt through the document.".to_string(),
        ])
        .with_model_name(MODEL),
    )
}

fn write_doc(dir: &Path, body: &str) -> PathBuf {
    let path = dir.join("doc.hick");
    std::fs::write(&path, body).unwrap();
    path
}

async fn run_with_runner(
    doc: &Path,
    runner: Option<Arc<dyn AgentRunner>>,
    cache_config: Option<&cache::CacheConfig>,
    collect_unverifiable: bool,
) -> anyhow::Result<hick_literate::PipelineResult> {
    let source = std::fs::read_to_string(doc).unwrap();
    let name = doc.display().to_string();
    run_pipeline_live(
        &[(name.as_str(), source.as_str())],
        &PipelineConfig {
            working_dir: Some(doc.parent().unwrap().to_path_buf()),
            max_rounds: 1,
            collect_unverifiable,
            agent_runner: runner,
            ..Default::default()
        },
        &[],
        cache_config,
        Arc::new(LocalExecutor::new().unwrap()),
    )
    .await
}

/// The shipped runner settles the vertex from inside the topological loop, and
/// its one write channel — `edit_doc` — leaves the bytes in the document.
#[tokio::test(flavor = "multi_thread")]
async fn the_shipped_runner_settles_the_vertex_and_its_edit_lands_in_the_document() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), &doc_source());
    let runner: Arc<dyn AgentRunner> = Arc::new(LlmAgentRunner::new(
        scripted(),
        Arc::new(LocalExecutor::new().unwrap()),
    ));

    let result = run_with_runner(&doc, Some(runner), None, false)
        .await
        .unwrap();

    let after = std::fs::read_to_string(&doc).unwrap();
    assert!(
        after.contains("hello from the agent"),
        "the agent's only write channel is the document: {after}"
    );
    // The re-preparation put the agent's `hick:file` into the same pass, so
    // the woven output carries it without a second run.
    assert!(
        result.files.contains_key("greeting.txt"),
        "outputs: {:?}",
        result.files.keys().collect::<Vec<_>>()
    );
    let summary: Vec<&str> = result
        .transcripts
        .iter()
        .filter(|(c, _)| hick_exec::is_agent_container(c))
        .flat_map(|(_, e)| e.iter().map(|e| e.output.as_str()))
        .collect();
    assert_eq!(summary, ["Wrote greeting.txt through the document."]);
}

/// A cell that names a model the run is not configured for fails loudly rather
/// than answering with a different model — the recording is keyed by the
/// model, so a substitution files the answer under a key nobody asks for.
#[tokio::test(flavor = "multi_thread")]
async fn a_cell_naming_another_model_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(
        dir.path(),
        &doc_source().replace(&format!("model=\"{MODEL}\""), "model=\"gpt-5\""),
    );
    let runner: Arc<dyn AgentRunner> = Arc::new(LlmAgentRunner::new(
        scripted(),
        Arc::new(LocalExecutor::new().unwrap()),
    ));

    let err = match run_with_runner(&doc, Some(runner), None, false).await {
        Ok(_) => panic!("a model mismatch must not be silently substituted"),
        Err(e) => format!("{e:#}"),
    };
    assert!(err.contains("gpt-5"), "{err}");
    assert!(err.contains(MODEL), "{err}");
}

/// `hickory test` on a document whose agent cell has no baseline: unverifiable
/// (exit 2), named without a container, with a message that says what to do.
///
/// This is hermetic on purpose — `run_doc` in `RunMode::Verify` attaches no
/// agent runner at all, even on a machine holding an API key.
#[tokio::test(flavor = "multi_thread")]
async fn test_reports_an_agent_cell_with_no_baseline_as_unverifiable() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), &doc_source());

    let run = run_doc(&doc, &[], RunMode::Verify, ExecutorChoice::Local)
        .await
        .expect("an agent cell with no baseline must not abort `hickory test`");

    let cell = CellId::containerless(7);
    assert_eq!(
        run.result.never_run.keys().collect::<Vec<_>>(),
        vec![&cell],
        "the agent cell is the one cell with no baseline"
    );
    assert!(matches!(
        run.result.never_run[&cell],
        NoBaseline::AgentWithoutRunner { .. }
    ));

    let failures = check_failures(&run, None).unwrap();
    assert_eq!(check_outcome(&failures), CheckOutcome::Unverifiable);
    assert_eq!(CheckOutcome::Unverifiable.exit_code(), 2);

    let CheckFailure::Unverifiable { cell, reason, .. } = &failures[0] else {
        panic!("expected an unverifiable failure, got {:?}", failures[0]);
    };
    let msg = unverifiable_message(&doc, cell, reason);
    assert!(msg.starts_with("UNVERIFIABLE"), "{msg}");
    assert!(msg.contains("line 7"), "{msg}");
    assert!(
        !msg.contains("container"),
        "an agent cell has no container to name: {msg}"
    );
    assert!(msg.contains("hickory run --cache"), "{msg}");
    assert!(
        msg.contains("write the greeting"),
        "the message should name the prompt: {msg}"
    );
}

/// `hickory test` never spends a token: even where a runner exists, the verify
/// path does not get one, so a cell either replays from its recording or is
/// reported. Here the recording is written by a `hickory run`-shaped pass and
/// the verify pass consumes it with no LLM in sight.
#[tokio::test(flavor = "multi_thread")]
async fn a_recorded_agent_cell_verifies_without_a_model() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), &doc_source());
    let cc = cache::CacheConfig::new(dir.path(), cache::CacheMode::Reuse);
    let runner: Arc<dyn AgentRunner> = Arc::new(LlmAgentRunner::new(
        scripted(),
        Arc::new(LocalExecutor::new().unwrap()),
    ));
    run_with_runner(&doc, Some(runner), Some(&cc), false)
        .await
        .unwrap();
    assert!(cc.cache_dir.is_dir());

    // The agent edited the document; freeze the cell so the second pass is a
    // pure replay, and run it with no runner at all.
    let frozen = std::fs::read_to_string(&doc).unwrap().replace(
        "<hick:agent id=\"a1\"",
        "<hick:agent id=\"a1\" freeze=\"true\"",
    );
    std::fs::write(&doc, frozen).unwrap();

    let result = run_with_runner(&doc, None, Some(&cc), true).await.unwrap();
    assert!(
        result.never_run.is_empty(),
        "a recorded agent cell has a baseline: {:?}",
        result.never_run
    );
}
