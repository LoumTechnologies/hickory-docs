//! Watch loop for `hick up` command.
//!
//! Monitors `.hick` source files for changes and re-runs the pipeline,
//! merging results with the previous snapshot via the merge orchestrator.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use log::{debug, error, info};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use hick_merge::{MergeOrchestrator, MergeStrategy, TakeGenerated};
use hick_store::{
    BuiltinVersionStore, FileProvenance, FsObjectStore, GitVersionStore, VersionStore,
};

use crate::store_config::{StoreConfig, StoreBackend};

/// Run the watch loop: initial run + re-run on file changes.
pub async fn run_watch(
    files: &[PathBuf],
    config: &StoreConfig,
    dry_run: bool,
    images_dir: PathBuf,
) -> Result<()> {
    let project_dir = std::env::current_dir()?;

    // Set up version store
    let store: Arc<dyn VersionStore> = create_store(config, &project_dir).await?;

    // Set up merge strategy
    let strategy: Box<dyn MergeStrategy> = if let Some(ref api_url) = config.merge_api {
        Box::new(hick_merge::LlmMergeStrategy::new(api_url))
    } else {
        Box::new(TakeGenerated)
    };

    // Initial run
    info!("Running initial pipeline...");
    run_pipeline_and_merge(files, config, dry_run, &images_dir, &store, strategy.as_ref())
        .await
        .unwrap_or_else(|e| {
            error!("Initial pipeline run failed: {e}");
        });

    // Set up file watcher
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        notify::Config::default().with_poll_interval(Duration::from_millis(100)),
    )?;

    // Watch each source file
    for file in files {
        let canonical = file
            .canonicalize()
            .with_context(|| format!("cannot watch {}", file.display()))?;
        watcher.watch(&canonical, RecursiveMode::NonRecursive)?;
        info!("Watching: {}", canonical.display());
    }

    info!("Watch mode active. Press Ctrl+C to stop.");

    // Debounce: collect events for 100ms before re-running
    loop {
        match rx.recv() {
            Ok(event) => {
                if !is_modify_event(&event) {
                    continue;
                }

                debug!("Change detected: {:?}", event.paths);

                // Drain any events within the debounce window
                let debounce = Duration::from_millis(100);
                while rx.recv_timeout(debounce).is_ok() {}

                info!("Re-running pipeline...");
                run_pipeline_and_merge(
                    files,
                    config,
                    dry_run,
                    &images_dir,
                    &store,
                    strategy.as_ref(),
                )
                .await
                .unwrap_or_else(|e| {
                    error!("Pipeline re-run failed: {e}");
                });
            }
            Err(_) => {
                // Channel disconnected — watcher was dropped
                break;
            }
        }
    }

    Ok(())
}

/// Create a version store based on configuration.
async fn create_store(config: &StoreConfig, project_dir: &Path) -> Result<Arc<dyn VersionStore>> {
    let backend = config.resolve_store_backend(project_dir);
    match backend {
        StoreBackend::Git => {
            let git_dir = project_dir.join(".hick/git");
            std::fs::create_dir_all(&git_dir)?;
            info!("Using git store at {}", git_dir.display());
            Ok(Arc::new(GitVersionStore::new(&git_dir).await?))
        }
        StoreBackend::Builtin | StoreBackend::Auto => {
            let store_dir = project_dir.join(".hick/store");
            std::fs::create_dir_all(&store_dir)?;
            info!("Using builtin store at {}", store_dir.display());
            let obj_store = Arc::new(FsObjectStore::new(&store_dir));
            Ok(Arc::new(BuiltinVersionStore::new(obj_store)))
        }
    }
}

/// Run the pipeline, merge with previous state, and write outputs.
async fn run_pipeline_and_merge(
    files: &[PathBuf],
    config: &StoreConfig,
    _dry_run: bool,
    _images_dir: &Path,
    store: &Arc<dyn VersionStore>,
    strategy: &dyn MergeStrategy,
) -> Result<()> {
    // Read source files
    let mut file_contents = Vec::new();
    for path in files {
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        file_contents.push((path.display().to_string(), source));
    }

    let sources: Vec<(&str, &str)> = file_contents
        .iter()
        .map(|(name, content)| (name.as_str(), content.as_str()))
        .collect();

    // Run multi-stage pipeline
    let pipeline_result =
        crate::run_pipeline_multi_stage(&sources, config.max_stages).await?;

    // Build pipeline files as bytes for merge
    let pipeline_files: HashMap<String, Vec<u8>> = pipeline_result
        .files
        .iter()
        .map(|(k, v)| (k.clone(), v.as_bytes().to_vec()))
        .collect();

    let pipeline_provenance: HashMap<String, FileProvenance> = pipeline_result.provenance;

    // Read current disk files for three-way merge
    let disk_files = read_disk_files(&pipeline_files)?;

    // Run merge
    let orchestrator = MergeOrchestrator::new(store.as_ref(), strategy);
    let merge_result = orchestrator
        .merge(&config.branch, &pipeline_files, &pipeline_provenance, &disk_files)
        .await?;

    if !merge_result.conflicts_resolved.is_empty() {
        info!(
            "Resolved {} conflict(s): {}",
            merge_result.conflicts_resolved.len(),
            merge_result.conflicts_resolved.join(", ")
        );
    }

    // Write merged files to disk
    for (path, content) in &merge_result.files {
        if let Some(parent) = PathBuf::from(path).parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        debug!("  Wrote {} ({} bytes)", path, content.len());
    }

    // Commit snapshot
    orchestrator
        .commit(
            &config.branch,
            &merge_result.files,
            &pipeline_provenance,
            Some("pipeline run"),
        )
        .await?;

    info!(
        "Pipeline complete: {} file(s) written, snapshot committed to '{}'",
        merge_result.files.len(),
        config.branch
    );

    Ok(())
}

/// Read existing disk files that correspond to pipeline output paths.
fn read_disk_files(pipeline_files: &HashMap<String, Vec<u8>>) -> Result<HashMap<String, Vec<u8>>> {
    let mut disk_files = HashMap::new();
    for path in pipeline_files.keys() {
        let disk_path = PathBuf::from(path);
        if disk_path.exists() {
            let content = std::fs::read(&disk_path)?;
            disk_files.insert(path.clone(), content);
        }
    }
    Ok(disk_files)
}

/// Check if a notify event represents a file modification.
fn is_modify_event(event: &notify::Event) -> bool {
    matches!(
        event.kind,
        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
    )
}
