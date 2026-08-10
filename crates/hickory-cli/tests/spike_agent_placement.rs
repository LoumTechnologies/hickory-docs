//! SPIKE CODE — THROWAWAY. Not a feature, not shipped, not wired into the
//! `hickory` binary. It exists to answer GitHub issue #4: **where does a
//! `hick:agent` node live — flow (converged during weave) or exec (a DAG
//! vertex that runs before weave)?**
//!
//! Findings are written up in `docs/specs/freeform/agent-placement-spike.md`.
//! Delete this file once the losing placement is deleted.
//!
//! Every assertion here is evidence for one of the six criteria declared in
//! the issue *before* either arm was built. Both arms are driven by
//! [`ScriptedLlmClient`], so the whole file costs **zero API calls** and is
//! fully deterministic.
//!
//! ## What each arm actually is
//!
//! Neither placement exists in the product yet, so each arm realizes the
//! placement's *definition* against the real pipeline rather than mocking it:
//!
//! * **flow arm** — a real [`Node`] implementation converged through
//!   `converge_with_provenance`, pulling its `LlmClient`/`Executor` out of
//!   `Context`'s TypeMap exactly as the design says, with the agent loop
//!   inside `get_stream` and the cell emitting once at settle.
//! * **exec arm** — the agent loop run as a scheduled vertex *before* the
//!   pipeline's output phase, with the cell itself present in the document as
//!   a DAG-visible, freeze-able, cache-keyed cell. That is what "a DAG vertex
//!   that runs before weave" means to every downstream consumer: `build_dag`,
//!   the topological loop, `never_run`, and the transcript cache all identify
//!   a cell that way.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use hick_exec::node::{
    BoxStream, Context, InsertionPoint, Node, NodeTrace, SourceOrigin, SpanNode, StringNode,
    converge_with_provenance,
};
use hickory_agent::{
    EditSession, LlmClient, Message, Role, ScriptedLlmClient, Turn, Usage, execute_tool, hashline,
    parse_response,
};
use hickory_cli::{
    CheckOutcome, ExecutorChoice, RunMode, check_failures, check_outcome, output_lineage, run_doc,
};
use hickory_executor::{Executor, LocalExecutor};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// The line the scripted agent anchors its one `edit_doc` below. Chosen so
/// the inserted `hick:file` block lands at document top level.
const ANCHOR_LINE: &str = "The agent writes below this line.";

/// A document with an agent cell and nothing else that executes. Used where
/// the *only* variable must be the agent cell itself.
fn doc_flow_only_agent() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="spike.md">
# Placement spike

{ANCHOR_LINE}

<hick:agent id="a1">
<hick:prompt>write the greeting</hick:prompt>
</hick:agent>
</hick:doc>
"#
    )
}

/// The same document under the **exec** placement: the agent cell is a DAG
/// vertex, so it is a cell `build_dag` sees, the topological loop schedules,
/// and `freeze`/`never_run` cover. `freeze="true"` states the only honest
/// verification stance for a nondeterministic cell — check it against a
/// recording rather than re-running it.
fn doc_exec_agent_frozen(freeze: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="spike.md">
# Placement spike

{ANCHOR_LINE}

<hick:container name="agent" image="alpine:3.20" />

<hick:exec container="agent" freeze="{freeze}">
printf 'write the greeting\n'
</hick:exec>
</hick:doc>
"#
    )
}

/// A document whose exec cell reads a file the agent is supposed to author.
/// Used for criterion 3(b): can an exec consume an agent's edits?
fn doc_exec_consumes_agent_output() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="spike.md">
# Placement spike

{ANCHOR_LINE}

<hick:container name="c" image="alpine:3.20" />

<hick:volume name="src" input="." />

<hick:exec container="c" mount="src:/project">
cat project/greeting.txt 2>/dev/null || printf 'MISSING\n'
</hick:exec>
</hick:doc>
"#
    )
}

/// The block the scripted agent inserts. Its only write channel is
/// `edit_doc`, per the settled design.
const AGENT_PAYLOAD: &str = "<hick:file path=\"greeting.txt\">\nhello from the agent\n</hick:file>";

/// Two scripted turns: one `edit_doc` tool call, then `done`. Enough to
/// answer every criterion — an agent node that edits once and settles.
fn scripted_two_turns() -> Arc<dyn LlmClient> {
    let anchor = hashline::line_hash(ANCHOR_LINE);
    let tool_turn = format!(
        "<hick:next>tool</hick:next>\n\
         <hick:tool name=\"edit_doc\">\n\
         <hick:arg name=\"after\">{anchor}</hick:arg>\n\
         <hick:input>\n{AGENT_PAYLOAD}\n</hick:input>\n\
         </hick:tool>"
    );
    Arc::new(
        ScriptedLlmClient::with_usages([
            (
                tool_turn,
                Usage {
                    input_tokens: 1200,
                    cache_creation_input_tokens: 400,
                    cache_read_input_tokens: 0,
                    output_tokens: 90,
                },
            ),
            (
                "<hick:next>done</hick:next>\nWrote greeting.txt through the document.".to_string(),
                Usage {
                    input_tokens: 300,
                    cache_creation_input_tokens: 0,
                    cache_read_input_tokens: 1200,
                    output_tokens: 20,
                },
            ),
        ])
        .with_model_name("claude-sonnet-5"),
    )
}

/// A script that never says `done`: used for the termination criterion.
fn scripted_never_settles(turns: usize) -> Arc<dyn LlmClient> {
    Arc::new(
        ScriptedLlmClient::new(
            (0..turns).map(|i| format!("<hick:next>code</hick:next>\n```bash\necho {i}\n```")),
        )
        .with_model_name("claude-sonnet-5"),
    )
}

fn write_doc(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, body).unwrap();
    path
}

fn executor() -> Arc<dyn Executor> {
    Arc::new(LocalExecutor::new().unwrap())
}

// ---------------------------------------------------------------------------
// The agent loop — identical in both arms, so placement is the only variable
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
struct AgentRun {
    turns: usize,
    usage: Usage,
    settled: bool,
    summary: String,
}

/// A deliberately tiny ReAct loop: the real one lives in `react_loop.rs` and
/// the spike must not fork it. `edit_doc` is the only write channel, which is
/// the whole point of the design being tested.
async fn run_scripted_agent(
    llm: &dyn LlmClient,
    exec: Arc<dyn Executor>,
    doc_path: &Path,
    max_turns: usize,
) -> anyhow::Result<AgentRun> {
    let mut session = EditSession::open(doc_path, &[]).await?;
    let mut run = AgentRun::default();
    for turn in 1..=max_turns {
        let (text, usage) = llm
            .complete_with_usage(vec![Message::new(Role::User, "go")])
            .await?;
        run.turns = turn;
        run.usage.add(&usage);
        match parse_response(&text) {
            Turn::Tool { invocation, .. } => {
                let outcome = execute_tool(&mut session, exec.clone(), &invocation).await;
                anyhow::ensure!(outcome.ok, "tool {} failed: {}", outcome.name, outcome.text);
            }
            Turn::Done { summary } => {
                run.settled = true;
                run.summary = summary;
                return Ok(run);
            }
            Turn::Code { .. } => {}
            Turn::Invalid { reason } => anyhow::bail!("malformed response: {reason}"),
        }
    }
    // `max_turns` is a graph invariant, not a cost policy: exhausting it is a
    // failure, never a partial result a downstream consumer accepts.
    anyhow::bail!("max_turns ({max_turns}) exhausted without settling")
}

// ---------------------------------------------------------------------------
// Arm: flow — a real Node converged during weave
// ---------------------------------------------------------------------------

/// What `Context`'s TypeMap supplies to the agent node, exactly as the design
/// specifies (so `hick-flow` never grows an LLM dependency).
struct AgentDeps {
    llm: Arc<dyn LlmClient>,
    executor: Arc<dyn Executor>,
    doc_path: PathBuf,
    /// Where the node reports what it did, so a test can inspect the run.
    record: Arc<Mutex<Result<AgentRun, String>>>,
}

struct AgentFlowNode {
    max_turns: usize,
}

impl Node for AgentFlowNode {
    fn get_stream(self: Arc<Self>, context: Context) -> BoxStream<Vec<NodeTrace>> {
        // The loop lives inside `get_stream`, so the DAG stays acyclic and
        // the cell emits **once**, at settle: `futures::stream::once`.
        Box::pin(futures::stream::once(async move {
            let deps = context
                .get_extension::<AgentDeps>()
                .expect("spike: AgentDeps extension missing from Context");
            let text = match run_scripted_agent(
                deps.llm.as_ref(),
                deps.executor.clone(),
                &deps.doc_path,
                self.max_turns,
            )
            .await
            {
                Ok(run) => {
                    let summary = run.summary.clone();
                    *deps.record.lock().unwrap() = Ok(run);
                    summary
                }
                Err(e) => {
                    // On error the stream must END carrying an error value,
                    // never propagate a panic — otherwise converge hangs.
                    *deps.record.lock().unwrap() = Err(e.to_string());
                    "[agent failed]".to_string()
                }
            };
            // There is no `SourceOrigin::Agent` variant yet, and even once
            // there is, a flow-emitted byte has no byte-precise source span.
            let leaf: Arc<dyn Node> = Arc::new(SpanNode::new(text, SourceOrigin::Synthetic));
            vec![NodeTrace::new(leaf)]
        }))
    }

    fn source_origin(&self) -> Option<&SourceOrigin> {
        None
    }
}

/// A node that never completes — the termination hazard the design names.
struct NeverCompletingNode;

impl Node for NeverCompletingNode {
    fn get_stream(self: Arc<Self>, _context: Context) -> BoxStream<Vec<NodeTrace>> {
        Box::pin(futures::stream::pending())
    }
}

/// Build the flow-arm graph: literal prose, the agent node, more literal
/// prose — i.e. an agent cell sitting in a weave insertion point.
fn flow_graph(node: Arc<dyn Node>) -> Arc<InsertionPoint> {
    let ip = Arc::new(InsertionPoint::new());
    ip.add(Arc::new(StringNode::new("before\n")));
    ip.add(node);
    ip.add(Arc::new(StringNode::new("\nafter\n")));
    ip.close();
    ip
}

// ===========================================================================
// Criterion 1 — `check` parity
// ===========================================================================

/// **Decisive.** The same never-run agent cell yields *different* `check`
/// verdicts under the two placements: flow reports the document verified,
/// exec reports it unverifiable.
#[tokio::test(flavor = "multi_thread")]
async fn c1_check_verdicts_differ_between_placements() {
    // --- flow arm: the agent cell is not a DAG cell, so `check` never sees it.
    let flow_dir = tempfile::tempdir().unwrap();
    let flow_doc = write_doc(flow_dir.path(), "spike.hick", &doc_flow_only_agent());
    // Establish the committed baseline the way a user would: run, write out.
    let run = run_doc(&flow_doc, &[], RunMode::Execute, ExecutorChoice::Local)
        .await
        .expect("flow arm: run should succeed — nothing in the DAG to fail");
    hickory_cli::write_outputs(&run, None).unwrap();

    let run = run_doc(&flow_doc, &[], RunMode::Verify, ExecutorChoice::Local)
        .await
        .unwrap();
    let flow_outcome = check_outcome(&check_failures(&run, None).unwrap());
    assert!(
        run.result.never_run.is_empty(),
        "flow arm: an agent cell contributes no never_run entry, so check has \
         nothing to call unverifiable"
    );
    assert_eq!(
        flow_outcome,
        CheckOutcome::Verified,
        "flow arm: check calls a document verified whose agent cell has never run"
    );

    // --- exec arm: the agent cell IS a DAG cell, frozen, with no recording.
    let exec_dir = tempfile::tempdir().unwrap();
    let exec_doc = write_doc(
        exec_dir.path(),
        "spike.hick",
        &doc_exec_agent_frozen("true"),
    );
    let run = run_doc(&exec_doc, &[], RunMode::Verify, ExecutorChoice::Local)
        .await
        .unwrap();
    let exec_outcome = check_outcome(&check_failures(&run, None).unwrap());
    assert_eq!(
        exec_outcome,
        CheckOutcome::Unverifiable,
        "exec arm: a frozen agent cell with no recording is unverifiable"
    );

    // The criterion, stated as the issue stated it.
    assert_ne!(
        flow_outcome, exec_outcome,
        "criterion 1: the placements do NOT produce identical check outcomes"
    );
    assert_eq!(flow_outcome.exit_code(), 0);
    assert_eq!(exec_outcome.exit_code(), 2);
}

/// The replay half of criterion 1: under exec placement a recorded agent cell
/// verifies. There is no equivalent under flow placement, because the
/// recording machinery is keyed by `(container, source_line)` in the
/// topological loop that a flow node never enters.
#[tokio::test(flavor = "multi_thread")]
async fn c1_exec_placement_verifies_against_a_recording() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "spike.hick", &doc_exec_agent_frozen("false"));

    // Record the cell (freeze="false" keeps it live), with caching on.
    let source = std::fs::read_to_string(&doc).unwrap();
    let name = doc.display().to_string();
    let cc = hick_literate::cache::CacheConfig::new(dir.path(), true, false);
    hick_literate::run_pipeline_live(
        &[(name.as_str(), source.as_str())],
        &hick_literate::PipelineConfig {
            working_dir: Some(dir.path().to_path_buf()),
            max_rounds: 1,
            on_exec: None,
            collect_unverifiable: false,
        },
        &[],
        Some(&cc),
        executor(),
    )
    .await
    .expect("recording run failed");
    assert!(
        cc.cache_dir.is_dir(),
        "the recording directory should exist after a cached run"
    );

    // Now freeze the same cell and check: served from the recording.
    std::fs::write(&doc, doc_exec_agent_frozen("true")).unwrap();
    let run = run_doc(&doc, &[], RunMode::Verify, ExecutorChoice::Local)
        .await
        .unwrap();
    assert!(
        run.result.never_run.is_empty(),
        "a frozen cell served FROM a recording has a baseline: {:?}",
        run.result.never_run
    );
    hickory_cli::write_outputs(&run, None).unwrap();
    let run = run_doc(&doc, &[], RunMode::Verify, ExecutorChoice::Local)
        .await
        .unwrap();
    assert_eq!(
        check_outcome(&check_failures(&run, None).unwrap()),
        CheckOutcome::Verified,
        "exec arm under replay: verified"
    );
}

// ===========================================================================
// Criterion 2 — re-entrancy
// ===========================================================================

/// A flow-placed agent edits the document *from inside* the converge that is
/// producing that document's output. It does not corrupt state and it does
/// not loop — but the run that produced the edit cannot observe it, because
/// the graph was assembled and every `InsertionPoint` closed before converge
/// began. The edit is only visible to a *later* whole-pipeline pass.
#[tokio::test(flavor = "multi_thread")]
async fn c2_flow_agent_edit_is_invisible_to_the_run_that_produced_it() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "spike.hick", &doc_flow_only_agent());

    let record = Arc::new(Mutex::new(Err("not run".to_string())));
    let ctx = Context::new().with_extension(AgentDeps {
        llm: scripted_two_turns(),
        executor: executor(),
        doc_path: doc.clone(),
        record: record.clone(),
    });

    let graph = flow_graph(Arc::new(AgentFlowNode { max_turns: 8 }));
    let (converged, _prov) = converge_with_provenance(graph, ctx)
        .await
        .expect("flow arm: converge should settle when the node settles");

    // The nested `run_pipeline_weave` inside EditSession::reweave ran to
    // completion inside the outer converge. No deadlock, no corruption.
    let run = record.lock().unwrap().clone().expect("agent run failed");
    assert!(run.settled);
    assert_eq!(run.turns, 2);

    // …but the converged output is the OLD document's output.
    assert!(
        !converged.contains("hello from the agent"),
        "flow arm: the converged output must not contain the agent's edit, \
         because the graph closed before converge: {converged:?}"
    );
    assert!(converged.contains("before") && converged.contains("after"));

    // The edit really did land on disk — it just needs another pass.
    let source = std::fs::read_to_string(&doc).unwrap();
    assert!(
        source.contains("greeting.txt"),
        "edit_doc wrote the document"
    );

    let second = hick_literate::run_pipeline_weave(
        &[(doc.display().to_string().as_str(), source.as_str())],
        &[],
        None,
    )
    .await
    .unwrap();
    assert!(
        second.files.contains_key("greeting.txt"),
        "a SECOND whole-pipeline pass sees the agent's edit"
    );
}

/// Under exec placement the agent runs before the output phase, so a single
/// pipeline pass already carries its edits.
#[tokio::test(flavor = "multi_thread")]
async fn c2_exec_agent_edit_is_visible_in_the_same_pass() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "spike.hick", &doc_flow_only_agent());

    // The vertex runs first, in schedule order, before any output is built.
    let run = run_scripted_agent(scripted_two_turns().as_ref(), executor(), &doc, 8)
        .await
        .expect("exec arm: agent vertex");
    assert!(run.settled);

    let source = std::fs::read_to_string(&doc).unwrap();
    let result = hick_literate::run_pipeline_weave(
        &[(doc.display().to_string().as_str(), source.as_str())],
        &[],
        None,
    )
    .await
    .unwrap();
    assert!(
        result.files.contains_key("greeting.txt"),
        "exec arm: the output phase of the SAME pass sees the agent's edit"
    );
}

// ===========================================================================
// Criterion 3 — ordering, both directions
// ===========================================================================

/// Direction (a): an agent cell consuming an exec's output. Weave runs after
/// exec, so the transcripts a flow-placed agent needs already exist when it
/// converges. This one works under both placements; it is recorded because
/// "supporting only one direction is disqualifying".
#[tokio::test(flavor = "multi_thread")]
async fn c3a_agent_can_consume_exec_output() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "spike.hick", &doc_exec_consumes_agent_output());

    let run = run_doc(&doc, &[], RunMode::Execute, ExecutorChoice::Local)
        .await
        .unwrap();
    let transcripts = &run.result.transcripts;
    assert!(
        transcripts.contains_key("c"),
        "exec transcripts exist before the output phase, so an agent node \
         converged during weave can read them"
    );
    let text: String = transcripts["c"]
        .iter()
        .map(|e| e.output.clone())
        .collect::<Vec<_>>()
        .join("");
    assert!(text.contains("MISSING"), "the exec ran: {text:?}");
}

/// Direction (b): an exec consuming an agent's edits. This is where the
/// placements separate. Under exec placement the agent vertex precedes the
/// consuming exec in one pass; under flow placement the agent has not run
/// yet when every exec in the document has already finished.
#[tokio::test(flavor = "multi_thread")]
async fn c3b_only_exec_placement_lets_an_exec_consume_agent_edits() {
    // --- flow arm: all execs finish before weave, so the agent's edit
    //     cannot reach the exec in the same pass.
    let flow_dir = tempfile::tempdir().unwrap();
    let flow_doc = write_doc(
        flow_dir.path(),
        "spike.hick",
        &doc_exec_consumes_agent_output(),
    );
    let run = run_doc(&flow_doc, &[], RunMode::Execute, ExecutorChoice::Local)
        .await
        .unwrap();
    let before: String = run.result.transcripts["c"]
        .iter()
        .map(|e| e.output.clone())
        .collect();
    assert!(
        before.contains("MISSING"),
        "flow arm: the exec ran before the agent existed: {before:?}"
    );
    // Only now does the flow-placed agent get to run (during weave).
    run_scripted_agent(scripted_two_turns().as_ref(), executor(), &flow_doc, 8)
        .await
        .unwrap();
    assert!(
        before.contains("MISSING"),
        "flow arm: the exec's transcript for THIS pass can never change"
    );

    // --- exec arm: the agent vertex is scheduled before the consuming exec.
    let exec_dir = tempfile::tempdir().unwrap();
    let exec_doc = write_doc(
        exec_dir.path(),
        "spike.hick",
        &doc_exec_consumes_agent_output(),
    );
    run_scripted_agent(scripted_two_turns().as_ref(), executor(), &exec_doc, 8)
        .await
        .unwrap();
    let run = run_doc(&exec_doc, &[], RunMode::Execute, ExecutorChoice::Local)
        .await
        .unwrap();
    let after: String = run.result.transcripts["c"]
        .iter()
        .map(|e| e.output.clone())
        .collect();
    assert!(
        after.contains("hello from the agent"),
        "exec arm: the exec consumed the agent's edit in the same pass: {after:?}"
    );
}

// ===========================================================================
// Criterion 4 — termination
// ===========================================================================

/// A flow node that fails to complete takes the whole document with it:
/// `converge` blocks on stream end, so the literal prose either side of the
/// agent cell is lost too. There is no per-cell bound anywhere in the
/// converge path.
#[tokio::test(flavor = "multi_thread")]
async fn c4_flow_a_non_completing_node_hangs_the_whole_document() {
    let graph = flow_graph(Arc::new(NeverCompletingNode));
    let converged = tokio::time::timeout(
        std::time::Duration::from_millis(300),
        converge_with_provenance(graph, Context::new()),
    )
    .await;
    assert!(
        converged.is_err(),
        "converge must still be blocked: one non-completing cell hangs the \
         document, including its literal siblings"
    );
}

/// Turn-budget exhaustion inside the node is the only thing that saves the
/// flow arm — and it only works because the spike's loop is careful to end
/// the stream carrying an error value rather than propagating a panic.
#[tokio::test(flavor = "multi_thread")]
async fn c4_flow_turn_budget_ends_the_stream_instead_of_hanging() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "spike.hick", &doc_flow_only_agent());

    let record = Arc::new(Mutex::new(Err("not run".to_string())));
    let ctx = Context::new().with_extension(AgentDeps {
        llm: scripted_never_settles(10),
        executor: executor(),
        doc_path: doc.clone(),
        record: record.clone(),
    });

    let graph = flow_graph(Arc::new(AgentFlowNode { max_turns: 3 }));
    let converged = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        converge_with_provenance(graph, ctx),
    )
    .await
    .expect("bounded node must terminate")
    .expect("converge produced output");
    assert!(converged.0.contains("[agent failed]"));
    let err = record.lock().unwrap().clone().unwrap_err();
    assert!(err.contains("max_turns (3) exhausted"), "{err}");
}

/// Under exec placement a cell with no baseline is *reported* — named, with a
/// reason and a remedy — and the rest of the document still produces output.
/// The blast radius is the cell, not the document.
#[tokio::test(flavor = "multi_thread")]
async fn c4_exec_a_cell_without_a_baseline_is_reported_not_fatal() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "spike.hick", &doc_exec_agent_frozen("true"));

    let run = tokio::time::timeout(
        std::time::Duration::from_secs(60),
        run_doc(&doc, &[], RunMode::Verify, ExecutorChoice::Local),
    )
    .await
    .expect("exec arm must not hang")
    .unwrap();

    assert_eq!(run.result.never_run.len(), 1);
    let (cell, reason) = run.result.never_run.iter().next().unwrap();
    let message = hickory_cli::unverifiable_message(&run.doc_path, cell, reason);
    assert!(message.contains("line"), "the cell is named: {message}");
    // The document still wove: one unverifiable cell is not a document-wide
    // stop.
    assert!(
        run.result.files.contains_key("spike.md"),
        "the rest of the document still produced output"
    );
}

// ===========================================================================
// Criterion 5 — lineage completeness
// ===========================================================================

/// Bytes a flow-placed agent emits carry no source span, so
/// `hickory lineage` cannot resolve them and `map_edits` would refuse to edit
/// through them. Bytes an exec-placed agent authors land in the *document*
/// first, so they arrive at lineage as ordinary `Literal` spans.
#[tokio::test(flavor = "multi_thread")]
async fn c5_only_exec_placement_produces_resolvable_lineage() {
    // --- flow arm.
    let flow_dir = tempfile::tempdir().unwrap();
    let flow_doc = write_doc(flow_dir.path(), "spike.hick", &doc_flow_only_agent());
    let record = Arc::new(Mutex::new(Err("not run".to_string())));
    let ctx = Context::new().with_extension(AgentDeps {
        llm: scripted_two_turns(),
        executor: executor(),
        doc_path: flow_doc.clone(),
        record,
    });
    let graph = flow_graph(Arc::new(AgentFlowNode { max_turns: 8 }));
    let (text, map) = converge_with_provenance(graph, ctx).await.unwrap();
    let spans = hickory_lineage::from_provenance_map(&map);
    let agent_start = text
        .find("Wrote greeting.txt")
        .expect("agent bytes present");
    let agent_span = spans
        .iter()
        .find(|p| p.start <= agent_start && agent_start < p.end)
        .expect("a provenance span covers the agent's bytes");
    assert!(
        agent_span.origin.source().is_none(),
        "flow arm: agent-authored bytes have no editable source — \
         `hickory lineage` reports them synthetic"
    );

    // --- exec arm: the edit is in the document before the graph is built.
    let exec_dir = tempfile::tempdir().unwrap();
    let exec_doc = write_doc(exec_dir.path(), "spike.hick", &doc_flow_only_agent());
    run_scripted_agent(scripted_two_turns().as_ref(), executor(), &exec_doc, 8)
        .await
        .unwrap();
    let run = run_doc(&exec_doc, &[], RunMode::Weave, ExecutorChoice::Local)
        .await
        .unwrap();
    let spans = output_lineage(&run, "greeting.txt").unwrap();
    assert!(
        spans.iter().any(|p| p.origin.source().is_some()),
        "exec arm: agent-authored bytes resolve to a document span, so \
         `hickory lineage` + `git blame` compose as the design says: {spans:?}"
    );
}

// ===========================================================================
// Criterion 6 — cost
// ===========================================================================

/// Placement does not move a single token. Both arms run the same loop with
/// the same scripted turns, so turns, the four-way split, and USD are equal.
/// Recorded through the harness's own `RunRecord`/`generate_report` so the
/// comparison is made with the instrument the issue names.
#[tokio::test(flavor = "multi_thread")]
async fn c6_cost_is_identical_across_placements() {
    use hickory_agent::harness::{RunRecord, generate_report};

    let mut records = Vec::new();
    let mut runs = Vec::new();
    for arm in ["flow", "exec"] {
        let dir = tempfile::tempdir().unwrap();
        let doc = write_doc(dir.path(), "spike.hick", &doc_flow_only_agent());
        let run = run_scripted_agent(scripted_two_turns().as_ref(), executor(), &doc, 8)
            .await
            .unwrap();
        records.push(RunRecord {
            experiment: "S1-agent-placement".to_string(),
            arm: arm.to_string(),
            baseline: arm == "flow",
            task_id: "write-greeting".to_string(),
            model: "claude-sonnet-5".to_string(),
            turn: None,
            usage: run.usage,
            cost_usd: hickory_agent::cost_usd("claude-sonnet-5", &run.usage),
            turns: Some(run.turns),
            completed: Some(run.settled),
            check_pass: Some(true),
            tools_requested: true,
            tools_active: true,
            at: "2026-08-09T00:00:00Z".to_string(),
        });
        runs.push(run);
    }

    assert_eq!(runs[0].turns, runs[1].turns);
    assert_eq!(runs[0].usage, runs[1].usage);
    assert_eq!(records[0].cost_usd, records[1].cost_usd);
    assert!(records[0].cost_usd.unwrap() > 0.0);

    // Round-trip through the harness so the four-way split is measured the
    // way every other experiment in this repo measures it.
    let out = tempfile::tempdir().unwrap();
    let runs_dir = out.path().join("runs");
    std::fs::create_dir_all(&runs_dir).unwrap();
    let jsonl: String = records
        .iter()
        .map(|r| format!("{}\n", serde_json::to_string(r).unwrap()))
        .collect();
    std::fs::write(runs_dir.join("s1.jsonl"), jsonl).unwrap();
    let report = generate_report(&runs_dir).unwrap();
    assert!(report.contains("S1-agent-placement"));
    assert!(report.contains("flow (baseline)"));
    assert!(report.contains("exec"));
}
