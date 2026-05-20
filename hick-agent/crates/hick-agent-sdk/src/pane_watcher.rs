//! Agent pane watcher — monitors `.agents/panes/*/pane.yaml` and emits [`UiEvent::PaneContent`].
//!
//! Each pane is a subdirectory of `.agents/panes/` containing a `pane.yaml` file. On create,
//! modify, or delete the watcher reads the entry file and emits a [`UiEvent::PaneContent`];
//! deleting the directory or `pane.yaml` emits a clear event (empty markdown).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::Context as _;
use serde::Deserialize;
use tokio::sync::broadcast;

use crate::ui_event::{IdGen, UiEvent};

// ---------------------------------------------------------------------------
// pane.yaml schema
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct PaneYaml {
    /// "markdown" or "html"
    pub r#type: String,
    pub title: String,
    /// Entry filename inside the pane directory.
    pub entry: String,
    /// Extra `connect-src` origins for html panes (additive with the runtime allowlist).
    #[serde(default)]
    pub extra_connect_src: Vec<String>,
}

impl PaneYaml {
    fn load(pane_dir: &Path) -> anyhow::Result<Self> {
        let yaml_path = pane_dir.join("pane.yaml");
        let text = std::fs::read_to_string(&yaml_path)
            .with_context(|| format!("read {}", yaml_path.display()))?;
        let parsed: PaneYaml = serde_yaml::from_str(&text)
            .with_context(|| format!("parse {}", yaml_path.display()))?;
        if parsed.r#type != "markdown" && parsed.r#type != "html" {
            anyhow::bail!("pane.yaml: type must be 'markdown' or 'html', got '{}'", parsed.r#type);
        }
        let entry_path = pane_dir.join(&parsed.entry);
        if !entry_path.exists() {
            anyhow::bail!(
                "pane.yaml: entry '{}' not found in {}",
                parsed.entry,
                pane_dir.display()
            );
        }
        Ok(parsed)
    }
}

// ---------------------------------------------------------------------------
// PaneWatcher
// ---------------------------------------------------------------------------

/// Watches `.agents/panes/*/pane.yaml` and emits [`UiEvent::PaneContent`] events.
pub struct PaneWatcher {
    panes_dir: PathBuf,
    event_tx: broadcast::Sender<UiEvent>,
    ids: IdGen,
}

impl PaneWatcher {
    pub fn new(project_dir: &Path, event_tx: broadcast::Sender<UiEvent>) -> Self {
        Self {
            panes_dir: project_dir.join(".agents").join("panes"),
            event_tx,
            ids: IdGen::new(),
        }
    }

    /// Initial scan emitting events for all existing panes (call at agent startup).
    pub fn initial_scan(&self) {
        if !self.panes_dir.exists() {
            return;
        }
        if let Ok(entries) = std::fs::read_dir(&self.panes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        let _ = self.emit_pane(name, &path);
                    }
                }
            }
        }
    }

    /// Polling loop — runs indefinitely, re-scanning every `interval`.
    pub async fn run(self, interval: Duration) {
        let mut last_mtimes: HashMap<String, (SystemTime, Option<SystemTime>)> = HashMap::new();

        self.initial_scan();

        loop {
            tokio::time::sleep(interval).await;

            let panes_dir = &self.panes_dir;
            if !panes_dir.exists() {
                // Nothing to watch yet.
                continue;
            }

            // Collect current pane directories.
            let mut current_names: std::collections::HashSet<String> = Default::default();

            if let Ok(entries) = std::fs::read_dir(panes_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                        continue;
                    };
                    let name = name.to_string();
                    current_names.insert(name.clone());

                    let yaml_path = path.join("pane.yaml");
                    let yaml_mtime = mtime_of(&yaml_path);

                    // Read entry mtime if yaml parses (best-effort).
                    let entry_mtime = PaneYaml::load(&path).ok().and_then(|py| {
                        let ep = path.join(&py.entry);
                        mtime_of(&ep)
                    });

                    let prev = last_mtimes.get(&name).copied();
                    let changed = match (prev, yaml_mtime) {
                        (None, _) => true,
                        (Some((prev_yaml, prev_entry)), Some(cur_yaml)) => {
                            cur_yaml != prev_yaml || entry_mtime != prev_entry
                        }
                        (Some(_), None) => false,
                    };

                    if changed {
                        last_mtimes.insert(name.clone(), (yaml_mtime.unwrap_or(SystemTime::UNIX_EPOCH), entry_mtime));
                        let _ = self.emit_pane(&name, &path);
                    }
                }
            }

            // Clear panes that no longer have a directory.
            let gone: Vec<String> = last_mtimes
                .keys()
                .filter(|k| !current_names.contains(*k))
                .cloned()
                .collect();
            for name in gone {
                last_mtimes.remove(&name);
                let _ = self.emit_clear(&name);
            }
        }
    }

    fn emit_pane(&self, name: &str, pane_dir: &Path) -> anyhow::Result<()> {
        let py = PaneYaml::load(pane_dir)?;

        let markdown = if py.r#type == "markdown" {
            let entry_path = pane_dir.join(&py.entry);
            std::fs::read_to_string(&entry_path)
                .with_context(|| format!("read {}", entry_path.display()))?
        } else {
            // For html panes the web UI iframes the file directly; markdown is empty.
            String::new()
        };

        let _ = self.event_tx.send(UiEvent::PaneContent {
            node_id: self.ids.next(),
            name: name.to_string(),
            title: py.title,
            markdown,
            pane_type: py.r#type,
            entry: py.entry,
        });
        Ok(())
    }

    fn emit_clear(&self, name: &str) {
        let _ = self.event_tx.send(UiEvent::PaneContent {
            node_id: self.ids.next(),
            name: name.to_string(),
            title: String::new(),
            markdown: String::new(),
            pane_type: "markdown".to_string(),
            entry: String::new(),
        });
    }
}

fn mtime_of(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}
