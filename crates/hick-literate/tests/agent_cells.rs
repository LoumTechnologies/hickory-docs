//! `<hick:agent>` as a DAG vertex scheduled by the topological loop.
//!
//! Guarantees protected here:
//! - `docs/guarantees/agent/an-agent-cell-is-a-dag-barrier.md`
//! - `docs/guarantees/agent/an-agent-recording-is-keyed-by-prompt-and-model.md`
//! - `docs/guarantees/agent/re-preparation-terminates.md`
//!
//! **Zero API calls.** The agent's reasoning is out of scope here: what is
//! under test is the *vertex* — that it is scheduled, ordered, recorded,
//! frozen, and bounded — so the runner is a stub that performs a scripted
//! edit. `crates/hickory-cli/tests/agent_cell_vertex.rs` covers the real
//! runner with a `ScriptedLlmClient`.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use hick_literate::agent_cell::{AgentRequest, AgentRunOutcome, AgentRunner};
use hick_literate::{
    CellId, LocalExecutor, NoBaseline, PipelineConfig, PipelineResult, cache, run_pipeline_live,
};

const MODEL: &str = "scripted-model-1";

/// A runner that never calls a model: it appends a fixed edit to the document
/// and settles. Enough to prove the vertex, which is what this issue is about.
struct StubRunner {
    /// Text appended to the document, just before `</hick:doc>`, each time an
    /// agent cell runs. `None` means "settle without editing".
    edit: Option<String>,
    /// Every prompt this runner was handed, in scheduling order.
    seen: Arc<Mutex<Vec<String>>>,
    model: String,
}

impl StubRunner {
    fn new(edit: Option<&str>) -> Self {
        Self {
            edit: edit.map(str::to_string),
            seen: Arc::new(Mutex::new(Vec::new())),
            model: MODEL.to_string(),
        }
    }
}

#[async_trait]
impl AgentRunner for StubRunner {
    fn model_name(&self) -> &str {
        &self.model
    }

    async fn run(&self, request: AgentRequest) -> anyhow::Result<AgentRunOutcome> {
        self.seen
            .lock()
            .unwrap()
            .push(request.prompt.trim().to_string());
        let mut edited = false;
        if let Some(edit) = &self.edit {
            let src = std::fs::read_to_string(&request.doc_path)?;
            let updated = src.replace("</hick:doc>", &format!("{edit}\n</hick:doc>"));
            std::fs::write(&request.doc_path, updated)?;
            edited = true;
        }
        Ok(AgentRunOutcome {
            summary: "settled".to_string(),
            model: self.model.clone(),
            edited_source: edited,
            session_path: None,
        })
    }
}

fn write_doc(dir: &Path, body: &str) -> PathBuf {
    let path = dir.join("doc.hick");
    std::fs::write(&path, body).unwrap();
    path
}

async fn run(
    doc: &Path,
    runner: Option<Arc<dyn AgentRunner>>,
    cache_config: Option<&cache::CacheConfig>,
    collect_unverifiable: bool,
) -> anyhow::Result<PipelineResult> {
    let source = std::fs::read_to_string(doc).unwrap();
    let name = doc.display().to_string();
    let dir = doc.parent().unwrap().to_path_buf();
    let config = PipelineConfig {
        working_dir: Some(dir),
        max_rounds: 1,
        on_exec: None,
        collect_unverifiable,
        agent_runner: runner,
        max_agent_reprepares: 0,
    };
    run_pipeline_live(
        &[(name.as_str(), source.as_str())],
        &config,
        &[],
        cache_config,
        Arc::new(LocalExecutor::new().unwrap()),
    )
    .await
}

fn doc_with_agent_between_two_execs() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
<hick:container name="before" image="host" />
<hick:container name="after" image="host" />
<hick:exec container="before">printf 'first\n'</hick:exec>
<hick:agent id="a1" model="scripted-model-1"><hick:prompt>do the thing</hick:prompt></hick:agent>
<hick:exec container="after">printf 'last\n'</hick:exec>
</hick:doc>"#
}

/// The vertex is scheduled by the topological loop, between the cells the
/// barrier orders it against — not before the run and not after it.
#[tokio::test(flavor = "multi_thread")]
async fn an_agent_vertex_runs_from_inside_the_topological_loop() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), doc_with_agent_between_two_execs());
    let runner = Arc::new(StubRunner::new(None));
    let seen = runner.seen.clone();

    let result = run(&doc, Some(runner), None, false).await.unwrap();

    assert_eq!(
        seen.lock().unwrap().as_slice(),
        ["do the thing"],
        "the agent cell ran exactly once, with its own prompt"
    );
    // Its answer is a transcript entry like any other cell's, under the
    // reserved synthetic container name.
    let agent_output: Vec<&str> = result
        .transcripts
        .iter()
        .filter(|(container, _)| hick_exec::is_agent_container(container))
        .flat_map(|(_, entries)| entries.iter().map(|e| e.output.as_str()))
        .collect();
    assert_eq!(agent_output, ["settled"]);
    // Both surrounding execs still ran: the barrier orders, it does not block.
    assert!(result.transcripts["before"][0].output.contains("first"));
    assert!(result.transcripts["after"][0].output.contains("last"));
}

/// The property the placement spike found decisive in the other direction: an
/// exec the agent *writes* runs in the same pass, because the re-preparation
/// happens at the agent's barrier and every remaining cell is behind it.
#[tokio::test(flavor = "multi_thread")]
async fn an_exec_the_agent_writes_runs_in_the_same_pass() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(
        dir.path(),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
<hick:agent id="a1" model="scripted-model-1"><hick:prompt>write a cell</hick:prompt></hick:agent>
</hick:doc>"#,
    );
    let runner = Arc::new(StubRunner::new(Some(
        r#"<hick:container name="authored" image="host" />
<hick:exec container="authored">printf 'from the agent\n'</hick:exec>"#,
    )));

    let result = run(&doc, Some(runner), None, false).await.unwrap();

    let authored = result
        .transcripts
        .get("authored")
        .expect("the exec the agent authored should have run in this same pass");
    assert!(authored[0].output.contains("from the agent"));
}

/// No provider key is the ordinary state of CI. The cell is *unverifiable*,
/// the rest of the document still runs, and the report names it without a
/// container — because it has none.
#[tokio::test(flavor = "multi_thread")]
async fn an_agent_cell_without_a_runner_is_unverifiable_not_fatal() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), doc_with_agent_between_two_execs());

    let result = run(&doc, None, None, true).await.unwrap();

    let cell = CellId::containerless(6);
    assert_eq!(
        result.never_run.keys().collect::<Vec<_>>(),
        vec![&cell],
        "exactly the agent cell has no baseline, and it is keyed containerless"
    );
    assert!(matches!(
        result.never_run[&cell],
        NoBaseline::AgentWithoutRunner {
            model_declared: true,
            ..
        }
    ));
    assert!(
        result.transcripts["after"][0].output.contains("last"),
        "the blast radius is the cell, not the document"
    );
}

/// A recorded agent cell replays with no runner at all — which is the whole
/// point of putting the model in the key: the document says which model
/// answered, so a machine with no credentials can still check it.
#[tokio::test(flavor = "multi_thread")]
async fn a_recorded_agent_cell_replays_without_a_runner() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), doc_with_agent_between_two_execs());
    let cc = cache::CacheConfig::new(dir.path(), true, false);

    let runner = Arc::new(StubRunner::new(None));
    run(&doc, Some(runner), Some(&cc), false).await.unwrap();

    // Freeze the cell and run again with NO runner.
    let frozen = std::fs::read_to_string(&doc).unwrap().replace(
        "<hick:agent id=\"a1\"",
        "<hick:agent id=\"a1\" freeze=\"true\"",
    );
    std::fs::write(&doc, frozen).unwrap();
    let result = run(&doc, None, Some(&cc), true).await.unwrap();

    assert!(
        result.never_run.is_empty(),
        "a recording IS a baseline: {:?}",
        result.never_run
    );
    let replayed: Vec<&str> = result
        .transcripts
        .iter()
        .filter(|(container, _)| hick_exec::is_agent_container(container))
        .flat_map(|(_, entries)| entries.iter().map(|e| e.output.as_str()))
        .collect();
    assert_eq!(replayed, ["settled"]);
}

/// Editing the prompt retires the recording. Without this, freeze would serve
/// the old answer to a question the document no longer asks.
#[tokio::test(flavor = "multi_thread")]
async fn a_changed_prompt_retires_the_recording() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), doc_with_agent_between_two_execs());
    let cc = cache::CacheConfig::new(dir.path(), true, false);
    run(
        &doc,
        Some(Arc::new(StubRunner::new(None))),
        Some(&cc),
        false,
    )
    .await
    .unwrap();

    let edited = std::fs::read_to_string(&doc)
        .unwrap()
        .replace(
            "<hick:agent id=\"a1\"",
            "<hick:agent id=\"a1\" freeze=\"true\"",
        )
        .replace("do the thing", "do a DIFFERENT thing");
    std::fs::write(&doc, edited).unwrap();

    let result = run(&doc, None, Some(&cc), true).await.unwrap();
    let cell = CellId::containerless(6);
    assert!(
        matches!(
            result.never_run.get(&cell),
            Some(NoBaseline::FrozenWithoutRecording { .. })
        ),
        "the recording must not answer a prompt it was not made for: {:?}",
        result.never_run
    );
}

/// A document whose agent writes another agent cell has no fixed point. It
/// fails at a declared bound rather than re-preparing forever — the same class
/// of invariant as `max_turns`.
#[tokio::test(flavor = "multi_thread")]
async fn re_preparation_terminates_at_its_declared_bound() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(
        dir.path(),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
<hick:agent id="a1" model="scripted-model-1"><hick:prompt>spawn another</hick:prompt></hick:agent>
</hick:doc>"#,
    );
    // Each run appends one MORE agent cell: the pathological case.
    let runner = Arc::new(StubRunner::new(Some(
        "<hick:agent><hick:prompt>spawn another</hick:prompt></hick:agent>",
    )));

    let err = match run(&doc, Some(runner), None, false).await {
        Ok(_) => panic!("a document with no fixed point must fail, not loop"),
        Err(e) => e,
    };
    let msg = format!("{err:#}");
    assert!(msg.contains("re-prepared"), "{msg}");
    assert!(msg.contains("fixed point"), "{msg}");
    assert!(
        msg.contains("max-turns"),
        "the message should name the invariant class: {msg}"
    );
}

/// `hick:expect` on an agent cell: the key that used to be
/// `(container, exec_line)` now names a cell with no container at all.
#[tokio::test(flavor = "multi_thread")]
async fn an_expectation_on_an_agent_cell_is_evaluated() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(
        dir.path(),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
<hick:agent id="a1" model="scripted-model-1"><hick:prompt>do the thing</hick:prompt>
<hick:expect>settled</hick:expect>
</hick:agent>
</hick:doc>"#,
    );

    let result = run(&doc, Some(Arc::new(StubRunner::new(None))), None, false)
        .await
        .unwrap();

    assert_eq!(result.expectations.len(), 1);
    let outcome = &result.expectations[0];
    assert!(outcome.passed, "{}", outcome.detail);
    assert_eq!(
        outcome.container, None,
        "an agent cell's expectation names no container, because it has none"
    );
}
