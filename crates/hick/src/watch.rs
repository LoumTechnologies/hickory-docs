//! Watch loop for `hick up` command.
//!
//! Monitors `.hick` source files for changes and re-runs the pipeline,
//! merging results with the previous snapshot via the merge orchestrator.

use std::collections::{HashMap, HashSet};
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

use crate::pool::{ContainerPool, PoolConfig, SharedPool};
use crate::store_config::{StoreBackend, StoreConfig};

/// Run the watch loop: initial run + re-run on file changes.
pub async fn run_watch(
    files: &[PathBuf],
    config: &StoreConfig,
    params: &[(String, String)],
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

    // Create container pool for live execution (survives across iterations)
    let pool: Option<SharedPool> = if !dry_run {
        let pool_config = PoolConfig::new(images_dir.clone());
        info!("Pool: created (max_warm: {})", pool_config.max_warm);
        Some(Arc::new(tokio::sync::Mutex::new(ContainerPool::new(
            pool_config,
        ))))
    } else {
        None
    };

    // Initial run
    info!("Running initial pipeline...");
    run_pipeline_and_merge(
        files,
        config,
        params,
        dry_run,
        &images_dir,
        &store,
        strategy.as_ref(),
        pool.as_ref(),
    )
    .await
    .unwrap_or_else(|e| {
        error!("Initial pipeline run failed: {e}");
    });
    log_pool_stats(pool.as_ref()).await;

    // Collect canonical paths and their parent directories for watching.
    // On macOS, notify's FsEventWatcher uses FSEvents which is a directory-level
    // API. Watching individual files can silently miss events. Watching parent
    // directories and filtering by source path is more reliable.
    let mut watched_files: HashSet<PathBuf> = HashSet::new();
    let mut watched_dirs: HashSet<PathBuf> = HashSet::new();
    for file in files {
        let canonical = file
            .canonicalize()
            .with_context(|| format!("cannot watch {}", file.display()))?;
        eprintln!("Watching: {}", canonical.display());
        if let Some(parent) = canonical.parent() {
            watched_dirs.insert(parent.to_path_buf());
        }
        watched_files.insert(canonical);
    }

    // Set up file watcher.  Use a tokio channel so the event loop can
    // `select!` between file events and Ctrl+C for clean shutdown.
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        notify::Config::default().with_poll_interval(Duration::from_millis(100)),
    )?;

    for dir in &watched_dirs {
        watcher.watch(dir, RecursiveMode::NonRecursive)?;
        debug!("Watching directory: {}", dir.display());
    }

    eprintln!("Watch mode active. Press Ctrl+C to stop.");

    // Debounce: collect events for 100ms before re-running.
    // Filter events to only source files so output writes don't cause loops.
    loop {
        // Wait for a file event or Ctrl+C, whichever comes first.
        let event = tokio::select! {
            event = rx.recv() => match event {
                Some(e) => e,
                None => break, // channel closed
            },
            _ = tokio::signal::ctrl_c() => break,
        };

        if !is_modify_event(&event) {
            continue;
        }

        let is_source_change = event.paths.iter().any(|p| {
            watched_files.contains(p) || p.canonicalize().is_ok_and(|c| watched_files.contains(&c))
        });
        if !is_source_change {
            continue;
        }

        debug!("Change detected: {:?}", event.paths);

        // Drain any events within the debounce window
        let debounce = Duration::from_millis(100);
        while let Ok(Some(_)) = tokio::time::timeout(debounce, rx.recv()).await {}

        eprintln!("Re-running pipeline...");
        run_pipeline_and_merge(
            files,
            config,
            params,
            dry_run,
            &images_dir,
            &store,
            strategy.as_ref(),
            pool.as_ref(),
        )
        .await
        .unwrap_or_else(|e| {
            error!("Pipeline re-run failed: {e}");
        });
        log_pool_stats(pool.as_ref()).await;
    }

    // Shutdown pool on exit
    eprintln!("\nShutting down...");
    if let Some(pool) = pool {
        let mut pool_guard = pool.lock().await;
        pool_guard.shutdown().await;
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
#[allow(clippy::too_many_arguments)]
async fn run_pipeline_and_merge(
    files: &[PathBuf],
    config: &StoreConfig,
    params: &[(String, String)],
    dry_run: bool,
    images_dir: &Path,
    store: &Arc<dyn VersionStore>,
    strategy: &dyn MergeStrategy,
    pool: Option<&SharedPool>,
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

    // Pre-warm pool from sources (if pool is active)
    if let Some(pool) = pool {
        let mut pool_guard = pool.lock().await;
        pool_guard.warm_from_sources(&sources).await?;
        drop(pool_guard);
    }

    // Run pipeline (live with pool, or dry-run multi-stage)
    let (pipeline_files, pipeline_provenance, pool_hits, pool_boot_time_saved) = if dry_run {
        let pipeline_result =
            crate::run_pipeline_multi_stage(&sources, params, config.max_stages).await?;
        let files: HashMap<String, Vec<u8>> = pipeline_result
            .files
            .iter()
            .map(|(k, v)| {
                let bytes = match v {
                    hick_exec::node::FileContent::Text(s) => s.as_bytes().to_vec(),
                    hick_exec::node::FileContent::Binary(d) => d.to_bytes().unwrap_or_default(),
                };
                (k.clone(), bytes)
            })
            .collect();
        (
            files,
            pipeline_result.provenance,
            0usize,
            std::time::Duration::ZERO,
        )
    } else {
        let pipeline_config = crate::PipelineConfig {
            images_dir: images_dir.to_path_buf(),
            toolchain_dir: None,
            working_dir: None,
            max_rounds: 1,
        };
        let result =
            crate::run_pipeline_live(&sources, &pipeline_config, params, None, pool).await?;
        let files: HashMap<String, Vec<u8>> = result
            .files
            .iter()
            .map(|(k, v)| {
                let bytes = match v {
                    hick_exec::node::FileContent::Text(s) => s.as_bytes().to_vec(),
                    hick_exec::node::FileContent::Binary(d) => d.to_bytes().unwrap_or_default(),
                };
                (k.clone(), bytes)
            })
            .collect();
        let provenance: HashMap<String, FileProvenance> = result
            .files
            .keys()
            .map(|k| {
                (
                    k.clone(),
                    FileProvenance::HickFile {
                        source: sources
                            .first()
                            .map(|(n, _)| n.to_string())
                            .unwrap_or_default(),
                    },
                )
            })
            .collect();
        if result.pool_hits > 0 {
            info!(
                "Pool: {} of {} containers adopted from pool",
                result.pool_hits,
                result.resource_stats.len()
            );
        }
        let hits = result.pool_hits;
        let saved = result.pool_boot_time_saved;
        (files, provenance, hits, saved)
    };

    // Evict stale containers after each run
    if let Some(pool) = pool {
        let mut pool_guard = pool.lock().await;
        pool_guard.evict_stale();
    }

    // Read current disk files for three-way merge
    let disk_files = read_disk_files(&pipeline_files)?;

    // Run merge
    let orchestrator = MergeOrchestrator::new(store.as_ref(), strategy);
    let merge_result = orchestrator
        .merge(
            &config.branch,
            &pipeline_files,
            &pipeline_provenance,
            &disk_files,
        )
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

    eprintln!(
        "Pipeline complete: {} file(s) written, snapshot committed to '{}'",
        merge_result.files.len(),
        config.branch
    );

    // Print pool savings summary when pool was used
    if pool_hits > 0
        && let Some(pool) = pool
    {
        let pool_guard = pool.lock().await;
        let stats = pool_guard.stats();
        let peak_mb = stats.peak_memory_bytes / (1024 * 1024);
        eprintln!(
            "  Pool: saved {} boot, cost {}MB for {}",
            crate::fmt_duration(pool_boot_time_saved),
            peak_mb,
            crate::fmt_duration(stats.uptime),
        );
    }

    Ok(())
}

/// Log pool statistics after a pipeline run.
async fn log_pool_stats(pool: Option<&SharedPool>) {
    if let Some(pool) = pool {
        let pool_guard = pool.lock().await;
        let stats = pool_guard.stats();
        let mem_mb = stats.estimated_memory_bytes / (1024 * 1024);
        let peak_mb = stats.peak_memory_bytes / (1024 * 1024);
        info!(
            "Pool: {} warm ({}MB, peak {}MB), {} served, {} evicted, uptime {}",
            stats.warm_count,
            mem_mb,
            peak_mb,
            stats.containers_served,
            stats.containers_evicted,
            crate::fmt_duration(stats.uptime),
        );
    }
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
///
/// Includes `EventKind::Any` because macOS FSEvents can emit it for
/// modifications that don't map to a specific sub-kind.
fn is_modify_event(event: &notify::Event) -> bool {
    matches!(
        event.kind,
        EventKind::Any | EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
    )
}
