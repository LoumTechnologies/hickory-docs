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
            Ok(other) => bail!(
                "unknown HICKORY_EXECUTOR value '{other}' (expected \"local\" or \"canopy\")"
            ),
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
}

/// Expand a path argument (file or directory) into `.hick` documents.
pub fn expand_docs(path: &Path) -> Result<Vec<PathBuf>> {
    let (files, _config) = expand_path_arg(path)?;
    Ok(files)
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

    let doc_name = doc_path.display().to_string();
    let sources = vec![(doc_name.as_str(), source.as_str())];

    let project_dir = doc_path.parent().unwrap_or(Path::new("."));

    let result = match mode {
        RunMode::Execute => {
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
    for (rel_path, content) in &run.result.files {
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
                detail: "output file missing on disk (run `hickory run` and commit it)"
                    .to_string(),
            }),
        }
    }
    Ok(failures)
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
pub fn output_lineage(
    run: &DocRun,
    output_path: &str,
) -> Result<Vec<hickory_lineage::Provenance>> {
    let map = run.result.provenance_maps.get(output_path).ok_or_else(|| {
        let mut available: Vec<&str> =
            run.result.provenance_maps.keys().map(String::as_str).collect();
        available.sort();
        anyhow::anyhow!(
            "no output named '{output_path}' — available outputs: {}",
            if available.is_empty() { "(none)".to_string() } else { available.join(", ") }
        )
    })?;
    Ok(hickory_lineage::from_provenance_map(map))
}
