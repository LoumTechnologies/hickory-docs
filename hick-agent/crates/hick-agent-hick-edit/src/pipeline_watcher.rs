//! Transparent pipeline: watch `.hick` files and output files.
//!
//! The agent never explicitly calls the pipeline. When `.hick` files change,
//! the watcher re-runs the pipeline and writes:
//! - Output files to the project directory
//! - Provenance JSON to `.hick/provenance/<file>.json`
//! - State metadata to `.hick/pipeline.state`
//!
//! When *output* files are edited (by the agent or a user in their IDE),
//! reverse provenance maps the changes back to `.hick` source, applies them,
//! and reruns the pipeline to verify the round-trip.
//!
//! A lock file (`.hick/pipeline.lock`) is held during each run.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use hick_flow::provenance::ProvenanceMap;
use log::{debug, error, info, warn};
use notify::{EventKind, PollWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use sha2::{Digest, Sha256};
use tokio::sync::{Mutex, Notify};

use crate::{HickEditFormat, HickEditService, PipelineOutput};
use crate::file_io::{load_hick_sources, write_hick_sources};
use crate::reverse_provenance::{
    ReverseResult, VerifyResult, apply_reverse_edits, diff_output, reverse_map, verify_round_trip,
};

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

/// Watches `.hick` files and output files, transparently reruns the pipeline,
/// and applies reverse provenance when output files are edited.
pub struct PipelineWatcher {
    project_dir: PathBuf,
    output_dir: Option<String>,
    edit_service: Arc<Mutex<HickEditService>>,
    /// Notified when a pipeline run completes (for staleness checks).
    run_complete: Arc<Notify>,
    /// Last known state (for staleness detection).
    state: Arc<Mutex<Option<PipelineState>>>,
    /// Cached output file content from the last pipeline run.
    /// Used for diff detection when output files are edited.
    last_output: Arc<Mutex<HashMap<String, String>>>,
    /// Cached provenance maps from the last pipeline run.
    last_provenance: Arc<Mutex<HashMap<String, ProvenanceMap>>>,
}

impl PipelineWatcher {
    pub fn new(project_dir: PathBuf, output_dir: Option<String>) -> Self {
        Self {
            project_dir,
            output_dir,
            edit_service: Arc::new(Mutex::new(HickEditService::new())),
            run_complete: Arc::new(Notify::new()),
            state: Arc::new(Mutex::new(None)),
            last_output: Arc::new(Mutex::new(HashMap::new())),
            last_provenance: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Run the pipeline once and write all outputs + provenance.
    pub async fn run_once(&self) -> Result<PipelineOutput> {
        let lock_path = self.hick_dir().join("pipeline.lock");
        std::fs::create_dir_all(self.hick_dir())?;

        std::fs::write(&lock_path, std::process::id().to_string())
            .context("failed to write pipeline lock")?;

        let result = self.run_pipeline_inner().await;

        let _ = std::fs::remove_file(&lock_path);

        result
    }

    /// Start the file watcher loop. Runs until the returned handle is dropped
    /// or the process exits.
    pub async fn watch_loop(self: Arc<Self>) -> Result<()> {
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

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let poll_config = notify::Config::default().with_poll_interval(Duration::from_millis(500));
        let make_cb = {
            let tx = tx.clone();
            move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            }
        };
        let mut watcher: Box<dyn Watcher + Send> =
            match RecommendedWatcher::new(make_cb, notify::Config::default()) {
                Ok(w) => {
                    info!("Pipeline watcher: using inotify");
                    Box::new(w)
                }
                Err(e) => {
                    warn!("Pipeline watcher: inotify unavailable ({e}), falling back to polling");
                    let cb = move |res: notify::Result<notify::Event>| {
                        if let Ok(event) = res {
                            let _ = tx.send(event);
                        }
                    };
                    Box::new(PollWatcher::new(cb, poll_config)?)
                }
            };

        watcher.watch(&self.project_dir, RecursiveMode::Recursive)?;
        info!(
            "Pipeline watcher: watching {} for .hick and output changes",
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

            let is_change = matches!(
                event.kind,
                EventKind::Any | EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
            );
            if !is_change {
                continue;
            }

            let hick_change = is_hick_change(&event);
            let output_change = self.is_output_change(&event).await;

            if !hick_change && !output_change {
                continue;
            }

            let debounce = Duration::from_millis(200);
            while let Ok(Some(_)) = tokio::time::timeout(debounce, rx.recv()).await {}

            if hick_change {
                if !self.is_stale().await {
                    debug!("Pipeline watcher: sources unchanged, skipping rerun");
                    continue;
                }
                info!("Pipeline watcher: rerunning pipeline (.hick change)");
                match self.run_once().await {
                    Ok(output) => {
                        info!("Pipeline watcher: produced {} files", output.files.len());
                    }
                    Err(e) => {
                        error!("Pipeline watcher: rerun failed: {e}");
                    }
                }
            } else if output_change {
                let changed_files = self.detect_output_changes(&event).await;
                for (rel_path, new_content) in changed_files {
                    info!("Pipeline watcher: output edit detected in {rel_path}");
                    match self.handle_output_edit(&rel_path, &new_content).await {
                        Ok(result) => {
                            if result.is_fully_reversible() {
                                info!(
                                    "Pipeline watcher: reverse-applied {} edits from {rel_path}",
                                    result.edits.len()
                                );
                            } else if !result.errors.is_empty() {
                                for err in &result.errors {
                                    warn!("Reverse provenance: {}", err.message);
                                }
                            }
                        }
                        Err(e) => {
                            error!(
                                "Pipeline watcher: reverse provenance failed for {rel_path}: {e}"
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if the pipeline output is stale (any .hick file changed since last run).
    pub async fn is_stale(&self) -> bool {
        let current_hashes = match hash_hick_sources(&self.project_dir) {
            Ok(h) => h,
            Err(_) => return true,
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
    // Reverse provenance: output file edits → .hick source edits
    // -----------------------------------------------------------------------

    /// Handle an edit to an output file by reverse-mapping it to .hick source.
    ///
    /// 1. Diff old (cached) vs new content
    /// 2. Look up provenance spans
    /// 3. Compute reverse edits (or errors for non-reversible regions)
    /// 4. Apply reversible edits to .hick source files
    /// 5. Rerun pipeline and verify round-trip
    pub async fn handle_output_edit(
        &self,
        output_file: &str,
        new_content: &str,
    ) -> Result<ReverseResult> {
        let old_content = {
            let cache = self.last_output.lock().await;
            cache.get(output_file).cloned()
        };
        let old_content = old_content
            .ok_or_else(|| anyhow::anyhow!("No cached content for {output_file} — cannot diff"))?;

        let provenance = {
            let prov_cache = self.last_provenance.lock().await;
            prov_cache.get(output_file).cloned()
        };
        let provenance = provenance.ok_or_else(|| {
            anyhow::anyhow!("No provenance data for {output_file} — cannot reverse-map")
        })?;

        let changes = diff_output(&old_content, new_content);
        if changes.is_empty() {
            return Ok(ReverseResult {
                edits: Vec::new(),
                errors: Vec::new(),
            });
        }

        let result = reverse_map(output_file, &changes, &provenance);

        if result.edits.is_empty() {
            return Ok(result);
        }

        let sources = load_hick_sources(&self.project_dir)?;
        let updated_sources = apply_reverse_edits(&sources, &result.edits);
        write_hick_sources(&self.project_dir, &updated_sources)?;
        info!(
            "Pipeline watcher: applied {} reverse edits to .hick source",
            result.edits.len()
        );

        match self.run_once().await {
            Ok(output) => {
                let verify = verify_round_trip(output_file, new_content, &output.files);
                match verify {
                    VerifyResult::Match => {
                        info!("Pipeline watcher: round-trip verification passed for {output_file}");
                    }
                    VerifyResult::Mismatch {
                        expected_len,
                        actual_len,
                        first_diff_byte,
                    } => {
                        warn!(
                            "Pipeline watcher: round-trip mismatch for {output_file} \
                             (expected {expected_len} bytes, got {actual_len}, \
                             first diff at byte {:?})",
                            first_diff_byte
                        );
                    }
                    VerifyResult::PipelineError(msg) => {
                        error!("Pipeline watcher: round-trip error for {output_file}: {msg}");
                    }
                }
            }
            Err(e) => {
                error!("Pipeline watcher: rerun after reverse edit failed: {e}");
            }
        }

        Ok(result)
    }

    /// Check if an event affects a known output file.
    async fn is_output_change(&self, event: &notify::Event) -> bool {
        let cache = self.last_output.lock().await;
        if cache.is_empty() {
            return false;
        }
        let base = self.output_base();
        event.paths.iter().any(|p| {
            p.strip_prefix(&base)
                .ok()
                .and_then(|rel| rel.to_str())
                .map(|rel| {
                    let normalized = rel.replace('\\', "/");
                    cache.contains_key(&normalized)
                })
                .unwrap_or(false)
        })
    }

    /// Detect which output files changed and read their new content.
    async fn detect_output_changes(&self, event: &notify::Event) -> Vec<(String, String)> {
        let cache = self.last_output.lock().await;
        let base = self.output_base();
        let mut changed = Vec::new();

        for path in &event.paths {
            if let Some(rel) = path.strip_prefix(&base).ok().and_then(|r| r.to_str()) {
                let normalized = rel.replace('\\', "/");
                if cache.contains_key(&normalized)
                    && let Ok(new_content) = std::fs::read_to_string(path)
                    && cache.get(&normalized).is_none_or(|old| old != &new_content)
                {
                    changed.push((normalized, new_content));
                }
            }
        }

        changed
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

    fn output_base(&self) -> PathBuf {
        match &self.output_dir {
            Some(d) => self.project_dir.join(d),
            None => self.project_dir.clone(),
        }
    }

    async fn run_pipeline_inner(&self) -> Result<PipelineOutput> {
        let sources = load_hick_sources(&self.project_dir)?;
        let source_pairs: Vec<(String, String)> = sources
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let file_hashes = hash_sources(&sources);

        let mut svc = self.edit_service.lock().await;
        let output = svc.run_pipeline(&source_pairs, &[]).await?;
        drop(svc);

        let base = self.output_base();
        for (rel_path, content) in &output.files {
            let full = base.join(rel_path);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&full, content)?;
        }

        *self.last_output.lock().await = output.files.clone();
        *self.last_provenance.lock().await = output.provenance_maps.clone();

        self.write_provenance(&output)?;

        let new_state = PipelineState {
            last_run: chrono_now(),
            file_hashes,
            output_files: output.files.keys().cloned().collect(),
        };
        self.write_state(&new_state)?;
        *self.state.lock().await = Some(new_state);

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

fn hash_sources(sources: &HashMap<String, String>) -> HashMap<String, String> {
    sources
        .iter()
        .map(|(path, content)| {
            let hash = hex_sha256(content.as_bytes());
            (path.clone(), hash)
        })
        .collect()
}

fn hash_hick_sources(project_dir: &Path) -> Result<HashMap<String, String>> {
    let sources = load_hick_sources(project_dir)?;
    Ok(hash_sources(&sources))
}

fn hex_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn chrono_now() -> String {
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
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

    #[tokio::test]
    async fn is_output_change_empty_cache() {
        let dir = TempDir::new().unwrap();
        let watcher = PipelineWatcher::new(dir.path().to_path_buf(), None);
        let event = notify::Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![dir.path().join("src/main.rs")],
            attrs: Default::default(),
        };
        assert!(!watcher.is_output_change(&event).await);
    }

    #[tokio::test]
    async fn is_output_change_with_cached_file() {
        let dir = TempDir::new().unwrap();
        let watcher = PipelineWatcher::new(dir.path().to_path_buf(), None);

        {
            let mut cache = watcher.last_output.lock().await;
            cache.insert("src/main.rs".to_string(), "old content".to_string());
        }

        let event = notify::Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![dir.path().join("src/main.rs")],
            attrs: Default::default(),
        };
        assert!(watcher.is_output_change(&event).await);

        let event2 = notify::Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![dir.path().join("untracked.txt")],
            attrs: Default::default(),
        };
        assert!(!watcher.is_output_change(&event2).await);
    }
}
