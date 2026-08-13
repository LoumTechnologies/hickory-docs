//! Library core of the `hick` CLI.
//!
//! The binary (`src/main.rs`) is a thin argument parser over these functions,
//! so the server can drive the same run/check/weave/render code paths via
//! library calls instead of shelling out.

pub mod agent_cell_runner;
pub mod agent_lineage;
pub mod doc_tools;
pub mod init;
pub mod mcp;
pub mod serve;
pub mod up;

/// `hick init` entry points: idempotent local git-repo setup.
pub use init::{InitReport, print_init_report, run_init};

/// Lineage for agent-authored bytes: session + turn from provenance,
/// authorship from `git blame`, graceful degradation when the session is
/// private or absent.
pub use agent_lineage::{AgentLineage, Authorship, Reasoning};

/// The binary's [`hick_literate::agent_cell::AgentRunner`]: settles one
/// `<hick:agent>` DAG vertex through the real ReAct loop, or reports that no
/// provider key is available so the cell is unverifiable rather than fatal.
pub use agent_cell_runner::LlmAgentRunner;

use std::collections::HashSet;
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
    Docker,
    Canopy,
}

impl ExecutorChoice {
    /// The name this backend is known by — the `HICKORY_EXECUTOR` value, the
    /// `GET /api/executor` field, and what the `hick serve` banner prints.
    pub fn as_str(self) -> &'static str {
        match self {
            ExecutorChoice::Local => "local",
            ExecutorChoice::Docker => "docker",
            ExecutorChoice::Canopy => "canopy",
        }
    }

    /// Read `HICKORY_EXECUTOR` (default: `local`).
    pub fn from_env() -> Result<Self> {
        match std::env::var("HICKORY_EXECUTOR").as_deref() {
            Err(_) | Ok("") | Ok("local") => Ok(ExecutorChoice::Local),
            Ok("docker") => Ok(ExecutorChoice::Docker),
            Ok("canopy") => Ok(ExecutorChoice::Canopy),
            Ok(other) => bail!(
                "unknown HICKORY_EXECUTOR value '{other}' (expected \"local\", \"docker\", \
                 or \"canopy\")"
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
        match self {
            ExecutorChoice::Local => Ok(Arc::new(LocalExecutor::new()?)),
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
    let doc = hick_lang::parse(source)
        .map_err(|e| anyhow::anyhow!("parse error in {}: {e}", doc_path.display()))?;
    let mut out = Vec::new();
    for tag in doc.find_tags("transform") {
        let select = tag.get_attribute("select").unwrap_or_default().to_string();
        let instruct = tag
            .get_attribute("instruct")
            .unwrap_or_default()
            .to_string();
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
pub fn transform_input(doc: &hick_lang::HickDocument, select: &str) -> String {
    hick_lang::fragments_matching(doc, select)
        .iter()
        .map(|t| hick_lang::tag_text(t))
        .collect::<Vec<_>>()
        .join("")
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

/// Run one document. Each document gets its own executor instance so
/// container namespaces never collide across documents.
pub async fn run_doc_cached(
    doc_path: &Path,
    params: &[(String, String)],
    mode: RunMode,
    executor_choice: ExecutorChoice,
    cache_mode: CacheMode,
) -> Result<DocRun> {
    let source = std::fs::read_to_string(doc_path)
        .with_context(|| format!("failed to read {}", doc_path.display()))?;
    let doc = hick_lang::parse(&source)
        .map_err(|e| anyhow::anyhow!("parse error in {}: {e}", doc_path.display()))?;

    // Warn BEFORE executing. The failure this predicts kills the run, so a
    // warning emitted afterwards is a warning nobody ever sees.
    for warning in escaping_warnings(&doc) {
        log::warn!("{}: {warning}", doc_path.display());
    }
    // Not gated on the executor: both resolve mounts under the workdir.
    for warning in absolute_mount_warnings(&doc) {
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

    let project_dir = doc_path.parent().unwrap_or(Path::new("."));

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

            let executor = executor_choice.build().await?;
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
                max_rounds: 1,
                on_exec: None,
                agent_runner,
                max_agent_reprepares: 0,
                // `check` wants every cell with no baseline reported as
                // unverifiable; `run` wants the first one to stop the run.
                collect_unverifiable: mode == RunMode::Verify,
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
    let project_dir = doc_path.parent().unwrap_or(Path::new("."));
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

/// Write a run's output files under `out_dir` (default: the document's
/// directory). Returns the paths written.
pub fn write_outputs(run: &DocRun, out_dir: Option<&Path>) -> Result<Vec<PathBuf>> {
    let base = match out_dir {
        Some(dir) => dir.to_path_buf(),
        None => run
            .doc_path
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf(),
    };
    let mut written = Vec::new();
    for (rel_path, content) in &run.result.files {
        let full = base.join(rel_path);
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
    Ok(written)
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
        let full = base.join(rel_path);
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
        let full = base.join(rel_path);
        let produced: Vec<u8> = match content {
            FileContent::Text(s) => s.clone().into_bytes(),
            FileContent::Binary(data) => data.to_bytes()?,
        };
        match std::fs::read(&full) {
            Ok(existing) if existing == produced => {}
            Ok(_) => failures.push(CheckFailure::Drift {
                doc: run.doc_path.clone(),
                output_path: full,
                detail: "committed file differs from freshly produced output".to_string(),
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
    let mut out = Vec::new();
    for tag in doc.tags().filter(|t| t.name == "exec") {
        let body = hick_lang::tag_text(tag);
        for (entity, literal) in ENTITIES {
            if body.contains(entity) {
                out.push(format!(
                    "line {}: exec body contains `{entity}` — hick does not \
                     unescape, so the shell receives those characters \
                     literally. Write `{literal}` directly.",
                    tag.source_line
                ));
                break;
            }
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
    Ok(hickory_lineage::from_provenance_map(map))
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
