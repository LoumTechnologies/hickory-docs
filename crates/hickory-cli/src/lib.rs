//! Library core of the `hickory` CLI.
//!
//! The binary (`src/main.rs`) is a thin argument parser over these functions,
//! so the server can drive the same run/check/weave/render code paths via
//! library calls instead of shelling out.

pub mod init;

/// `hickory init` entry points: idempotent local git-repo setup.
pub use init::{InitReport, print_init_report, run_init};

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context as _, Result, bail};
use hick_exec::node::FileContent;
use hick_literate::render::{Block, BlockModelInput, build_block_model};
use hick_literate::{
    Executor, LocalExecutor, PipelineConfig, PipelineResult, cache, expand_path_arg,
    run_pipeline_live, run_pipeline_weave,
};

/// Which executor backend to use, from `HICKORY_EXECUTOR`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutorChoice {
    Local,
    Canopy,
}

impl ExecutorChoice {
    /// Read `HICKORY_EXECUTOR` (default: `local`).
    pub fn from_env() -> Result<Self> {
        match std::env::var("HICKORY_EXECUTOR").as_deref() {
            Err(_) | Ok("") | Ok("local") => Ok(ExecutorChoice::Local),
            Ok("canopy") => Ok(ExecutorChoice::Canopy),
            Ok(other) => {
                bail!("unknown HICKORY_EXECUTOR value '{other}' (expected \"local\" or \"canopy\")")
            }
        }
    }

    /// Build the executor. Canopy reads its `CANOPY_*` env config here
    /// (connection to the node agent is lazy — first use).
    pub fn build(self) -> Result<Arc<dyn Executor>> {
        match self {
            ExecutorChoice::Local => Ok(Arc::new(LocalExecutor::new()?)),
            ExecutorChoice::Canopy => Ok(Arc::new(
                hickory_executor_canopy::CanopyExecutor::from_env()?,
            )),
        }
    }
}

/// How to obtain transcripts for a document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    /// Execute every exec block through the executor.
    Execute,
    /// Never execute: use cached transcripts where present, mark the rest
    /// never-run.
    Weave,
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
    /// an LLM wrote the passage, so the fix is `hickory refresh`, not a re-run.
    StaleTransform {
        doc: PathBuf,
        line: usize,
        select: String,
        instruct: String,
    },
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
/// were correct where they were written. `hickory check docs/` should not
/// start failing the moment an agent runs in that tree.
///
/// Naming a session file EXPLICITLY still works — `hickory weave` on a
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

/// Run one document. Each document gets its own executor instance so
/// container namespaces never collide across documents.
pub async fn run_doc(
    doc_path: &Path,
    params: &[(String, String)],
    mode: RunMode,
    executor_choice: ExecutorChoice,
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
        && mode == RunMode::Execute
        && let Some(warning) = ignored_image_warning(&doc)
    {
        log::info!("{}: {warning}", doc_path.display());
    }

    let doc_name = doc_path.display().to_string();
    let sources = vec![(doc_name.as_str(), source.as_str())];

    let project_dir = doc_path.parent().unwrap_or(Path::new("."));

    let result = match mode {
        RunMode::Execute => {
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
            // through `hickory run`. One place, every caller.
            //
            // Guarantee:
            // docs/guarantees/execution/first-run-behaves-like-every-later-run.md
            stage_woven_files(doc_path, &sources, params).await;

            let executor = executor_choice.build()?;
            let config = PipelineConfig {
                working_dir: Some(project_dir.to_path_buf()),
                max_rounds: 1,
                on_exec: None,
            };
            run_pipeline_live(&sources, &config, params, None, executor).await?
        }
        RunMode::Weave => {
            // Use the project's transcript cache when it exists.
            let cc = cache::CacheConfig::new(project_dir, true, false);
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
    let cc = cache::CacheConfig::new(project_dir, true, false);
    let cache_config = cc.cache_dir.is_dir().then_some(&cc);
    let Ok(result) = run_pipeline_weave(sources, params, cache_config).await else {
        return;
    };
    let _ = write_files_if_absent(&result.files, project_dir);
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
        match content {
            FileContent::Text(s) => std::fs::write(&full, s)?,
            FileContent::Binary(data) => std::fs::write(&full, data.to_bytes()?)?,
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

/// Collect check failures: unmet expectations plus drift between produced
/// outputs and the committed files on disk.
pub fn check_failures(run: &DocRun, out_dir: Option<&Path>) -> Result<Vec<CheckFailure>> {
    let mut failures = Vec::new();
    for outcome in &run.result.expectations {
        if !outcome.passed {
            failures.push(CheckFailure::Expectation(outcome.clone()));
        }
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
                detail: "output file missing on disk (run `hickory run` and commit it)".to_string(),
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
    let never_run: HashSet<(String, usize)> = run.result.never_run.clone();
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
