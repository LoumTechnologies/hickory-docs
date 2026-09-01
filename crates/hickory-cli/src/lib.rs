//! Library core of the `hick` CLI.
//!
//! The binary (`src/main.rs`) is a thin argument parser over these functions,
//! so the server can drive the same run/check/weave/render code paths via
//! library calls instead of shelling out.

pub mod adopt;
pub mod agent_cell_runner;
pub mod agent_lineage;
pub mod anchor;
pub mod carry;
pub mod claude_code;
pub mod code_model;
pub mod continuity;
pub mod dap_install;
pub mod debug_sessions;
pub mod diagram;
pub mod doc_tools;
pub mod editor_lsp;
pub mod emission;
pub mod floor;
pub mod history;
pub mod index_install;
pub mod index_read;
pub mod ingest;
pub mod ingest_exec;
pub mod init;
pub mod language_tier;
pub mod lsp_install;
pub mod mcp;
pub mod merge_driver;
pub mod open_app;
pub mod replay;
pub mod scaffold;
pub mod search_install;
pub mod serve;
pub mod tool_install;
pub mod typed_client;
pub mod up;

/// `hick init` entry points: idempotent local git-repo setup.
pub use editor_lsp::{AdoptedServer, EditorOutcome, EditorSetup};
pub use init::{InitReport, print_init_report, run_init};

/// Lineage for agent-authored bytes: session + turn from provenance,
/// authorship from `git blame`, graceful degradation when the session is
/// private or absent.
pub use agent_lineage::{AgentLineage, Authorship, Reasoning};

/// The binary's [`hick_literate::agent_cell::AgentRunner`]: settles one
/// `<hick:agent>` DAG vertex through the real ReAct loop, or reports that no
/// provider key is available so the cell is unverifiable rather than fatal.
pub use agent_cell_runner::LlmAgentRunner;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context as _, Result, bail};
use hick_exec::node::FileContent;
use hick_literate::render::{Block, BlockModelInput, build_block_model};
use hick_literate::{
    CellId, Executor, LocalExecutor, NoBaseline, PipelineConfig, PipelineResult, cache,
    expand_path_arg, run_pipeline_live, run_pipeline_weave,
};

/// Which executor backend to use, from `HICKORY_EXECUTOR`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutorChoice {
    Local,
    /// The local executor, with each cell confined to its own workdir.
    /// See `hickory-executor-sandbox`.
    Sandbox,
    Docker,
    Canopy,
}

impl ExecutorChoice {
    /// The name this backend is known by — the `HICKORY_EXECUTOR` value, the
    /// `GET /api/executor` field, and what the `hick serve` banner prints.
    pub fn as_str(self) -> &'static str {
        match self {
            ExecutorChoice::Local => "local",
            ExecutorChoice::Sandbox => "sandbox",
            ExecutorChoice::Docker => "docker",
            ExecutorChoice::Canopy => "canopy",
        }
    }

    /// Read `HICKORY_EXECUTOR` (default: `sandbox`).
    ///
    /// The default is the confined one, and it refuses to run where it cannot
    /// confine anything. A document is a file people send each other and
    /// agents write; "read every cell before running it" is advice nobody
    /// follows twice, so the safe execution has to be the one you get without
    /// asking. `HICKORY_EXECUTOR=local` is still there, one variable away,
    /// and says plainly what it gives up.
    pub fn from_env() -> Result<Self> {
        match std::env::var("HICKORY_EXECUTOR").as_deref() {
            Err(_) | Ok("") | Ok("sandbox") => Ok(ExecutorChoice::Sandbox),
            Ok("local") => Ok(ExecutorChoice::Local),
            Ok("docker") => Ok(ExecutorChoice::Docker),
            Ok("canopy") => Ok(ExecutorChoice::Canopy),
            Ok(other) => bail!(
                "unknown HICKORY_EXECUTOR value '{other}' (expected \"local\", \"sandbox\", \
                 \"docker\", or \"canopy\")"
            ),
        }
    }

    /// Build the executor. Canopy reads its `CANOPY_*` env config here
    /// (connection to the node agent is lazy — first use).
    ///
    /// Docker probes the daemon eagerly, so an unreachable Docker is a clear
    /// startup error rather than a cell that fails halfway through a run —
    /// which means this is `async`.
    pub async fn build(self) -> Result<Arc<dyn Executor>> {
        self.build_for(None).await
    }

    /// [`build`](Self::build), naming the project the run belongs to.
    ///
    /// The local executors derive a scratch directory from this so a cell
    /// that prints its own path reproduces. `None` means "use the working
    /// directory", which is right for a caller serving a folder and wrong for
    /// one that was handed a document elsewhere — see
    /// [`LocalExecutor::new_stable_for`].
    pub async fn build_for(self, project: Option<&Path>) -> Result<Arc<dyn Executor>> {
        match self {
            // The derived scratch directory: `hick` is one executor in one
            // process, which is the case it is safe and useful for.
            ExecutorChoice::Local => Ok(Arc::new(LocalExecutor::new_stable_for(project)?)),
            ExecutorChoice::Sandbox => Ok(Arc::new(
                hickory_executor_sandbox::SandboxedExecutor::new_stable_for(project)?,
            )),
            ExecutorChoice::Docker => Ok(Arc::new(
                hickory_executor_docker::DockerExecutor::new().await?,
            )),
            ExecutorChoice::Canopy => Ok(Arc::new(
                hickory_executor_canopy::CanopyExecutor::from_env()?,
            )),
        }
    }
}

/// How to obtain transcripts for a document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    /// Execute every exec block through the executor. A cell that declares
    /// `freeze="true"` and has no recording yet is executed **once** and
    /// recorded: `run` is the mode that establishes baselines, so a document
    /// can declare a cell frozen from the moment it is written.
    Execute,
    /// Execute like [`RunMode::Execute`], but collect cells with no baseline
    /// as *unverifiable* instead of aborting, so `check` can report every
    /// such cell in one pass and exit with the unverifiable code.
    Verify,
    /// Never execute: use cached transcripts where present, mark the rest
    /// never-run.
    Weave,
}

impl RunMode {
    /// Whether this mode really runs commands through the executor.
    pub fn executes(self) -> bool {
        matches!(self, RunMode::Execute | RunMode::Verify)
    }
}

/// What `check` concluded about a document.
///
/// The four are deliberately distinct, because the fix for each is
/// different: drift means someone forgot to regenerate, a failed expectation
/// means a claim the document makes is false, unverifiable means nothing was
/// ever established. CI has to be able to respond differently to those — most
/// concretely, a job may auto-regenerate drift and must never auto-anything a
/// false claim — so each gets its own exit code. See
/// `docs/guarantees/verification/test-separates-unverifiable-from-drifted.md`.
///
/// The **`Ord` order is the precedence order**, not the exit-code order:
/// [`check_outcome`] takes the maximum of the outcomes present, and the
/// numbers attached by [`CheckOutcome::exit_code`] are frozen where issue #3
/// left them so existing CI keeps working.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CheckOutcome {
    /// Re-derivation matches what is committed.
    Verified,
    /// A committed output no longer reproduces, or a `hick:transform` passage
    /// is stale. The document did derive; what is on disk is out of date.
    Drifted,
    /// At least one cell has no baseline at all, so there is nothing for
    /// re-derivation to be compared against.
    Unverifiable,
    /// A `hick:expect` did not hold: the document states something that is
    /// not true of what its own cells produced.
    ExpectationFailed,
}

impl CheckOutcome {
    /// The process exit code for this outcome. **This is a user-facing
    /// contract** — CI scripts branch on it — so these numbers are as stable
    /// as any other part of the CLI surface. `3` is the newest, and is
    /// deliberately appended rather than slotted in between: renumbering
    /// `unverifiable` would silently change the meaning of every existing
    /// `if [ $? -eq 2 ]`.
    pub fn exit_code(self) -> u8 {
        match self {
            CheckOutcome::Verified => 0,
            CheckOutcome::Drifted => 1,
            CheckOutcome::Unverifiable => 2,
            CheckOutcome::ExpectationFailed => 3,
        }
    }
}

/// The result of processing one document.
pub struct DocRun {
    /// Path of the `.hick` source document.
    pub doc_path: PathBuf,
    /// Raw source text.
    pub source: String,
    /// Parsed document (spans intact; unfiltered).
    pub doc: hick_lang::HickDocument,
    /// Pipeline result: files, transcripts, expectations.
    pub result: PipelineResult,
}

/// One reason `check` fails.
#[derive(Debug)]
pub enum CheckFailure {
    Expectation(hick_literate::expect::ExpectationOutcome),
    /// A produced output file differs from (or is missing on) disk.
    Drift {
        doc: PathBuf,
        output_path: PathBuf,
        detail: String,
    },
    /// A `hick:transform` passage was written from bytes that have since
    /// changed. Unlike drift this is not a mismatch that can be recomputed —
    /// an LLM wrote the passage, so the fix is `hick refresh`, not a re-run.
    StaleTransform {
        doc: PathBuf,
        line: usize,
        select: String,
        instruct: String,
    },
    /// A cell has no baseline: it neither executed nor was answered from a
    /// recording, so re-derivation has nothing to compare against. This is
    /// NOT drift — drift needs a baseline to have drifted from.
    Unverifiable {
        doc: PathBuf,
        cell: CellId,
        reason: NoBaseline,
    },
}

impl CheckFailure {
    /// Which outcome this failure implies on its own.
    ///
    /// A stale `hick:transform` counts as drift rather than a failed
    /// expectation: like a woven file that no longer reproduces, it says the
    /// committed bytes are out of date with their inputs, and the fix is to
    /// regenerate (`hick refresh`). No claim was falsified.
    pub fn outcome(&self) -> CheckOutcome {
        match self {
            CheckFailure::Expectation(_) => CheckOutcome::ExpectationFailed,
            CheckFailure::Unverifiable { .. } => CheckOutcome::Unverifiable,
            CheckFailure::Drift { .. } | CheckFailure::StaleTransform { .. } => {
                CheckOutcome::Drifted
            }
        }
    }
}

/// The report for one unverifiable cell: which cell, why it has no baseline,
/// and what to do about it.
///
/// The "what to do" is deliberately checked against what the shipped binary
/// actually accepts: every command named here is one `hick` really runs.
/// `hick run --cache` writes recordings; `hick test` never does, so
/// that it can never manufacture the baseline it then compares against.
pub fn unverifiable_message(doc: &Path, cell: &CellId, reason: &NoBaseline) -> String {
    const KEYED_BY: &str = "A recording is keyed by the container image, capabilities, command \
         text, and secret names together, so editing any of them retires the old recording — \
         this can also mean \"the cell changed since it was recorded\".";

    // `hick run` is what establishes a baseline: a cell frozen from the
    // start executes exactly once, on the run that records it, with no edit
    // to the document. `test` deliberately has no `--cache` and never
    // records: a verifier that can write its own baseline verifies nothing.
    let record_it = format!(
        "To record a baseline: run `hick run {}` once — a frozen cell with no recording \
         executes exactly once, on the run that records it, and replays from then on. Then \
         re-run `hick test {}`. `hick test` never writes a recording, on purpose: a \
         check that writes its own baseline is not a check.",
        doc.display(),
        doc.display()
    );

    let head = format!("UNVERIFIABLE {} {cell}", doc.display());
    match reason {
        NoBaseline::NotExecuted => format!(
            "{head}: no recorded transcript, and this mode never executes cells, so nothing \
             was ever established for it.\n  \
             Next steps: run `hick run {}` to execute the document, then commit its \
             outputs; `hick weave` only renders what has already been recorded.",
            doc.display()
        ),
        NoBaseline::FrozenWithoutRecording {
            command,
            frozen_by_cell,
        } => {
            let why = if *frozen_by_cell {
                "the cell declares freeze=\"true\", so `hick test` will not run it"
            } else {
                "this run is frozen run-wide, so `hick test` will not run the cell"
            };
            format!(
                "{head}: {why}, and no recording exists for it (command: {command}). Nothing \
                 was ever established for this cell — that is not drift, which needs a \
                 baseline to have drifted from.\n  \
                 Next steps: {record_it}\n  \
                 Or remove freeze=\"true\" from the cell (set freeze=\"false\") so \
                 `hick test` executes it and verifies its real output every time.\n  \
                 {KEYED_BY}"
            )
        }
        NoBaseline::AgentWithoutRunner {
            prompt,
            model_declared,
        } => {
            let replay = if *model_declared {
                "This cell declares its model=, so a recording made on a machine with a key \
                 replays here with no key at all — commit .hick-cache/transcripts/ for it."
            } else {
                "This cell declares no model=. Add one (model=\"…\") so a recording made \
                 elsewhere can be found from here: an agent recording is keyed by the prompt \
                 AND the model, and with no runner there is nothing to ask which model would \
                 have run."
            };
            format!(
                "{head}: this is an agent cell (prompt: {prompt}), no recording answered it, \
                 and this run has no agent runner configured — no provider API key, so \
                 nothing could have run it. Nothing was ever established for this cell; that \
                 is not drift.\n  \
                 Next steps: export the provider's key (ANTHROPIC_API_KEY, or \
                 HICKORY_AGENT_PROVIDER plus that provider's key) and run `hick run \
                 --cache {}` once to establish a baseline, then commit it — or declare \
                 freeze=\"true\" on the cell, which makes plain `hick run` record it \
                 once and replay it after that.\n  \
                 {replay}\n  \
                 This is the expected state in CI and on a fresh clone: the rest of the \
                 document still ran, and only this cell is unverifiable.",
                doc.display()
            )
        }
        NoBaseline::FrozenWithoutCacheDirectory { command } => format!(
            "{head}: the cell declares freeze=\"true\" (command: {command}), but there is no \
             recording directory (.hick-cache/transcripts/) next to the document, so no \
             recording can exist. Nothing was ever established for this cell.\n  \
             Next steps: {record_it}\n  \
             Or remove freeze=\"true\" from the cell (set freeze=\"false\") so it executes \
             and is verified for real every time.\n  \
             Common causes: the document was moved away from its project's .hick-cache/, or \
             it has never been run — `hick run` creates the recording directory, and \
             `hick test` never does."
        ),
    }
}

/// The verdict for a whole set of failures.
///
/// Precedence, weakest to strongest:
/// `Verified < Drifted < Unverifiable < ExpectationFailed`.
///
/// Unverifiable outranks drifted: when a cell has no baseline, the drift
/// verdict for the document it is part of is not trustworthy, and the
/// stronger statement ("this document is not actually verified") is the one
/// CI needs to hear.
///
/// A failed expectation outranks both, because it is the only outcome that
/// asserts something is *definitely* wrong rather than out of date or
/// unknown, and it is the one no automation may act on by itself. It is also
/// never contaminated by a missing baseline: an unverifiable cell never
/// evaluates an expectation, so every expectation that failed belongs to a
/// cell that really ran.
pub fn check_outcome(failures: &[CheckFailure]) -> CheckOutcome {
    failures
        .iter()
        .map(CheckFailure::outcome)
        .max()
        .unwrap_or(CheckOutcome::Verified)
}

/// Every `hick:transform` in `source` whose inputs have changed since the
/// passage was written.
///
/// Checking a transform never calls a model: the document does not claim its
/// passage reproduces, only that it was written from exactly these bytes under
/// exactly this instruction. That claim is a fingerprint comparison, which
/// makes it free, offline, and deterministic in CI — the properties that let
/// an LLM-written passage live in a verified document at all.
pub fn stale_transforms(doc_path: &Path, source: &str) -> Result<Vec<CheckFailure>> {
    let doc = transform_document(doc_path, source)?;
    let mut out = Vec::new();
    for tag in own_transforms(&doc) {
        let (select, instruct) = transform_spec(tag);
        let recorded = tag.get_attribute("from").unwrap_or_default();
        let input = transform_input(&doc, &select);
        let actual = hick_lang::transform_fingerprint(&input, &instruct);
        if recorded != actual {
            out.push(CheckFailure::StaleTransform {
                doc: doc_path.to_path_buf(),
                line: tag.source_line,
                select,
                instruct,
            });
        }
    }
    Ok(out)
}

/// The bytes a transform reads: its selected fragments, concatenated in
/// document order.
/// Load a document the way the transform fingerprint paths must see it.
///
/// `hick test` (which checks a `from=` fingerprint) and `hick refresh` (which
/// writes one) have to agree byte-for-byte about a transform's input, or every
/// refresh would immediately read as stale. That means both must apply the same
/// transcript derivation the pipeline applies — so they share this function
/// rather than each calling `parse` and hoping.
///
/// Includes and upstreams are resolved first, exactly as the pipeline resolves
/// them: a transform may `select=` a meeting turn or a decision that lives in
/// a document upstream of this one, and that selection has to find the same
/// bytes here that `hick:paste` finds at weave — otherwise the fingerprint is
/// taken over an empty input, and a passage summarizing another document can
/// never be anything but stale (or, worse, never stale once the other document
/// changes).
pub fn transform_document(doc_path: &Path, source: &str) -> Result<hick_lang::HickDocument> {
    let mut doc = hick_lang::parse(source)
        .map_err(|e| anyhow::anyhow!("parse error in {}: {e}", doc_path.display()))?;
    let base_dir = project_dir_of(doc_path);
    let mut seen = std::collections::HashSet::new();
    if let Ok(canonical) = std::fs::canonicalize(doc_path) {
        seen.insert(canonical);
    }
    hick_lang::resolve_includes(&mut doc, base_dir, &mut seen)
        .map_err(|e| anyhow::anyhow!("include error in {}: {e}", doc_path.display()))?;
    hick_transcript::expand(&mut doc);
    Ok(doc)
}

/// The transforms that belong to THIS document — not ones spliced in by
/// `hick:include`, whose spans index the included file and whose passages are
/// that file's to check and refresh. `hick test` on the includer would
/// otherwise report the included passage twice, and `hick refresh` would
/// write a passage at another file's offsets into this one.
pub fn own_transforms(doc: &hick_lang::HickDocument) -> Vec<&hick_lang::HickTag> {
    let mut tags: Vec<&hick_lang::HickTag> = doc
        .tags()
        .filter(|t| t.name == "transform" || t.name == "check")
        .filter(|t| t.source_span.is_none_or(|s| s.file_id.is_none()))
        .collect();
    tags.sort_by_key(|t| t.source_line);
    tags
}

/// The instruction a `hick:check` runs under when it names none of its own:
/// one sentence against its sources, BACKED or UNSUPPORTED, citing ids. Fixed
/// text, so a document full of checks carries the question once — here — and
/// `from=` still fingerprints it, so a change to this sentence is a change to
/// every check, visibly.
pub const CHECK_INSTRUCT: &str = "The input is a list of fragments, each prefixed with its id in brackets; meeting turns also name their speaker and time. Exactly one fragment is a sentence from a Slack message I am about to send (its id starts with #m or #a); every other fragment is a source: a meeting turn or a finding from the analysis or the fix. Say whether the message sentence is BACKED or UNSUPPORTED by the sources. If BACKED, for each factual part of it name the source id and quote verbatim the shortest passage that backs it. If any part is not backed by any source, say which part. Two or three lines, no preamble.";

/// What a transform-like element reads and is asked: for `hick:transform`,
/// its `select=` and `instruct=`; for `hick:check`, the sentence under test
/// (`claim=`) joined with its sources (`against=`), and the built-in
/// instruction unless `instruct=` overrides it. A check IS a transform — same
/// fingerprint, same refresh, same staleness — spelled for the one question
/// people ask most.
pub fn transform_spec(tag: &hick_lang::HickTag) -> (String, String) {
    if tag.name == "check" {
        let claim = tag
            .get_attribute("claim")
            .unwrap_or_default()
            .trim()
            .to_string();
        let against = tag
            .get_attribute("against")
            .unwrap_or_default()
            .trim()
            .to_string();
        let select = match (claim.is_empty(), against.is_empty()) {
            (false, false) => format!("{claim},{against}"),
            (false, true) => claim,
            (true, _) => against,
        };
        let instruct = tag
            .get_attribute("instruct")
            .filter(|i| !i.trim().is_empty())
            .unwrap_or(CHECK_INSTRUCT)
            .to_string();
        return (select, instruct);
    }
    (
        tag.get_attribute("select").unwrap_or_default().to_string(),
        tag.get_attribute("instruct")
            .unwrap_or_default()
            .to_string(),
    )
}

/// One declared citation: an element that says `cites="…"` and the fragments
/// its selectors name — DECLARED provenance, the author's assertion, resolved
/// to places so the app can draw it as one (and never as lineage).
#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct DeclaredCite {
    /// The `cites=` value as written.
    pub select: String,
    /// The citing element.
    pub from: CitePlace,
    /// What the selectors resolved to — possibly in upstream documents,
    /// possibly nothing (a dangling citation is reported, not hidden).
    pub to: Vec<CitePlace>,
}

/// A place a citation points at or comes from: a file and a line range.
#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct CitePlace {
    pub path: String,
    pub first_line: usize,
    pub last_line: usize,
    /// The element's name (`claim`, `transform`, `copy`, `said`, …).
    pub element: String,
    /// Its id, when it has one.
    pub id: Option<String>,
}

/// Every `cites=` in the document (its own elements, not spliced ones),
/// resolved against the document as the pipeline sees it — upstreams and
/// transcript turns included. Paths are the document's path for its own
/// elements and the spliced file's canonical path for upstream fragments.
pub fn declared_cites(doc_path: &Path, source: &str) -> Result<Vec<DeclaredCite>> {
    let doc = transform_document(doc_path, source)?;
    let own_lines = |span: Option<hick_lang::SourceSpan>| -> (usize, usize) {
        span.map(|s| {
            (
                line_of(source, s.start),
                line_of(source, s.end.max(s.start)),
            )
        })
        .unwrap_or((0, 0))
    };
    // Line numbers in a spliced file need that file's text.
    let mut file_sources: std::collections::HashMap<usize, String> = Default::default();
    let mut place_of = |tag: &hick_lang::HickTag| -> CitePlace {
        let (path, first, last) = match tag.source_span {
            Some(s) => match s.file_id {
                None => {
                    let (a, b) = own_lines(Some(s));
                    (doc_path.display().to_string(), a, b)
                }
                Some(id) => {
                    let path = doc.span_files[usize::from(id)].clone();
                    let text = file_sources
                        .entry(usize::from(id))
                        .or_insert_with(|| std::fs::read_to_string(&path).unwrap_or_default())
                        .clone();
                    (
                        path,
                        line_of(&text, s.start),
                        line_of(&text, s.end.max(s.start)),
                    )
                }
            },
            None => {
                // A derived turn has no tag span; its text child carries the
                // span into the transcript's file.
                match tag.children.first() {
                    Some(hick_lang::HickNode::Text(_, Some(s))) => match s.file_id {
                        None => {
                            let (a, b) = own_lines(Some(*s));
                            (doc_path.display().to_string(), a, b)
                        }
                        Some(id) => {
                            let path = doc.span_files[usize::from(id)].clone();
                            let text = file_sources
                                .entry(usize::from(id))
                                .or_insert_with(|| {
                                    std::fs::read_to_string(&path).unwrap_or_default()
                                })
                                .clone();
                            (
                                path,
                                line_of(&text, s.start),
                                line_of(&text, s.end.max(s.start)),
                            )
                        }
                    },
                    _ => (doc_path.display().to_string(), 0, 0),
                }
            }
        };
        CitePlace {
            path,
            first_line: first,
            last_line: last,
            element: tag.name.clone(),
            id: tag.get_attribute("id").map(str::to_string),
        }
    };
    let mut out = Vec::new();
    let mut stack: Vec<&hick_lang::HickTag> = doc.tags().collect();
    let mut citing: Vec<&hick_lang::HickTag> = Vec::new();
    while let Some(tag) = stack.pop() {
        // Only this document's own elements declare for it.
        if tag.source_span.is_some_and(|s| s.file_id.is_some()) {
            continue;
        }
        if tag
            .get_attribute("cites")
            .is_some_and(|c| !c.trim().is_empty())
        {
            citing.push(tag);
        }
        for child in &tag.children {
            if let hick_lang::HickNode::Tag(t) = child {
                stack.push(t);
            }
        }
    }
    citing.sort_by_key(|t| t.source_line);
    for tag in citing {
        let select = tag
            .get_attribute("cites")
            .unwrap_or_default()
            .trim()
            .to_string();
        let to = hick_lang::fragments_matching(&doc, &select)
            .into_iter()
            .map(&mut place_of)
            .collect();
        out.push(DeclaredCite {
            select,
            from: place_of(tag),
            to,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod mount_warning_tests {
    use super::absolute_mount_warnings;

    /// `mount="src:/project"` is fine; only a command that says `/project/…`
    /// is the mistake the warning exists for.
    #[test]
    fn warns_only_when_a_command_uses_the_absolute_path() {
        let relative = hick_lang::parse(
            "<hick:exec container=\"py\" mount=\"src:/project\">python3 project/a.py</hick:exec>",
        )
        .unwrap();
        assert!(absolute_mount_warnings(&relative).is_empty());
        let absolute = hick_lang::parse(
            "<hick:exec container=\"py\" mount=\"src:/project\">python3 /project/a.py</hick:exec>",
        )
        .unwrap();
        assert_eq!(absolute_mount_warnings(&absolute).len(), 1);
    }
}

/// 1-based line of byte offset `at` in `text`.
fn line_of(text: &str, at: usize) -> usize {
    text.as_bytes()[..at.min(text.len())]
        .iter()
        .filter(|b| **b == b'\n')
        .count()
        + 1
}

/// The bytes a transform was written from: every selected fragment, in
/// document order, one paragraph each.
///
/// Each fragment is rendered the way a reader would want to cite it — its id
/// in brackets when it has one, a speaker turn as `Sam (00:00:56.000): …`, a
/// transcript as its turns one per line — and fragments are separated by a
/// blank line. That is what lets a passage that checks one sentence against
/// a meeting say *which* turn backs it, by id, rather than being handed the
/// meeting and the sentence run together as one string with no seam.
///
/// This rendering IS the fingerprinted input, so changing it changes every
/// fingerprint; that is the right trade, because an input the model could
/// not parse is an input the fingerprint was protecting for nothing.
pub fn transform_input(doc: &hick_lang::HickDocument, select: &str) -> String {
    hick_lang::fragments_matching(doc, select)
        .iter()
        .map(|t| render_fragment(t))
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn render_fragment(tag: &hick_lang::HickTag) -> String {
    let id = tag
        .get_attribute("id")
        .map(|id| format!("[#{id}] "))
        .unwrap_or_default();
    match tag.name.as_str() {
        "said" => format!("{id}{}{}", said_header(tag), tag.text_content().trim()),
        "transcript" => {
            let turns: Vec<String> = tag
                .children
                .iter()
                .filter_map(|n| match n {
                    hick_lang::HickNode::Tag(t) if t.name == "said" => Some(render_fragment(t)),
                    _ => None,
                })
                .collect();
            if turns.is_empty() {
                format!("{id}{}", tag.text_content().trim())
            } else {
                turns.join("\n")
            }
        }
        _ => format!("{id}{}", tag.text_content().trim()),
    }
}

fn said_header(tag: &hick_lang::HickTag) -> String {
    match (tag.get_attribute("by"), tag.get_attribute("at")) {
        (Some(by), Some(at)) => format!("{by} ({at}): "),
        (Some(by), None) => format!("{by}: "),
        (None, Some(at)) => format!("({at}): "),
        (None, None) => String::new(),
    }
}

/// Expand a path argument (file or directory) into `.hick` documents.
///
/// Expanding a DIRECTORY skips agent session documents. A session is a
/// record of what an agent did, not a pipeline to re-run: it embeds the
/// document that was under discussion, `hick:upstream` and all, so checking
/// one resolves those edges relative to `sessions/` and fails on paths that
/// were correct where they were written. `hick test docs/` should not
/// start failing the moment an agent runs in that tree.
///
/// Naming a session file EXPLICITLY still works — `hick weave` on a
/// session is a real thing to want.
pub fn expand_docs(path: &Path) -> Result<Vec<PathBuf>> {
    let (files, _config) = expand_path_arg(path)?;
    if path.is_file() {
        return Ok(files);
    }
    Ok(files
        .into_iter()
        .filter(|f| {
            std::fs::read_to_string(f)
                .map(|s| !hick_lang::is_session_source(&s))
                .unwrap_or(true)
        })
        .collect())
}

/// What a run does with the project's transcript cache
/// (`.hick-cache/transcripts/` next to the document), run-wide.
///
/// One setting on one axis — what a missing recording means — so there is no
/// impossible fourth state to defend against. The run-wide default is
/// [`CacheMode::Off`]: `hick run` asks *"what is the answer now"*, so no
/// cell is answered from a recording and nothing is recorded. `--cache`
/// selects [`CacheMode::Reuse`] and `--freeze` selects
/// [`CacheMode::Require`]; a cell's own `freeze=` attribute overrides
/// whichever of them is in force.
///
/// Guarantee: `docs/guarantees/verification/recordings-are-written-only-when-asked-for.md`
pub use hick_literate::cache::CacheMode;

/// Run one document with the default cache mode ([`CacheMode::Off`]).
///
/// Every cell executes and nothing is recorded, which is what every caller
/// that has no `--cache`/`--freeze` flags to offer wants.
pub async fn run_doc(
    doc_path: &Path,
    params: &[(String, String)],
    mode: RunMode,
    executor_choice: ExecutorChoice,
) -> Result<DocRun> {
    run_doc_cached(doc_path, params, mode, executor_choice, CacheMode::Off).await
}

/// The directory a document lives in, never the empty path.
///
/// `Path::new("d.hick").parent()` is `Some("")`, not `None`, so
/// `parent().unwrap_or(".")` silently yields an empty path for every document
/// named without a directory — which is how every `hick run <bare-filename>`
/// on this machine came to share one scratch directory.
fn project_dir_of(doc_path: &Path) -> &Path {
    match doc_path.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir,
        _ => Path::new("."),
    }
}

/// Run one document. Each document gets its own executor instance so
/// container namespaces never collide across documents.
pub async fn run_doc_cached(
    doc_path: &Path,
    params: &[(String, String)],
    mode: RunMode,
    executor_choice: ExecutorChoice,
    cache_mode: CacheMode,
) -> Result<DocRun> {
    run_doc_subset(doc_path, params, mode, executor_choice, cache_mode, None).await
}

/// [`run_doc_cached`], narrowed to a subgraph of the document's own DAG.
///
/// `subset: None` (what `run_doc_cached` always passes) is the ordinary,
/// unchanged whole-document run. `subset: Some(ids)` treats every exec cell
/// OUTSIDE that set as absent — not run, not required to succeed, and not a
/// dependency failure for anything inside the set. This exists for `hick
/// ingest --from`: computing a target cell's transitive predecessors and
/// passing that closure here lets ingest run just enough of the document to
/// read the target cell's output, without requiring cells the target does
/// not depend on to already be correct — a genuine chicken-and-egg problem
/// for a cell being authored before the rest of the document that will
/// depend on it exists. See
/// docs/guarantees/authoring/ingest-does-not-require-the-rest-of-the-document-to-already-pass.md.
pub async fn run_doc_subset(
    doc_path: &Path,
    params: &[(String, String)],
    mode: RunMode,
    executor_choice: ExecutorChoice,
    cache_mode: CacheMode,
    subset: Option<std::collections::HashSet<hick_exec::dag::ExecId>>,
) -> Result<DocRun> {
    let source = std::fs::read_to_string(doc_path)
        .with_context(|| format!("failed to read {}", doc_path.display()))?;
    // `parse_from_path`: this document is about to have its outputs written,
    // so its `weave_path` must be the resolved one — the markdown file of its
    // own name when it names none (`bare-documents.md`). The pipeline resolves
    // it the same way, and a `DocRun` whose `doc` disagreed with the files it
    // produced is how the adoption guard came to trip on a file it had itself
    // just written.
    let mut doc = hick_lang::parse_from_path(&source, doc_path)
        .map_err(|e| anyhow::anyhow!("parse error in {}: {e}", doc_path.display()))?;
    // The same projection the pipeline builds, so this `DocRun`'s document and
    // the run's own agree about what a transcript contains.
    hick_transcript::expand(&mut doc);

    // Warn BEFORE executing. The failure this predicts kills the run, so a
    // warning emitted afterwards is a warning nobody ever sees.
    for warning in escaping_warnings(&doc) {
        log::warn!("{}: {warning}", doc_path.display());
    }
    // Not gated on the executor: both resolve mounts under the workdir.
    for warning in absolute_mount_warnings(&doc) {
        log::warn!("{}: {warning}", doc_path.display());
    }
    // A cell that mounts its own output silently loses every recording it
    // makes, and the symptom (`[never run]` over a real run) points nowhere
    // near the cause.
    for warning in self_mounting_warnings(&doc) {
        log::warn!("{}: {warning}", doc_path.display());
    }
    // A silent clobber or an unrelated exit code, either far from the
    // second declaration that actually caused it.
    for warning in output_collision_warnings(&doc) {
        log::warn!("{}: {warning}", doc_path.display());
    }
    // A drawing nobody checks is the thing this feature exists to prevent.
    for warning in claim_warnings(&doc) {
        log::warn!("{}: {warning}", doc_path.display());
    }
    for warning in diagram_assertion_warnings(&doc) {
        log::warn!("{}: {warning}", doc_path.display());
    }
    if executor_choice == ExecutorChoice::Local
        && mode.executes()
        && let Some(warning) = ignored_image_warning(&doc)
    {
        log::info!("{}: {warning}", doc_path.display());
    }

    let doc_name = doc_path.display().to_string();
    let sources = vec![(doc_name.as_str(), source.as_str())];

    let project_dir = project_dir_of(doc_path);

    let result = match mode {
        RunMode::Execute | RunMode::Verify => {
            // Stage the woven files before executing, so a cell can run a
            // file its own document assembles on the FIRST run. Volumes are
            // seeded from the working directory before execution, while
            // `hick:file` content is a result of the content phase — without
            // this, the central move of literate programming only ever works
            // on the second run.
            //
            // This lives inside `run_doc` rather than in the CLI commands
            // because the server already staged separately and the CLI did
            // not: the same document ran through the web app and failed
            // through `hick run`. One place, every caller.
            //
            // Guarantee:
            // docs/guarantees/execution/first-run-behaves-like-every-later-run.md
            stage_woven_files(doc_path, &sources, params).await;

            // The document's own directory, not the shell's. `hick run
            // ../other/doc.hick` is an ordinary thing to type, and keying the
            // scratch root on where the person was standing gave two
            // unrelated documents one name to fight over.
            let executor = executor_choice.build_for(doc_path.parent()).await?;
            // An agent cell needs a model, and only `hick run` may buy one.
            //
            // `hick test` deliberately gets NO runner even on a machine
            // that has a key: a verifier that spends the reader's tokens is a
            // verifier nobody can safely point at a document they did not
            // write, and `agent-cells.md` names that exact hazard. So an agent
            // cell is verified against its recording or reported unverifiable
            // — never re-run to see what it says this time, which would not be
            // a verification anyway, the cell being nondeterministic.
            //
            // A machine with no provider key gets `None` here too, which is
            // the ordinary state of CI: the cell is unverifiable, not fatal.
            let agent_runner = (mode == RunMode::Execute)
                .then(|| agent_cell_runner::LlmAgentRunner::from_env(executor.clone()))
                .flatten()
                .map(|r| Arc::new(r) as Arc<dyn hick_literate::agent_cell::AgentRunner>);
            let config = PipelineConfig {
                working_dir: Some(project_dir.to_path_buf()),
                // A cell may write a `.hick` file, and its fragments become
                // available to the documents in this run — the way a
                // generator contributes a `using` line to a file somebody
                // else owns. Only `.hick` outputs are read this way: a
                // generator's ordinary output is bytes, and scanning it for
                // markup would make `<hick:` unwritable by any program.
                //
                // Bounded rather than run to a fixed point: a document that
                // writes a document that writes a document is a pipeline, and
                // four rounds is enough of one to be useful without letting a
                // mistake spin.
                max_rounds: 4,
                on_exec: None,
                // Best-effort visibility into a cell's real output while a
                // LATER cell in the same run is still broken — the flush into
                // `PipelineResult::files` this run will make on success, if
                // it succeeds, is unaffected either way. Never the final
                // destination: writes an intermediate mirror under
                // `.hick-cache/`, not the real output tree, so it never
                // needs the local-history stop, the missing-recording
                // preservation check, or the read-only clearing that
                // `write_outputs_detailed` owns for the real write. See
                // docs/guarantees/execution/an-earlier-cells-output-survives-a-later-cells-failure.md.
                on_volume_flush: Some(volume_flush_mirror(project_dir.to_path_buf())),
                subset,
                agent_runner,
                max_agent_reprepares: 0,
                // `check` wants every cell with no baseline reported as
                // unverifiable; `run` wants the first one to stop the run.
                collect_unverifiable: mode == RunMode::Verify,
                // The machine-wide default cell time limit, honouring
                // HICKORY_CELL_TIMEOUT. Resolved here — the entry point for
                // `hick run`, `hick test`, `hick up --run`, and the serve run
                // path — so a malformed value fails the run at its start with
                // a message naming the variable, not mid-document.
                // docs/guarantees/execution/a-cell-cannot-hang-a-run.md
                cell_timeout: hick_literate::cell_timeout::CellTimeoutDefault::from_env()?,
            };
            // The pipeline always gets the project's transcript cache, and
            // always by path rather than only when the directory already
            // exists. With no `--cache`/`--freeze` the run-wide mode is
            // `Off`: `run` asks "what is the answer now", so no cell is
            // answered from a recording and nothing is recorded run-wide.
            // The one cell that still uses the directory is one that declares
            // `freeze="true"` for itself — it finds its recording there, and
            // on its very first run it writes one there. Gating on
            // `is_dir()` would make that first run silently record nothing,
            // which is precisely the two-step dance issue #10 removed.
            let cc = cache::CacheConfig::new(project_dir, cache_mode);
            if cache_mode == CacheMode::Reuse {
                std::fs::create_dir_all(&cc.cache_dir).with_context(|| {
                    format!(
                        "failed to create the recording directory {}\n  \
                         Next steps: check that {} is writable, then re-run \
                         `hick run --cache {}`.",
                        cc.cache_dir.display(),
                        project_dir.display(),
                        doc_path.display()
                    )
                })?;
            }
            run_pipeline_live(&sources, &config, params, Some(&cc), executor).await?
        }
        RunMode::Weave => {
            // Use the project's transcript cache when it exists.
            let cc = cache::CacheConfig::new(project_dir, cache::CacheMode::Reuse);
            let cache_config = cc.cache_dir.is_dir().then_some(&cc);
            run_pipeline_weave(&sources, params, cache_config).await?
        }
    };

    Ok(DocRun {
        doc_path: doc_path.to_path_buf(),
        source,
        doc,
        result,
    })
}

/// Weave without executing and write only the output files that do not yet
/// exist, next to the document.
///
/// MISSING files only: a file already on disk is the committed baseline that
/// `check` compares against, and overwriting it with a weave-mode copy would
/// manufacture drift.
///
/// Never fails a run. A document that cannot be woven will fail the real run
/// a moment later with a better message, so a staging error is swallowed
/// rather than pre-empting it.
async fn stage_woven_files(doc_path: &Path, sources: &[(&str, &str)], params: &[(String, String)]) {
    let project_dir = project_dir_of(doc_path);
    let cc = cache::CacheConfig::new(project_dir, cache::CacheMode::Reuse);
    let cache_config = cc.cache_dir.is_dir().then_some(&cc);
    let Ok(result) = run_pipeline_weave(sources, params, cache_config).await else {
        return;
    };
    let _ = write_files_if_absent(&result.files, project_dir);
}

/// Drop a file's read-only bit if it has one. Absent files and permission
/// systems that will not cooperate are both fine to ignore — the write that
/// follows reports the real problem with better context than this could.
fn clear_read_only(path: &Path) {
    let _ = crate::up::state::set_read_only(path, false);
}

/// Resolve a document-declared output path against `base`, refusing any path
/// that would land outside it.
///
/// A `<hick:file path=…>` value comes from the document — a file people send
/// each other and agents write, which is the same threat model that makes the
/// sandbox the default executor. An absolute path, or a `..` that climbs out
/// of the output directory, would let a document write anywhere the user can
/// without executing a single cell (`Path::join` silently DISCARDS `base`
/// when the right-hand side is absolute). So containment is checked
/// lexically, before anything touches the filesystem.
pub fn contained_output_path(base: &Path, rel_path: &str) -> Result<PathBuf> {
    use std::path::Component;
    let mut depth: i64 = 0;
    let mut escapes = false;
    for component in Path::new(rel_path).components() {
        match component {
            Component::Normal(_) => depth += 1,
            Component::CurDir => {}
            Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    escapes = true;
                    break;
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                escapes = true;
                break;
            }
        }
    }
    if escapes {
        bail!(
            "refusing output path '{rel_path}': a <hick:file path=…> must stay under \
             the document's output directory ({base}).\n  \
             Absolute paths and `..` components that climb out of it are refused: a \
             document is a file people share, and an escaping output path would let \
             it write anywhere on this machine without running a single cell.\n  \
             Next step: make the path relative to the document, e.g. \
             path=\"src/app.py\".",
            base = base.display()
        );
    }
    Ok(base.join(rel_path))
}

/// What one call to [`write_outputs_detailed`] did to the disk.
#[derive(Debug, Default)]
pub struct WrittenOutputs {
    /// Files written, sorted.
    pub written: Vec<PathBuf>,
    /// Files left exactly as they were, sorted: an artifact already on disk
    /// whose produced bytes came from a cell with no recording. See
    /// [`write_outputs_detailed`].
    pub preserved: Vec<PathBuf>,
    /// Paths a volume produced that this repository's `.gitignore` would
    /// ignore, and which were therefore not written, sorted. See
    /// [`volume_paths_to_skip`].
    pub ignored: Vec<String>,
}

/// A [`hick_literate::VolumeFlushHook`] that mirrors each cell's newly-known
/// output volume files under `.hick-cache/last-run/`, as they become known —
/// not the real output tree, and never the run's authoritative write.
///
/// The friction this answers: without it, a cell's real, already-succeeded
/// output volume is invisible on disk if a LATER cell in the same run then
/// fails — `write_outputs_detailed` never runs, because `run_pipeline_live`
/// never returns `Ok`. Writing straight to the real output tree here instead
/// was considered and rejected: `write_outputs_detailed` owns real
/// correctness properties an incremental, best-effort hook has no business
/// reimplementing in parallel — the local-history "before" snapshot
/// (`record_generated_writes`, which must run before ANY write reaches the
/// real tree, or its own before/after diff is corrupted by our own earlier
/// write), the missing-recording preservation check, and read-only clearing.
/// A scratch mirror under `.hick-cache/` — already gitignored by `hick
/// init`, already this tool's own bookkeeping location — sidesteps all of
/// that by never touching the real tree at all.
///
/// Best-effort: an I/O failure here is logged and otherwise ignored. It must
/// never abort a run over a debugging convenience.
fn volume_flush_mirror(project_dir: PathBuf) -> hick_literate::VolumeFlushHook {
    let base = project_dir.join(".hick-cache").join("last-run");
    Arc::new(move |vol_name, files| {
        for (rel_path, content) in files {
            let full = match contained_output_path(&base, rel_path) {
                Ok(p) => p,
                Err(e) => {
                    log::warn!("volume '{vol_name}': not mirroring '{rel_path}': {e}");
                    continue;
                }
            };
            if let Some(parent) = full.parent()
                && let Err(e) = std::fs::create_dir_all(parent)
            {
                log::warn!(
                    "volume '{vol_name}': could not create '{}' to mirror '{rel_path}': {e}",
                    parent.display()
                );
                continue;
            }
            let written = match content {
                FileContent::Text(s) => std::fs::write(&full, s),
                FileContent::Binary(data) => match data.to_bytes() {
                    Ok(bytes) => std::fs::write(&full, bytes),
                    Err(e) => {
                        log::warn!("volume '{vol_name}': not mirroring '{rel_path}': {e}");
                        continue;
                    }
                },
            };
            if let Err(e) = written {
                log::warn!(
                    "volume '{vol_name}': could not mirror '{rel_path}' to '{}': {e}",
                    full.display()
                );
            }
        }
    })
}

/// Write a run's output files under `out_dir` (default: the document's
/// directory). Returns the paths written.
pub fn write_outputs(run: &DocRun, out_dir: Option<&Path>) -> Result<Vec<PathBuf>> {
    Ok(write_outputs_detailed(run, out_dir)?.written)
}

/// Write a run's output files, reporting what was written and what was
/// deliberately left alone.
///
/// **An artifact already on disk is not overwritten with `[never run]`.** A
/// weave that cannot find a cell's recording weaves that cell as a marker,
/// and every file the cell fed carries the marker with it — so a `hick weave`
/// on a machine with no `.hick-cache` replaced a committed SVG with four
/// words while the app, which executes, kept rendering the chart. Disk and
/// app disagreed and nothing said so. The bytes on disk are the product of a
/// run that really happened; a weave that never ran anything has nothing
/// truer to put there, so it keeps its hands off. Same reasoning as
/// [`write_missing_outputs`]: never manufacture drift.
///
/// The **weave target is exempt** and is always written. It is not an
/// artifact of a run — it is this weave's own report, and a report that says
/// `[never run]` is telling the truth about the recordings it found. Keeping
/// it stale would be the lie.
///
/// A file that does NOT yet exist is written either way: there is nothing to
/// destroy, and the marker is then the honest content of a document that has
/// never been run.
pub fn write_outputs_detailed(run: &DocRun, out_dir: Option<&Path>) -> Result<WrittenOutputs> {
    let base = match out_dir {
        Some(dir) => dir.to_path_buf(),
        None => run
            .doc_path
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf(),
    };
    let missing_recording = run.result.outputs_missing_a_recording();
    let weave_target = run.doc.weave_path.as_deref();
    let skip = volume_paths_to_skip(run, &base);
    // Before anything reaches disk, and never after: whatever is there right
    // now is what a person would lose.
    record_generated_writes(run, &base, !run.result.transcripts.is_empty());
    let mut written = Vec::new();
    let mut preserved = Vec::new();
    for (rel_path, content) in &run.result.files {
        if skip.contains(rel_path) {
            continue;
        }
        let full = contained_output_path(&base, rel_path)?;
        if missing_recording.contains(rel_path)
            && Some(rel_path.as_str()) != weave_target
            && full.exists()
        {
            preserved.push(full);
            continue;
        }
        if let Some(parent) = full.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        // A woven output can be on disk read-only: `hick up` marks a fully
        // generated file that way so an editor refuses it before the user
        // types. Regenerating that file is exactly what is meant to overwrite
        // it, so clear the mark rather than failing on a permission error.
        clear_read_only(&full);
        match content {
            FileContent::Text(s) => std::fs::write(&full, s)
                .with_context(|| format!("failed to write {}", full.display()))?,
            FileContent::Binary(data) => std::fs::write(&full, data.to_bytes()?)
                .with_context(|| format!("failed to write {}", full.display()))?,
        }
        written.push(full);
    }
    written.sort();
    preserved.sort();
    let mut ignored: Vec<String> = skip.into_iter().collect();
    ignored.sort();
    Ok(WrittenOutputs {
        written,
        preserved,
        ignored,
    })
}

/// Paths a VOLUME produced that this repository's `.gitignore` would ignore.
///
/// A volume's output is everything a program left in a directory, and for a
/// compiled language most of that is build output. Flushing it wrote every
/// `obj/` the build touched back into the repository — one `hick run` of a
/// .NET project left a few hundred files nobody wanted, and the obvious
/// declaration to reach for (`input="src" output="src"`) is the one that does
/// it worst.
///
/// `hick ingest` has always filtered the same bytes through the same
/// `.gitignore`, for the same stated reason: a document that owns the source
/// must not carry build output. This makes the flush agree with the ingest.
///
/// **Only volume output is filtered.** A `hick:file path="target/x"` is a
/// document saying precisely which file to write, and an author who names an
/// ignored path meant it. A volume names a directory and inherits whatever
/// was in it, which is a different kind of statement.
///
/// A directory that is not a git repository filters nothing, exactly as the
/// ingest does there.
fn volume_paths_to_skip(run: &DocRun, base: &Path) -> std::collections::HashSet<String> {
    let mut from_volumes: Vec<String> = run
        .result
        .volume_outputs
        .values()
        .flat_map(|files| files.keys().cloned())
        .collect();
    if from_volumes.is_empty() {
        return std::collections::HashSet::new();
    }
    from_volumes.sort();
    from_volumes.dedup();
    match hick_literate::volume_state::gitignored(base, &from_volumes) {
        Ok(Some(ignored)) => ignored.into_iter().collect(),
        // Not a repository, or no git: nothing is filtered, which is what
        // happened before this existed.
        Ok(None) | Err(_) => std::collections::HashSet::new(),
    }
}

/// Leave a local-history stop before generated files are overwritten.
///
/// Called by BOTH output writers — this one and the `hick up` loop's
/// `WovenState::write_output` — because a rule about what reaches disk that
/// is applied to only one of them is a rule with a hole in it. That gap is
/// what once let a weave destroy committed artifacts.
///
/// The act is `Run` or `Weave`, which are shown and **compared**, never
/// reverted: the next run would undo the revert. Recording them anyway is the
/// point of "show the document before the run" — the actual bytes, not
/// "re-run and hope the inputs are the same".
fn record_generated_writes(run: &DocRun, base: &Path, executed: bool) {
    let root = run
        .doc_path
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();
    let writes: Vec<(PathBuf, Vec<u8>)> = run
        .result
        .files
        .iter()
        .filter_map(|(rel, content)| {
            let full = contained_output_path(base, rel).ok()?;
            let bytes = match content {
                FileContent::Text(s) => s.clone().into_bytes(),
                FileContent::Binary(data) => data.to_bytes().ok()?,
            };
            Some((full, bytes))
        })
        .collect();
    if writes.is_empty() {
        return;
    }
    let kind = if executed {
        hickory_workspace::history::ActKind::Run
    } else {
        hickory_workspace::history::ActKind::Weave
    };
    crate::history::record(
        &root,
        kind,
        Some(run.doc_path.display().to_string()),
        &writes,
    );
}

/// Write only the output files that are MISSING under `out_dir`, leaving any
/// file already there untouched. Returns the paths written.
///
/// This is for staging a document's woven files before execution so that a
/// cell can run a file its own document assembles. It must never overwrite:
/// a file already on disk is the committed baseline that `check` compares
/// freshly produced output against, and replacing it would manufacture drift.
pub fn write_missing_outputs(run: &DocRun, out_dir: Option<&Path>) -> Result<Vec<PathBuf>> {
    let base = match out_dir {
        Some(dir) => dir.to_path_buf(),
        None => run
            .doc_path
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf(),
    };
    write_files_if_absent(&run.result.files, &base)
}

/// The write half of [`write_missing_outputs`], over the file map alone, so
/// staging can reuse it without inventing a [`DocRun`].
fn write_files_if_absent(
    files: &std::collections::HashMap<String, FileContent>,
    base: &Path,
) -> Result<Vec<PathBuf>> {
    let mut written = Vec::new();
    for (rel_path, content) in files {
        let full = contained_output_path(base, rel_path)?;
        if full.exists() {
            continue;
        }
        if let Some(parent) = full.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        match content {
            FileContent::Text(s) => std::fs::write(&full, s)?,
            FileContent::Binary(data) => std::fs::write(&full, data.to_bytes()?)?,
        }
        written.push(full);
    }
    written.sort();
    Ok(written)
}

/// Collect check failures: cells with no baseline (unverifiable), unmet
/// expectations, and drift between produced outputs and the committed files
/// on disk.
///
/// Drift comparison is SKIPPED for a document that has an unverifiable cell.
/// A cell that could not run contributes nothing to the woven output, so
/// every file it feeds would "differ from disk" — drift reported as a
/// consequence of a missing baseline is noise that buries the real finding.
/// Expectations are unaffected: an unverifiable cell never evaluates one, so
/// the expectations that remain are all about cells that really ran.
pub fn check_failures(run: &DocRun, out_dir: Option<&Path>) -> Result<Vec<CheckFailure>> {
    let mut failures = Vec::new();
    for (cell, reason) in &run.result.never_run {
        failures.push(CheckFailure::Unverifiable {
            doc: run.doc_path.clone(),
            cell: cell.clone(),
            reason: reason.clone(),
        });
    }
    for outcome in &run.result.expectations {
        if !outcome.passed {
            failures.push(CheckFailure::Expectation(outcome.clone()));
        }
    }
    if !run.result.never_run.is_empty() {
        return Ok(failures);
    }

    let base = match out_dir {
        Some(dir) => dir.to_path_buf(),
        None => run
            .doc_path
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf(),
    };
    let volatile = volatile_outputs(&run.doc);
    for (rel_path, content) in &run.result.files {
        // A volatile output is a REPORT, not a reproducible artifact. Drift
        // checking asks "do these bytes reproduce", which is only a
        // meaningful question when the inputs are fixed. For a document over
        // live data — a corpus that grows, a dashboard, anything sampling the
        // world — the answer is always no, and a check that always fails
        // teaches people to ignore it.
        //
        // Expectations are unaffected: `hick:expect` asks "did the claim
        // hold", which stays meaningful over live data and is exactly how a
        // volatile document still gets verified.
        if volatile.contains(rel_path.as_str()) {
            continue;
        }
        let full = contained_output_path(&base, rel_path)?;
        let produced: Vec<u8> = match content {
            FileContent::Text(s) => s.clone().into_bytes(),
            FileContent::Binary(data) => data.to_bytes()?,
        };
        match std::fs::read(&full) {
            Ok(existing) if existing == produced => {}
            Ok(existing) => failures.push(CheckFailure::Drift {
                doc: run.doc_path.clone(),
                output_path: full,
                detail: describe_drift(&existing, &produced),
            }),
            Err(_) => failures.push(CheckFailure::Drift {
                doc: run.doc_path.clone(),
                output_path: full,
                detail: "output file missing on disk (run `hick run` and commit it)".to_string(),
            }),
        }
    }
    Ok(failures)
}

/// What specifically differs between the committed bytes and the freshly
/// produced ones — where the mismatch starts and how the lengths compare —
/// rather than only the fact that they differ.
///
/// Before this, `hick test`'s `DRIFTED` verdict named the file and nothing
/// else, which left an author with no way to see what it thought differed
/// short of diffing the files by hand outside the tool. Found wanting while
/// root-causing a real `DRIFTED` report against a tree that turned out to be
/// byte-identical across runs — the actual cause was cache-key instability
/// (`docs/guarantees/execution/a-volume-carries-what-the-repository-carries.md`),
/// not a bug in this comparison, but nothing in the message said so, and
/// `hick test --json`'s per-cell `status: "ok"` gave no reason to suspect the
/// comparison itself either.
fn describe_drift(existing: &[u8], produced: &[u8]) -> String {
    let common = existing
        .iter()
        .zip(produced.iter())
        .take_while(|(a, b)| a == b)
        .count();
    let len_note = match existing.len().cmp(&produced.len()) {
        std::cmp::Ordering::Equal => format!("both {} bytes", existing.len()),
        std::cmp::Ordering::Less => format!(
            "on disk is {} bytes, freshly produced is {} bytes ({} more)",
            existing.len(),
            produced.len(),
            produced.len() - existing.len()
        ),
        std::cmp::Ordering::Greater => format!(
            "on disk is {} bytes, freshly produced is {} bytes ({} fewer)",
            existing.len(),
            produced.len(),
            existing.len() - produced.len()
        ),
    };
    if common == existing.len().min(produced.len()) {
        // One is an exact byte-for-byte prefix of the other — there is no
        // "first differing byte" to report, only where they stop agreeing.
        return format!(
            "committed file differs from freshly produced output — identical for the \
             first {common} bytes, then one simply ends ({len_note})"
        );
    }
    // A line number, when both sides are valid UTF-8 — the common,
    // human-legible case (source, markdown, config) — is more useful than a
    // byte offset alone, but the offset is kept either way.
    let line = match (std::str::from_utf8(existing), std::str::from_utf8(produced)) {
        (Ok(e), Ok(_)) => Some(
            e.as_bytes()[..common]
                .iter()
                .filter(|&&b| b == b'\n')
                .count()
                + 1,
        ),
        _ => None,
    };
    match line {
        Some(n) => format!(
            "committed file differs from freshly produced output — first difference at \
             byte {common} (line {n}), {len_note}"
        ),
        None => format!(
            "committed file differs from freshly produced output — first difference at \
             byte {common}, {len_note}"
        ),
    }
}

/// XML entities that reached a command verbatim.
///
/// hick's no-escaping invariant means `&lt;` inside an exec body is five
/// literal characters handed to the shell, not `<`. That is the correct and
/// deliberate behaviour — but it is also the single most likely mistake for
/// anyone who has ever written XML, and it fails far from its cause: the
/// shell reports `Syntax error: "&" unexpected` and says nothing about
/// escaping. (Written by an agent that made exactly this mistake while
/// dogfooding, and spent a cycle on the shell error before seeing it.)
///
/// A warning, never an error: a document may legitimately discuss entities.
/// Documents ABOUT hick use the `h:` prefix, so their examples are text and
/// never reach here.
pub fn escaping_warnings(doc: &hick_lang::HickDocument) -> Vec<String> {
    const ENTITIES: &[(&str, char)] = &[
        ("&lt;", '<'),
        ("&gt;", '>'),
        ("&amp;", '&'),
        ("&quot;", '"'),
        ("&apos;", '\''),
    ];
    // Extensions where an entity is ordinary content rather than a mistake.
    // An HTML page full of `&amp;` is correct HTML, and warning about it
    // would teach people to ignore the warning that matters.
    const MARKUP: &[&str] = &[
        "html", "htm", "xml", "xhtml", "svg", "xsl", "xslt", "rss", "atom", "md", "markdown",
        "vue", "razor", "cshtml", "xaml", "plist", "resx",
    ];
    let mut out = Vec::new();
    for tag in doc.tags() {
        let (what, where_to) = match tag.name.as_str() {
            "exec" => ("exec body", "the shell receives those characters literally"),
            // A `hick:file` is where this mistake is worst. The shell at
            // least fails loudly and soon; a generated source file takes the
            // five literal characters, is written without complaint, and
            // fails later in a compiler that has never heard of hick. That
            // is a long way from the cause.
            "file" => {
                let path = tag.get_attribute("path").unwrap_or_default();
                let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
                if MARKUP.contains(&ext.as_str()) {
                    continue;
                }
                (
                    "file body",
                    "the generated file receives those characters literally",
                )
            }
            _ => continue,
        };
        let body = hick_lang::tag_text(tag);
        for (entity, literal) in ENTITIES {
            if body.contains(entity) {
                out.push(format!(
                    "line {}: {what} contains `{entity}` — hick does not \
                     unescape, so {where_to}. Write `{literal}` directly.",
                    tag.source_line
                ));
                break;
            }
        }
    }
    out
}

/// Diagrams whose claim nothing checks, and diagrams claiming a check that
/// does not exist.
///
/// A picture of a system is an assertion — "these are the layers, this one
/// never calls that one" — and the reason architecture diagrams are all
/// wrong is that nothing ever re-reads them. `asserts` is how a diagram names
/// the cells that prove it:
///
/// ```text
/// <hick:diagram renderer="mermaid" asserts="#no-back-edges">
/// ```
///
/// Two failure modes, and they need different words. A diagram naming an id
/// that is nowhere in the document is a BROKEN reference — the author meant
/// to be checked and is not, usually because a cell was renamed. A diagram
/// with no `asserts` at all is an UNVERIFIED drawing: legal, sometimes
/// deliberate (a sketch of something outside this repository), and worth
/// saying out loud exactly once so nobody mistakes it for a checked one.
///
/// Both stay warnings. Whether a picture needs proof is the author's call,
/// and a tool that refused to weave an unproven sketch would just teach
/// people to draw somewhere else.
pub fn diagram_assertion_warnings(doc: &hick_lang::HickDocument) -> Vec<String> {
    let ids: std::collections::HashSet<&str> =
        doc.tags().filter_map(|t| t.get_attribute("id")).collect();
    let mut out = Vec::new();
    for tag in doc.tags().filter(|t| t.name == "diagram") {
        let line = tag.source_line;
        // A graph scene that does not parse weaves as its raw JSON — legal,
        // but the author should hear about it here, beside the assertion
        // warnings, rather than discover an unreadable fence later. A body
        // holding a `<hick:paste>` is checked after resolution instead, which
        // only the run can do.
        if tag.get_attribute("renderer") == Some(hick_literate::scene::GRAPH_RENDERER)
            && !tag.child_tags().any(|t| t.name == "paste")
        {
            match hick_literate::scene::parse_scene(&tag.text_content()) {
                Ok(scene) => {
                    for warning in hick_literate::scene::scene_warnings(&scene) {
                        out.push(format!("line {line}: {warning}"));
                    }
                }
                Err(err) => out.push(format!(
                    "line {line}: this graph diagram's body is not a scene \
                     ({err}), so it will weave as raw JSON instead of a \
                     picture. A scene is {{\"nodes\": …, \"edges\": …, \
                     \"layout\": …}} — see \
                     docs/specs/freeform/a-diagram-you-can-drag.md."
                )),
            }
        }
        let Some(asserts) = tag.get_attribute("asserts") else {
            out.push(format!(
                "line {line}: this diagram asserts nothing, so nothing will \
                 fail when it stops being true. Add `asserts=\"#id\"` naming \
                 the cell(s) that prove it, or leave it as a deliberate sketch."
            ));
            continue;
        };
        for selector in asserts.split_whitespace() {
            let Some(id) = selector.strip_prefix('#') else {
                out.push(format!(
                    "line {line}: `asserts` takes `#id` selectors, so `{selector}` \
                     matches nothing. Give the proving cell an `id` and name it here."
                ));
                continue;
            };
            if !ids.contains(id) {
                out.push(format!(
                    "line {line}: this diagram says it is proved by `#{id}`, but \
                     no tag in this document has that id — the check it names \
                     does not exist. Renaming a cell without renaming the \
                     reference is the usual cause."
                ));
            }
        }
    }
    out
}

/// Claims whose standing is missing, unknown, or unfalsifiable as written.
///
/// A `<hick:claim>` is an assertion ABOUT an assertion: nothing verifies that
/// Sam is an expert, and nothing ever will. What can be checked is whether the
/// marking is meaningful enough to be worth reading — which is what these
/// warnings are for. See `docs/specs/freeform/provenance-and-standing.md`.
///
/// All of these are **warnings, never errors**, following the same reasoning as
/// the diagram assertions: a tool that refused to weave an imperfectly marked
/// claim would only teach people to stop marking claims, and an unused marking
/// system is worse than none because it makes the marked subset look complete.
pub fn claim_warnings(doc: &hick_lang::HickDocument) -> Vec<String> {
    let extra: Vec<String> = doc
        .frontmatter
        .as_ref()
        .map(|fm| fm.list("standings"))
        .unwrap_or_default();
    let allowed: Vec<&str> = hick_lang::STANDINGS
        .iter()
        .copied()
        .chain(extra.iter().map(String::as_str))
        .collect();
    let vocabulary = allowed.join(", ");

    let mut out = Vec::new();
    for tag in doc.tags().filter(|t| t.name == "claim") {
        let line = tag.source_line;

        if tag
            .get_attribute("by")
            .filter(|by| !by.is_empty())
            .is_none()
        {
            out.push(format!(
                "line {line}: this claim says nothing about who is making it. \
                 Add `by=\"name\"` — an unattributed claim is indistinguishable \
                 from the document's own prose, which is what marking it was \
                 supposed to prevent."
            ));
        }

        match tag.get_attribute("standing").filter(|s| !s.is_empty()) {
            None => out.push(format!(
                "line {line}: this claim declares no `standing`, so it says \
                 someone asserted something without saying on what footing. \
                 Use one of: {vocabulary}."
            )),
            Some(standing) if !allowed.contains(&standing) => out.push(format!(
                "line {line}: `standing=\"{standing}\"` is not a standing this \
                 document knows. Use one of: {vocabulary} — or add \"{standing}\" \
                 to a `standings:` list in this document's frontmatter if it is \
                 a distinction your notes really make."
            )),
            Some("expert")
                if tag
                    .get_attribute("scope")
                    .filter(|s| !s.is_empty())
                    .is_none() =>
            {
                out.push(format!(
                    "line {line}: this claims expertise without a `scope`. \
                     Expertise is never global, and an expert speaking outside \
                     their scope is exactly what this marking exists to make \
                     visible — add `scope=\"…\"` naming what they are expert in."
                ));
            }
            Some(_) => {}
        }
    }
    out
}

/// Mounts at absolute paths, which the local executor cannot honour.
///
/// Both executors map `mount="v:/out"` to `/out` UNDER the container workdir
/// — `LocalExecutor` to `<tmp>/<container>/out`, `CanopyExecutor` to
/// `/hickory-work/out`, and there is a test in the canopy crate
/// (`mount_dir_mirrors_local_executor_semantics`) asserting they agree. A
/// command that then writes to the ABSOLUTE `/out` addresses the real
/// filesystem root and fails with `cannot create /out/...: Directory
/// nonexistent`, an error naming neither the mount nor the cause.
///
/// So this is not a dev/prod divergence: such a document is broken on every
/// executor. It stays a warning rather than an error because only the
/// commands know which paths they use, and this checks the declaration.
pub fn absolute_mount_warnings(doc: &hick_lang::HickDocument) -> Vec<String> {
    let mut out = Vec::new();
    for tag in doc.tags().filter(|t| t.name == "exec") {
        let Some(mount) = tag.get_attribute("mount") else {
            continue;
        };
        for spec in mount.split(',') {
            let Some((_vol, path)) = spec.split_once(':') else {
                continue;
            };
            let path = path.trim();
            if !path.starts_with('/') {
                continue;
            }
            // The spec `src:/project` is fine on its own — every executor
            // strips the slash and mounts under the workdir. What fails is a
            // COMMAND that then says `/project/...`, so warn only when one
            // does; a cell that addresses `project/...` relatively has done
            // nothing wrong and should not be told otherwise.
            let command = hick_lang::tag_text(tag);
            let absolute_use = command.contains(&format!("{path}/"))
                || command.contains(&format!("{path} "))
                || command.trim_end().ends_with(path);
            if !absolute_use {
                continue;
            }
            let rel = path.trim_start_matches('/');
            out.push(format!(
                "line {}: mounts at `{path}`, but mounts resolve UNDER the \
                 container workdir — `{path}` becomes `<workdir>{path}`. \
                 Commands must address it relatively (`{rel}/...`); an \
                 absolute `{path}` reaches the real filesystem root and fails.",
                tag.source_line
            ));
        }
    }
    out
}

/// Input volumes that carry a document's own *unstable* output back into
/// the cell.
///
/// A cell's recording is keyed by the digest of what it mounts, so a volume
/// whose directory contains a file this document writes puts that file inside
/// the key of the cell that writes it. When the file's bytes change on every
/// run, the key changes on every run, no recording of the cell is ever
/// findable again, and the next weave writes `[never run]` over the output of
/// a run that really happened.
///
/// **Only unstable outputs count**, which is the whole difficulty of stating
/// this. Mounting a directory that holds a `hick:file` the document assembles
/// from literal text is not a hazard — it is the central move of literate
/// programming, a cell running a script its own document wrote, and those
/// bytes are the same on every run. Two kinds are not:
///
/// - **The weave target.** It carries the cell's own transcript, so running
///   the cell changes it, so the next run's key differs. This is the one that
///   was actually hit: a measurement cell mounting `.`.
/// - **A `hick:file` fed by a cell.** Its bytes are a run's output, and a run
///   is only as reproducible as what it ran.
///
/// A warning rather than an error: a document measuring its own weave is
/// doing something legitimate and merely unstable, and the fix — mount what
/// the cell reads, not the folder it lives in — is not always available.
pub fn self_mounting_warnings(doc: &hick_lang::HickDocument) -> Vec<String> {
    let mut unstable: Vec<(String, &'static str)> = doc
        .tags()
        .filter(|t| t.name == "file")
        // A file whose body is a cell's output, not literal text.
        .filter(|t| t.child_tags().any(|c| c.name == "exec"))
        .filter_map(|t| t.get_attribute("path"))
        .map(|p| (p.trim().to_string(), "which a cell in this document fills"))
        .collect();
    if let Some(weave) = doc.weave_path.as_deref()
        && weave != hick_lang::WEAVE_NONE
    {
        unstable.push((
            weave.to_string(),
            "this document's own weave, which carries the cell's transcript",
        ));
    }

    let mut out = Vec::new();
    for tag in doc.tags().filter(|t| t.name == "volume") {
        let Some(input) = tag.get_attribute("input") else {
            continue;
        };
        let dir = input.trim().trim_end_matches('/');
        // `.` is the whole folder; anything else claims a subtree.
        let covers = |path: &str| -> bool {
            if dir.is_empty() || dir == "." {
                return !path.starts_with("../");
            }
            path.strip_prefix(dir)
                .is_some_and(|rest| rest.starts_with('/'))
        };
        let Some((clash, why)) = unstable.iter().find(|(p, _)| covers(p)) else {
            continue;
        };
        out.push(format!(
            "line {}: the input volume `{}` carries `{clash}`, {why}. A \
             cell's recording is keyed by what it mounts, so that output is \
             inside the key of the cell producing it: every run changes the \
             key and the next weave reports the cell as never run. Mount what \
             the cell reads, not the folder it lives in.",
            tag.source_line,
            input.trim()
        ));
    }
    out
}

/// A path this document has already ingested (a `hick:ingested` block's
/// child `hick:file` elements — the record of what a run actually produced)
/// that a SEPARATE, top-level `<hick:file path="…">` also declares.
///
/// `hick:file` content is written to host disk in one upfront pass before
/// any `hick:exec` cell runs, with no regard for document order. Once a path
/// is recorded inside `hick:ingested`, a second top-level declaration of the
/// exact same path either gets silently reverted (if the scaffolder cell
/// that produced it runs again with `--force`) or leaves the author with two
/// declarations disagreeing about what the file contains — the shape hit
/// authoring a tutorial that tried to give an ingested file an earlier,
/// smaller "stub" version via a second `hick:file`.
///
/// Deliberately an EXACT path match, not a directory/prefix match: an output
/// volume's prefix is an ordinary directory that legitimately holds many
/// hand-authored files alongside the ingested ones (this is the normal
/// shape, not a hazard — an earlier, prefix-based version of this check
/// warned on every one of them and was corrected after running it against a
/// real ingest-based document). A bare scaffolder cell that has not been
/// ingested yet cannot be checked this way either: its future output
/// filenames are not known until it runs, which is exactly the run this
/// check happens before.
///
/// A warning rather than a hard error, matching `self_mounting_warnings`:
/// the author may already know which declaration should win.
pub fn output_collision_warnings(doc: &hick_lang::HickDocument) -> Vec<String> {
    let ingested: HashMap<&str, usize> = doc
        .tags()
        .filter(|t| t.name == "exec")
        .flat_map(|t| t.child_tags())
        .filter(|t| t.name == "ingested")
        .flat_map(|t| t.child_tags())
        .filter(|t| t.name == "file")
        .filter_map(|t| t.get_attribute("path").map(|p| (p.trim(), t.source_line)))
        .collect();
    if ingested.is_empty() {
        return Vec::new();
    }

    doc.tags()
        .filter(|t| t.name == "file")
        .filter_map(|t| t.get_attribute("path").map(|p| (p.trim(), t.source_line)))
        .filter_map(|(path, line)| {
            let ingested_line = *ingested.get(path)?;
            Some(format!(
                "line {line}: `<hick:file path=\"{path}\">` declares a path \
                 already ingested at line {ingested_line}. `hick:file` content \
                 is written to disk before any cell runs, so this second \
                 declaration either gets silently reverted the next time the \
                 scaffolder cell runs with `--force`, or leaves the two \
                 declarations disagreeing about what the file contains. Drop \
                 this declaration and let the ingested content own the path, \
                 or re-ingest to bring it up to date instead of hand-editing \
                 a competing copy."
            ))
        })
        .collect()
}

/// Containers declaring an image the local executor will not honour.
///
/// `LocalExecutor` records `image=` and ignores it: cells run against the
/// HOST toolchain. A document declaring `image="duckdb/duckdb:v1.5.5"` then
/// fails with `duckdb: not found` on a machine without duckdb — an error
/// that reads like a broken document rather than a missing local dependency,
/// and it is the first thing a new user hits after `cargo install`.
///
/// One line per document, listing the images, so the reader knows which
/// tools they are expected to have.
pub fn ignored_image_warning(doc: &hick_lang::HickDocument) -> Option<String> {
    let mut images: Vec<&str> = doc
        .tags()
        .filter(|t| t.name == "container")
        .filter_map(|t| t.get_attribute("image"))
        .filter(|i| *i != "host")
        .collect();
    images.sort_unstable();
    images.dedup();
    if images.is_empty() {
        return None;
    }
    Some(format!(
        "the local executor ignores image= — these cells run against your HOST \
         toolchain, so the tools in {} must be installed locally. Declared: {}",
        if images.len() == 1 {
            "this image"
        } else {
            "these images"
        },
        images.join(", ")
    ))
}

/// Output paths this document declares `volatile="true"`, excluded from
/// drift comparison.
///
/// Both `<hick:file path="..." volatile="true">` and a volatile document
/// root (`<hick:doc weave="..." volatile="true">`, which marks the woven
/// markdown) are honoured.
fn volatile_outputs(doc: &hick_lang::HickDocument) -> HashSet<String> {
    fn is_volatile(tag: &hick_lang::HickTag) -> bool {
        matches!(tag.get_attribute("volatile"), Some("true"))
    }
    let mut out = HashSet::new();
    for tag in doc.find_tags("file") {
        if is_volatile(tag)
            && let Some(path) = tag.get_attribute("path")
        {
            out.insert(path.to_string());
        }
    }
    // The root `hick:doc` is the document itself, not one of its child
    // nodes, so its attributes are not reachable via `find_tags` — they are
    // parsed onto `HickDocument`. Reading the root through `find_tags` looked
    // right and silently matched nothing.
    if doc.volatile
        && let Some(weave) = &doc.weave_path
    {
        out.insert(weave.clone());
    }
    out
}

/// Build the block model (`docs/specs/freeform/api.md`) for a run.
pub fn block_model(run: &DocRun) -> Vec<Block> {
    let never_run = run.result.never_run.clone();
    build_block_model(&BlockModelInput {
        doc: &run.doc,
        transcripts: &run.result.transcripts,
        expectations: &run.result.expectations,
        files: Some(&run.result.files),
        never_run: &never_run,
    })
}

/// The block model as the JSON body the server's render endpoint returns.
pub fn block_model_json(run: &DocRun) -> Result<serde_json::Value> {
    Ok(serde_json::json!({ "blocks": block_model(run) }))
}

/// A file a CELL wrote into an output volume, rather than one the document
/// weaves.
///
/// These have no byte-precise lineage and never will: their bytes are a
/// program's output, and no span of any document produced them. But "no
/// output named that" — which is what lineage said before this existed — is
/// the wrong answer to a real question. A generated file has a provenance,
/// just a coarser one: **this cell, in this document, wrote all of it.**
///
/// That distinction is the point. A reader looking at a source file needs to
/// know which of three things it is: text a person typed (`literal`), bytes a
/// foreign tool wrote and this document owns (`ingested`), or output a cell
/// produced on the way past (this). The first two are byte-precise and
/// editable; this one is neither, and saying so plainly is what stops someone
/// editing a file that regenerates over them.
pub struct GeneratedFile {
    /// The output volume the file landed in.
    pub volume: String,
    /// Containers whose cells wrote into that volume, in document order.
    pub cells: Vec<String>,
    /// Source lines of the `hick:exec` tags that mounted it.
    pub lines: Vec<usize>,
}

/// Whether `path` is written by one of this document's output volumes.
///
/// Matched on the volume's `output=` prefix, which is where the volume's
/// contents land — so `apiout` declared `output="src/Api"` claims
/// `src/Api/Endpoints.g.cs`.
pub fn generated_file(run: &DocRun, path: &str) -> Option<GeneratedFile> {
    let normalized = path.replace('\\', "/");
    let mut best: Option<(usize, String, String)> = None; // (prefix len, volume, output)
    for tag in run.doc.tags() {
        if tag.name != "volume" {
            continue;
        }
        let (Some(name), Some(output)) = (tag.get_attribute("name"), tag.get_attribute("output"))
        else {
            continue;
        };
        let prefix = output.trim_end_matches('/');
        // The longest matching declaration wins, so a narrow volume nested
        // inside a wide one is reported as the one that actually wrote it.
        if (normalized == prefix || normalized.starts_with(&format!("{prefix}/")))
            && best.as_ref().is_none_or(|(len, _, _)| prefix.len() > *len)
        {
            best = Some((prefix.len(), name.to_string(), output.to_string()));
        }
    }
    let (_, volume, _) = best?;

    let mut cells = Vec::new();
    let mut lines = Vec::new();
    for tag in run.doc.tags() {
        if tag.name != "exec" {
            continue;
        }
        let mounts = tag.get_attribute("mount").unwrap_or_default();
        let mounts_it = mounts
            .split(',')
            .filter_map(|entry| entry.trim().split(':').next())
            .any(|v| v == volume);
        if mounts_it {
            if let Some(container) = tag.get_attribute("container") {
                cells.push(container.to_string());
            }
            lines.push(tag.source_line);
        }
    }
    Some(GeneratedFile {
        volume,
        cells,
        lines,
    })
}

/// Byte-precise lineage of one generated output file: the api.md
/// `Provenance[]` shape (identical to what the server's
/// `GET /api/docs/:id/outputs/file` returns).
pub fn output_lineage(run: &DocRun, output_path: &str) -> Result<Vec<hickory_lineage::Provenance>> {
    let map = run.result.provenance_maps.get(output_path).ok_or_else(|| {
        let mut available: Vec<&str> = run
            .result
            .provenance_maps
            .keys()
            .map(String::as_str)
            .collect();
        available.sort();
        anyhow::anyhow!(
            "no output named '{output_path}' — available outputs: {}",
            if available.is_empty() {
                "(none)".to_string()
            } else {
                available.join(", ")
            }
        )
    })?;
    let mut provenance = hickory_lineage::from_provenance_map(map);
    // Bytes a CELL produced: the provenance map knows the cell by its source
    // line, and the document knows the cell's span at that line. Joining the
    // two here gives exec output an origin the ribbons can draw — the cell
    // itself — instead of `synthetic`, which told a reader "nowhere". A paste
    // of `#cell-id` arrives here too, so the quoted number's ribbon ends at
    // the computation.
    let exec_spans: std::collections::HashMap<usize, (usize, usize)> = {
        let mut out = std::collections::HashMap::new();
        let mut stack: Vec<&hick_lang::HickTag> = run.doc.tags().collect();
        while let Some(tag) = stack.pop() {
            if tag.name == "exec"
                && let Some(span) = tag.source_span
                && span.file_id.is_none()
            {
                out.entry(tag.source_line).or_insert((span.start, span.end));
            }
            for child in &tag.children {
                if let hick_lang::HickNode::Tag(t) = child {
                    stack.push(t);
                }
            }
        }
        out
    };
    for (entry, span) in provenance.iter_mut().zip(map.spans().iter()) {
        if let hick_exec::node::SourceOrigin::Exec { tag_line, .. } = &span.origin
            && matches!(entry.origin, hickory_lineage::Origin::Synthetic)
            && let Some((start, end)) = exec_spans.get(tag_line)
        {
            entry.origin = hickory_lineage::Origin::Exec {
                doc_path: run.doc_path.display().to_string(),
                span: (*start, *end),
            };
        }
    }
    Ok(provenance)
}

/// Human-readable lineage for every agent-authored range of one output.
///
/// One entry per `Origin::Agent` provenance span, in output order:
///
/// ```text
/// foo.rs:42 ← session abc123 turn 7 · committed by … · reasoning not available to you
/// ```
///
/// Authorship is derived from `git blame` on the document span the bytes map
/// to, never from a stored `author` field. An unreadable session, an absent
/// span, and an uncommitted working tree are all reported, never errors — see
/// `docs/guarantees/lineage/agent-lineage-degrades-without-a-session.md`.
pub fn agent_lineage_report(run: &DocRun, output_path: &str) -> Result<Vec<AgentLineage>> {
    let provenance = output_lineage(run, output_path)?;
    let content = match run.result.files.get(output_path) {
        Some(FileContent::Text(s)) => s.as_str(),
        _ => "",
    };
    // The document is the thing git blames; sessions live beside it.
    let project_dir = run
        .doc_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let doc_name = run
        .doc_path
        .file_name()
        .map(PathBuf::from)
        .unwrap_or_else(|| run.doc_path.clone());

    Ok(provenance
        .iter()
        .filter_map(|p| {
            let (session, turn) = p.origin.agent()?;
            // A span in the document gives a line to blame. Absent one, the
            // report still carries session, turn, and an honest "unknown".
            let doc_span = p
                .origin
                .location()
                .map(|(_, start, _)| agent_lineage::line_of(&run.source, start))
                .map(|line| (doc_name.as_path(), line));
            Some(agent_lineage::describe(
                &project_dir,
                output_path,
                agent_lineage::line_of(content, p.start),
                session,
                turn,
                doc_span,
            ))
        })
        .collect())
}

// Protects docs/guarantees/execution/output-paths-stay-inside-the-project.md
#[cfg(test)]
mod contained_output_path_tests {
    use super::contained_output_path;
    use std::path::Path;

    #[test]
    fn ordinary_relative_paths_resolve_under_base() {
        let base = Path::new("/tmp/doc");
        assert_eq!(
            contained_output_path(base, "src/app.py").unwrap(),
            base.join("src/app.py")
        );
        // `..` that stays inside the base is allowed.
        assert_eq!(
            contained_output_path(base, "a/../b.txt").unwrap(),
            base.join("a/../b.txt")
        );
        assert_eq!(
            contained_output_path(base, "./c.txt").unwrap(),
            base.join("./c.txt")
        );
    }

    #[test]
    fn absolute_paths_are_refused() {
        // Path::join would DISCARD the base for these — the exact behaviour
        // the guard exists to remove.
        let base = Path::new("/tmp/doc");
        let err = contained_output_path(base, "/etc/cron.d/x").unwrap_err();
        assert!(err.to_string().contains("refusing output path"), "{err}");
    }

    #[test]
    fn parent_escapes_are_refused() {
        let base = Path::new("/tmp/doc");
        for p in ["../evil.sh", "a/../../evil.sh", ".."] {
            let err = contained_output_path(base, p).unwrap_err();
            assert!(
                err.to_string().contains("refusing output path"),
                "{p}: {err}"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn drive_prefixes_are_refused() {
        let base = Path::new("C:\\work\\doc");
        assert!(contained_output_path(base, "C:\\evil.txt").is_err());
    }
}

#[cfg(test)]
mod self_mounting_tests {
    use super::self_mounting_warnings;

    fn doc(source: &str) -> hick_lang::HickDocument {
        hick_lang::parse(source).expect("the document parses")
    }

    #[test]
    fn mounting_the_whole_folder_warns_about_the_weave_target() {
        // The archetype, and how the warehouse's measurement cell was
        // written: `input="."` in a document that weaves markdown beside
        // itself. It worked for exactly as long as nobody had the app open.
        let warnings = self_mounting_warnings(&doc(r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="report.md">
<hick:volume name="all" input="." />
<hick:exec container="c" mount="all:project">
wc -l project/*
</hick:exec>
</hick:doc>
"#));
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("report.md"), "{}", warnings[0]);
        assert!(
            warnings[0].contains("keyed by what it mounts"),
            "{}",
            warnings[0]
        );
    }

    #[test]
    fn a_file_a_cell_fills_is_unstable_and_warns() {
        let warnings = self_mounting_warnings(&doc(r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:volume name="t" input="tools" />
<hick:file path="tools/list.txt">
<hick:exec container="c" show="output">
ls
</hick:exec>
</hick:file>
</hick:doc>
"#));
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("tools/list.txt"), "{}", warnings[0]);
    }

    #[test]
    fn a_script_the_document_types_and_the_cell_runs_is_the_point_of_all_this() {
        // The central move of literate programming: a document writes a
        // script from literal text and mounts the directory so a cell can run
        // it. The bytes are identical on every run, the key is stable, and
        // warning here would fire on nearly every document there is.
        let warnings = self_mounting_warnings(&doc(r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:volume name="t" input="tools" />
<hick:file path="tools/gen.py">print(1)
</hick:file>
<hick:exec container="c" mount="t:tools">
python3 tools/gen.py
</hick:exec>
</hick:doc>
"#));
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn a_volume_that_carries_nothing_this_document_writes_is_silent() {
        // The ordinary shape — mount what the cell reads — must never warn,
        // or the warning becomes noise and stops being read.
        let warnings = self_mounting_warnings(&doc(r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="report.md">
<hick:volume name="src" input="src" />
<hick:volume name="out" output="generated" />
<hick:exec container="c" mount="src:src,out:out">
echo hi
</hick:exec>
</hick:doc>
"#));
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn a_prefix_that_is_not_a_directory_boundary_is_not_a_match() {
        // `tools` must not claim `toolsmith.md`.
        let warnings = self_mounting_warnings(&doc(r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="toolsmith.md">
<hick:volume name="t" input="tools" />
</hick:doc>
"#));
        assert!(warnings.is_empty(), "{warnings:?}");
    }
}

// Protects docs/guarantees/verification/a-drift-report-names-what-differs.md
#[cfg(test)]
mod describe_drift_tests {
    use super::describe_drift;

    #[test]
    fn a_middle_byte_difference_names_the_offset_and_line() {
        let existing = b"line one\nline two\nline three\n";
        let produced = b"line one\nline TWO\nline three\n";
        let detail = describe_drift(existing, produced);
        assert!(detail.contains("byte 14"), "{detail}");
        assert!(detail.contains("line 2"), "{detail}");
        assert!(detail.contains("both 29 bytes"), "{detail}");
    }

    #[test]
    fn one_side_ending_early_is_reported_as_a_prefix_not_a_byte_offset() {
        let existing = b"line one\nline two\n";
        let produced = b"line one\nline two\nline three\n";
        let detail = describe_drift(existing, produced);
        assert!(
            detail.contains("identical for the first 18 bytes"),
            "{detail}"
        );
        assert!(detail.contains("11 more"), "{detail}");
        // Not a byte-offset claim: there is no differing byte, only an end.
        assert!(!detail.contains("first difference at byte"), "{detail}");
    }

    #[test]
    fn non_utf8_content_still_reports_a_byte_offset_with_no_line_number() {
        let existing = [0u8, 1, 2, 255];
        let produced = [0u8, 1, 9, 255];
        let detail = describe_drift(&existing, &produced);
        assert!(detail.contains("byte 2"), "{detail}");
        assert!(!detail.contains("line"), "{detail}");
    }
}

// Protects docs/guarantees/execution/an-output-path-declared-twice-warns-before-it-runs.md
#[cfg(test)]
mod output_collision_tests {
    use super::output_collision_warnings;

    fn doc(source: &str) -> hick_lang::HickDocument {
        hick_lang::parse(source).expect("the document parses")
    }

    #[test]
    fn a_second_hick_file_at_an_already_ingested_path_warns() {
        // The exact shape that bit the ingest-based tutorial: an author
        // tries to give an ingested file an earlier "stub" version via a
        // second, top-level `hick:file` at the same path.
        let warnings = output_collision_warnings(&doc(r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:volume name="project" output="app" />
<hick:exec container="c" mount="project:out">
dotnet new console -o out
<hick:ingested from="#x" sha256="abc" at="2026-08-24" files="1" skipped="0">
<hick:file path="app/Program.cs">real content</hick:file>
</hick:ingested>
</hick:exec>
<hick:file path="app/Program.cs">stub</hick:file>
</hick:doc>
"##));
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("app/Program.cs"), "{}", warnings[0]);
        assert!(warnings[0].contains("already ingested"), "{}", warnings[0]);
    }

    #[test]
    fn hand_authored_files_beside_an_ingested_one_are_silent() {
        // The false positive an earlier, prefix-based version of this check
        // produced on the real ingest-based tutorial: hand-authored files
        // living in the SAME output directory as an ingested scaffold, at
        // DIFFERENT paths, are the ordinary shape and must never warn.
        let warnings = output_collision_warnings(&doc(r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:volume name="project" output="app" />
<hick:exec container="c" mount="project:out">
dotnet new console -o out
<hick:ingested from="#x" sha256="abc" at="2026-08-24" files="1" skipped="0">
<hick:file path="app/Program.cs">real content</hick:file>
</hick:ingested>
</hick:exec>
<hick:file path="app/Todo.cs">class Todo {}</hick:file>
<hick:file path="app/TodoStore.cs">class TodoStore {}</hick:file>
</hick:doc>
"##));
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn a_document_with_no_ingest_at_all_is_silent() {
        // A bare scaffolder cell that has not been ingested yet cannot be
        // checked this way — its future output filenames are not known
        // until it runs, which is exactly the run this check happens before.
        let warnings = output_collision_warnings(&doc(r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:volume name="project" output="app" />
<hick:file path="app/Program.cs">stub</hick:file>
<hick:exec container="c" mount="project:out">
dotnet new console -o out
</hick:exec>
</hick:doc>
"#));
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn an_ingested_files_own_child_path_does_not_self_collide() {
        // The ingested `hick:file` is the one and only declaration of its
        // path — nothing else in the document repeats it — so there is
        // nothing to warn about by itself.
        let warnings = output_collision_warnings(&doc(r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:volume name="project" output="app" />
<hick:exec container="c" mount="project:out">
dotnet new console -o out
<hick:ingested from="#x" sha256="abc" at="2026-08-24" files="1" skipped="0">
<hick:file path="app/Program.cs">real content</hick:file>
</hick:ingested>
</hick:exec>
</hick:doc>
"##));
        assert!(warnings.is_empty(), "{warnings:?}");
    }
}

#[cfg(test)]
mod project_dir_tests {
    use super::project_dir_of;
    use std::path::Path;

    /// `parent()` of a bare filename is `Some("")`, not `None`.
    ///
    /// The trap that made every `hick run <bare-filename>` on the machine
    /// share one scratch directory: `unwrap_or(".")` never fires for
    /// `Some("")`, so the empty path went straight through to a hash that is
    /// the same constant for everybody.
    #[test]
    fn a_document_named_without_a_directory_lives_in_the_current_one() {
        assert_eq!(project_dir_of(Path::new("d.hick")), Path::new("."));
        assert_eq!(
            project_dir_of(Path::new("notes/d.hick")),
            Path::new("notes")
        );
        assert_eq!(project_dir_of(Path::new("/tmp/d.hick")), Path::new("/tmp"));
        // Never the empty path, whatever it is handed.
        for name in ["d.hick", "notes/d.hick", "/tmp/d.hick", ""] {
            assert!(
                !project_dir_of(Path::new(name)).as_os_str().is_empty(),
                "{name} produced an empty project directory"
            );
        }
    }
}
