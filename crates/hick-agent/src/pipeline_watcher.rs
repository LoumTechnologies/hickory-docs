//! Transparent pipeline: watch `.hick` files, auto-rerun pipeline, write provenance.
//!
//! The agent never explicitly calls the pipeline. When `.hick` files change,
//! the watcher re-runs the pipeline and writes:
//! - Output files to the project directory
//! - Provenance JSON to `.hick/provenance/<file>.json`
//! - State metadata to `.hick/pipeline.state`
//!
//! A lock file (`.hick/pipeline.lock`) is held during each run.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use sha2::{Digest, Sha256};
use tokio::sync::{Mutex, Notify};

use hick_agent_core::{HickEditFormat, HickEditService, PipelineOutput};

use crate::file_io::load_hick_sources;

// ---------------------------------------------------------------------------
// Pipeline state file
// ---------------------------------------------------------------------------

/// Metadata written to `.hick/pipeline.state` after each successful run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipelineState {
    /// ISO-8601 timestamp of the last successful run.
    pub last_run: String,
    /// SHA-256 hex digest per `.hick` source file at the time of the run.
    pub file_hashes: HashMap<String, String>,
    /// Output file paths produced by the last run.
    pub output_files: Vec<String>,
}

// ---------------------------------------------------------------------------
// PipelineWatcher
// ---------------------------------------------------------------------------

/// Watches `.hick` files and transparently reruns the pipeline.
pub struct PipelineWatcher {
    project_dir: PathBuf,
    output_dir: Option<String>,
    edit_service: Arc<Mutex<HickEditService>>,
    /// Notified when a pipeline run completes (for staleness checks).
    run_complete: Arc<Notify>,
    /// Last known state (for staleness detection).
    state: Arc<Mutex<Option<PipelineState>>>,
}

impl PipelineWatcher {
    pub fn new(project_dir: PathBuf, output_dir: Option<String>) -> Self {
        Self {
            project_dir,
            output_dir,
            edit_service: Arc::new(Mutex::new(HickEditService::new())),
            run_complete: Arc::new(Notify::new()),
            state: Arc::new(Mutex::new(None)),
        }
    }

    /// Run the pipeline once and write all outputs + provenance.
    pub async fn run_once(&self) -> Result<PipelineOutput> {
        let lock_path = self.hick_dir().join("pipeline.lock");
        std::fs::create_dir_all(self.hick_dir())?;

        // Acquire lock file
        std::fs::write(&lock_path, std::process::id().to_string())
            .context("failed to write pipeline lock")?;

        let result = self.run_pipeline_inner().await;

        // Always release lock
        let _ = std::fs::remove_file(&lock_path);

        result
    }

    /// Start the file watcher loop. Runs until the returned handle is dropped
    /// or the process exits.
    pub async fn watch_loop(self: Arc<Self>) -> Result<()> {
        // Initial run
        info!("Pipeline watcher: initial run");
        match self.run_once().await {
            Ok(output) => {
                info!(
                    "Pipeline watcher: initial run produced {} files",
                    output.files.len()
                );
            }
            Err(e) => {
                warn!("Pipeline watcher: initial run failed: {e}");
            }
        }

        // Set up file watcher
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            notify::Config::default().with_poll_interval(Duration::from_millis(100)),
        )?;

        // Watch the project directory for .hick file changes
        watcher.watch(&self.project_dir, RecursiveMode::Recursive)?;
        info!(
            "Pipeline watcher: watching {} for .hick changes",
            self.project_dir.display()
        );

        loop {
            let event = tokio::select! {
                event = rx.recv() => match event {
                    Some(e) => e,
                    None => break,
                },
                _ = tokio::signal::ctrl_c() => break,
            };

            if !is_hick_change(&event) {
                continue;
            }

            debug!("Pipeline watcher: .hick change detected: {:?}", event.paths);

            // Debounce: drain events for 200ms
            let debounce = Duration::from_millis(200);
            while let Ok(Some(_)) = tokio::time::timeout(debounce, rx.recv()).await {}

            // Check if sources actually changed (avoid spurious reruns)
            if !self.is_stale().await {
                debug!("Pipeline watcher: sources unchanged, skipping rerun");
                continue;
            }

            info!("Pipeline watcher: rerunning pipeline");
            match self.run_once().await {
                Ok(output) => {
                    info!("Pipeline watcher: produced {} files", output.files.len());
                }
                Err(e) => {
                    error!("Pipeline watcher: rerun failed: {e}");
                }
            }
        }

        Ok(())
    }

    /// Check if the pipeline output is stale (any .hick file changed since last run).
    pub async fn is_stale(&self) -> bool {
        let current_hashes = match hash_hick_sources(&self.project_dir) {
            Ok(h) => h,
            Err(_) => return true, // conservative: rerun if we can't hash
        };

        let state = self.state.lock().await;
        match state.as_ref() {
            None => true,
            Some(s) => s.file_hashes != current_hashes,
        }
    }

    /// Block until the pipeline is current (not stale), running it if needed.
    /// Call this before returning output file content to the agent.
    pub async fn ensure_current(&self) -> Result<()> {
        if self.is_stale().await {
            self.run_once().await?;
        }
        Ok(())
    }

    /// Get a handle for staleness notification.
    pub fn run_complete_notify(&self) -> Arc<Notify> {
        Arc::clone(&self.run_complete)
    }

    // -----------------------------------------------------------------------
    // Internal
    // -----------------------------------------------------------------------

    fn hick_dir(&self) -> PathBuf {
        self.project_dir.join(".hick")
    }

    fn provenance_dir(&self) -> PathBuf {
        self.hick_dir().join("provenance")
    }

    async fn run_pipeline_inner(&self) -> Result<PipelineOutput> {
        // Load sources
        let sources = load_hick_sources(&self.project_dir)?;
        let source_pairs: Vec<(String, String)> = sources
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        // Compute hashes for state tracking
        let file_hashes = hash_sources(&sources);

        // Run pipeline
        let mut svc = self.edit_service.lock().await;
        let output = svc.run_pipeline(&source_pairs, &[]).await?;
        drop(svc);

        // Write output files
        let base = match &self.output_dir {
            Some(d) => self.project_dir.join(d),
            None => self.project_dir.clone(),
        };
        for (rel_path, content) in &output.files {
            let full = base.join(rel_path);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&full, content)?;
        }

        // Write provenance JSON
        self.write_provenance(&output)?;

        // Write state
        let new_state = PipelineState {
            last_run: chrono_now(),
            file_hashes,
            output_files: output.files.keys().cloned().collect(),
        };
        self.write_state(&new_state)?;
        *self.state.lock().await = Some(new_state);

        // Notify waiters
        self.run_complete.notify_waiters();

        Ok(output)
    }

    fn write_provenance(&self, output: &PipelineOutput) -> Result<()> {
        let prov_dir = self.provenance_dir();
        std::fs::create_dir_all(&prov_dir)?;

        for (file_path, prov_map) in &output.provenance_maps {
            let safe_name = file_path.replace('/', "__");
            let json_path = prov_dir.join(format!("{safe_name}.json"));
            let json =
                serde_json::to_string_pretty(prov_map).context("failed to serialize provenance")?;
            std::fs::write(&json_path, json)?;
            debug!("Wrote provenance: {}", json_path.display());
        }

        // Also write a manifest listing all provenance files
        let manifest_entries: Vec<serde_json::Value> = output
            .provenance_maps
            .keys()
            .map(|k| {
                let safe = k.replace('/', "__");
                serde_json::json!({
                    "output_file": k,
                    "provenance_file": format!("{safe}.json"),
                })
            })
            .collect();
        let manifest_json = serde_json::to_string_pretty(&manifest_entries)?;
        std::fs::write(prov_dir.join("manifest.json"), manifest_json)?;

        Ok(())
    }

    fn write_state(&self, state: &PipelineState) -> Result<()> {
        let state_path = self.hick_dir().join("pipeline.state");
        let json =
            serde_json::to_string_pretty(state).context("failed to serialize pipeline state")?;
        std::fs::write(&state_path, json)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Check if a notify event represents a `.hick` file modification.
fn is_hick_change(event: &notify::Event) -> bool {
    let is_change = matches!(
        event.kind,
        EventKind::Any | EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
    );
    if !is_change {
        return false;
    }
    event
        .paths
        .iter()
        .any(|p| p.extension().is_some_and(|e| e == "hick"))
}

/// Compute SHA-256 hashes for in-memory source content.
fn hash_sources(sources: &HashMap<String, String>) -> HashMap<String, String> {
    sources
        .iter()
        .map(|(path, content)| {
            let hash = hex_sha256(content.as_bytes());
            (path.clone(), hash)
        })
        .collect()
}

/// Compute SHA-256 hashes for `.hick` files on disk.
fn hash_hick_sources(project_dir: &Path) -> Result<HashMap<String, String>> {
    let sources = load_hick_sources(project_dir)?;
    Ok(hash_sources(&sources))
}

fn hex_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// ISO-8601 timestamp (no chrono dependency — just use system time).
fn chrono_now() -> String {
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    // Simple unix timestamp as ISO-ish string
    let secs = duration.as_secs();
    format!("{secs}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn hash_sources_deterministic() {
        let mut sources = HashMap::new();
        sources.insert("a.hick".to_string(), "hello".to_string());
        let h1 = hash_sources(&sources);
        let h2 = hash_sources(&sources);
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_sources_changes_on_content_change() {
        let mut sources = HashMap::new();
        sources.insert("a.hick".to_string(), "hello".to_string());
        let h1 = hash_sources(&sources);
        sources.insert("a.hick".to_string(), "world".to_string());
        let h2 = hash_sources(&sources);
        assert_ne!(h1["a.hick"], h2["a.hick"]);
    }

    #[test]
    fn is_hick_change_filters_non_hick() {
        let event = notify::Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![PathBuf::from("foo.rs")],
            attrs: Default::default(),
        };
        assert!(!is_hick_change(&event));

        let event = notify::Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![PathBuf::from("main.hick")],
            attrs: Default::default(),
        };
        assert!(is_hick_change(&event));
    }

    #[test]
    fn pipeline_state_serialization() {
        let state = PipelineState {
            last_run: "12345".to_string(),
            file_hashes: HashMap::from([("a.hick".to_string(), "abc123".to_string())]),
            output_files: vec!["src/main.rs".to_string()],
        };
        let json = serde_json::to_string(&state).unwrap();
        let state2: PipelineState = serde_json::from_str(&json).unwrap();
        assert_eq!(state2.last_run, "12345");
        assert_eq!(state2.file_hashes["a.hick"], "abc123");
        assert_eq!(state2.output_files, vec!["src/main.rs"]);
    }

    #[tokio::test]
    async fn watcher_starts_with_no_state() {
        let dir = TempDir::new().unwrap();
        let watcher = PipelineWatcher::new(dir.path().to_path_buf(), None);
        assert!(watcher.is_stale().await);
    }

    #[test]
    fn provenance_dir_path() {
        let watcher = PipelineWatcher::new(PathBuf::from("/workspace"), None);
        assert_eq!(
            watcher.provenance_dir(),
            PathBuf::from("/workspace/.hick/provenance")
        );
    }
}
