<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/agent_cell_vertex.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-932eec9d0689"></a>
## `crates/hickory-cli/tests/agent_cell_vertex.rs`

Related contracts: [An Agent Cell Is A DAG Vertex, And Its Edges Are A Barrier](../../../../guarantees/agent/an-agent-cell-is-a-dag-barrier/contract.md), [An Agent Recording Is Keyed By The Prompt And The Model](../../../../guarantees/agent/an-agent-recording-is-keyed-by-prompt-and-model/contract.md), [Re-Preparation After An Agent Edit Terminates, Under A Declared Bound](../../../../guarantees/agent/re-preparation-terminates/contract.md), [`hick test` Never Spends The Reader's Tokens](../../../../guarantees/agent/test-never-spends-tokens/contract.md).

<a id="f-932eec9d0689-MODEL"></a>
### `MODEL`

The `hick:agent` DAG vertex driven by the **real** agent loop.
Guarantees protected here:
- `docs/guarantees/agent/an-agent-cell-is-a-dag-barrier.md`
- `docs/guarantees/agent/an-agent-recording-is-keyed-by-prompt-and-model.md`
- `docs/guarantees/agent/test-never-spends-tokens.md`
**Zero API calls**: every turn comes from `ScriptedLlmClient`. This is the
companion to `crates/hick-literate/tests/agent_cells.rs`, which covers the
same vertex with a stub runner; here the runner is the shipped
`LlmAgentRunner` over the Python process supervisor. The recording case uses
an unchanged workspace and a real verified artifact, then replays without a model.

<storycode:copy id="f-932eec9d0689-MODEL">
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
//! `LlmAgentRunner` over the Python process supervisor. The recording case uses
//! an unchanged workspace and a real verified artifact, then replays without a model.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use hick_literate::agent_cell::AgentRunner;
use hick_literate::{CellId, LocalExecutor, NoBaseline, PipelineConfig, cache, run_pipeline_live};
use hickory_agent::{LlmClient, ScriptedLlmClient};
use hickory_cli::{
    CheckFailure, CheckOutcome, ExecutorChoice, LlmAgentRunner, RunMode, check_failures,
    check_outcome, run_doc, unverifiable_message,
};

const MODEL: &str = "claude-sonnet-5";
</storycode:copy>
<a id="f-932eec9d0689-ANCHOR"></a>
### `ANCHOR`

<storycode:copy id="f-932eec9d0689-ANCHOR">
const ANCHOR: &str = "The agent writes below this line.";

</storycode:copy>
<a id="f-932eec9d0689-doc_source"></a>
### `doc_source`

<storycode:copy id="f-932eec9d0689-doc_source">
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

</storycode:copy>
<a id="f-932eec9d0689-scripted"></a>
### `scripted`

One `edit_doc` turn, then `done` — the smallest run that proves the write
channel and the settle.

<storycode:copy id="f-932eec9d0689-scripted">
/// One `edit_doc` turn, then `done` — the smallest run that proves the write
/// channel and the settle.
fn scripted() -> Arc<dyn LlmClient> {
    Arc::new(ScriptedLlmClient::new([
        serde_json::json!({"python":r#"import json
s = json.loads(tool('changeset_status', {})['content'][0]['text'])
tool('submit_changeset', {'boundary':s['boundary'], 'expected':s['expected'], 'entry':s['entry'], 'sources':{s['entry']:s['template']}})"#}).to_string(),
        serde_json::json!({"done":"Recorded unchanged fixture"}).to_string(),
    ]).with_model_name(MODEL))
}

</storycode:copy>
<a id="f-932eec9d0689-write_doc"></a>
### `write_doc`

<storycode:copy id="f-932eec9d0689-write_doc">
fn write_doc(dir: &Path, body: &str) -> PathBuf {
    let path = dir.join("doc.md");
    std::fs::write(&path, body).unwrap();
    path
}

</storycode:copy>
<a id="f-932eec9d0689-run_with_runner"></a>
### `run_with_runner`

<storycode:copy id="f-932eec9d0689-run_with_runner">
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

</storycode:copy>
<a id="f-932eec9d0689-a_cell_naming_another_model_is_refused"></a>
### `a_cell_naming_another_model_is_refused`

A model mismatch is refused before launching Python or buying a request.

<storycode:copy id="f-932eec9d0689-a_cell_naming_another_model_is_refused">
/// A model mismatch is refused before launching Python or buying a request.
#[tokio::test(flavor = "multi_thread")]
async fn a_cell_naming_another_model_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(
        dir.path(),
        &doc_source().replace(&format!("model=\"{MODEL}\""), "model=\"gpt-5\""),
    );
    let runner: Arc<dyn AgentRunner> = Arc::new(LlmAgentRunner::new(
        scripted(),
        PathBuf::from("/unused-model-mismatch"),
    ));

    let err = match run_with_runner(&doc, Some(runner), None, false).await {
        Ok(_) => panic!("a model mismatch must not be silently substituted"),
        Err(e) => format!("{e:#}"),
    };
    assert!(err.contains("gpt-5"), "{err}");
    assert!(err.contains(MODEL), "{err}");
}

</storycode:copy>
<a id="f-932eec9d0689-test_reports_an_agent_cell_with_no_baseline_as_unverifiable"></a>
### `test_reports_an_agent_cell_with_no_baseline_as_unverifiable`

`hick test` on a document whose agent cell has no baseline: unverifiable
(exit 2), named without a container, with a message that says what to do.
This is hermetic on purpose — `run_doc` in `RunMode::Verify` attaches no
agent runner at all, even on a machine holding an API key.

<storycode:copy id="f-932eec9d0689-test_reports_an_agent_cell_with_no_baseline_as_unverifiable">
/// `hick test` on a document whose agent cell has no baseline: unverifiable
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
        .expect("an agent cell with no baseline must not abort `hick test`");

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
    assert!(msg.contains("hick run --cache"), "{msg}");
    assert!(
        msg.contains("write the greeting"),
        "the message should name the prompt: {msg}"
    );
}

</storycode:copy>
<a id="f-932eec9d0689-a_recorded_agent_cell_verifies_without_a_model"></a>
### `a_recorded_agent_cell_verifies_without_a_model`

`hick test` never spends a token: even where a runner exists, the verify
path does not get one, so a cell either replays from its recording or is
reported. Here the recording is written by a `hick run`-shaped pass and
the verify pass consumes it with no LLM in sight.

<storycode:copy id="f-932eec9d0689-a_recorded_agent_cell_verifies_without_a_model">
/// `hick test` never spends a token: even where a runner exists, the verify
/// path does not get one, so a cell either replays from its recording or is
/// reported. Here the recording is written by a `hick run`-shaped pass and
/// the verify pass consumes it with no LLM in sight.
#[tokio::test(flavor = "multi_thread")]
async fn a_recorded_agent_cell_verifies_without_a_model() {
    let Some(python) = std::env::var_os("HICKORY_AGENT_TEST_PYTHON") else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), &doc_source());
    let cc = cache::CacheConfig::new(dir.path(), cache::CacheMode::Reuse);
    let runner: Arc<dyn AgentRunner> =
        Arc::new(LlmAgentRunner::new(scripted(), PathBuf::from(python)));
    run_with_runner(&doc, Some(runner), Some(&cc), false)
        .await
        .unwrap();
    assert!(cc.cache_dir.is_dir());

    // Freeze the recorded cell so the second pass is a pure replay without a model.
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
</storycode:copy>

### Assemble `crates/hickory-cli/tests/agent_cell_vertex.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/agent_cell_vertex.rs"><storycode:paste select="#f-932eec9d0689-MODEL"/><storycode:paste select="#f-932eec9d0689-ANCHOR"/><storycode:paste select="#f-932eec9d0689-doc_source"/><storycode:paste select="#f-932eec9d0689-scripted"/><storycode:paste select="#f-932eec9d0689-write_doc"/><storycode:paste select="#f-932eec9d0689-run_with_runner"/><storycode:paste select="#f-932eec9d0689-a_cell_naming_another_model_is_refused"/><storycode:paste select="#f-932eec9d0689-test_reports_an_agent_cell_with_no_baseline_as_unverifiable"/><storycode:paste select="#f-932eec9d0689-a_recorded_agent_cell_verifies_without_a_model"/></storycode:file>

</storycode:doc>
