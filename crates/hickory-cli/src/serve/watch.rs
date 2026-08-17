//! The up-loop, running inside the app's process.
//!
//! `local-only.md` names the design: the desktop app runs the up-loop and the
//! rooms together, so the window and the working tree can never disagree for
//! long. This module is that loop — the same weave/reverse machinery
//! `hick up` runs headless (`crate::up`), driven from the serve state, with
//! one addition the headless loop has no use for: after every batch, changed
//! documents are reconciled into their live rooms, so an edit made by vim, a
//! formatter, or a coding agent appears in the editor buffer instead of being
//! clobbered by the room's next debounced persist.
//!
//! What is deliberately NOT here:
//!
//! - **No directory lock.** The desktop shell already holds
//!   [`crate::up::DirectoryLock`] for this folder; taking it twice would
//!   refuse ourselves.
//! - **No `--run`.** The in-app loop weaves on change; executing cells is the
//!   Run button's deliberate act, same as the headless default.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use anyhow::{Context as _, Result};
use notify::{EventKind, RecursiveMode, Watcher as _};

use super::LocalState;
use crate::up::state::WovenState;
use crate::up::{UpConfig, handle_batch, weave_document};

/// How long the directory must stay quiet before a burst of events is one
/// batch. Matches the headless loop's weave debounce.
const DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(120);

/// A running in-app up-loop. Ask it to stop (or drop it at exit) and the
/// loop clears its read-only marks before finishing — those are a live
/// signal from a running loop, not a property of the files.
pub struct WatchGuard {
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    task: Option<tokio::task::JoinHandle<()>>,
}

impl WatchGuard {
    /// Stop the loop and wait for its cleanup to finish.
    pub async fn shutdown(mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(task) = self.task.take() {
            let _ = task.await;
        }
    }
}

impl Drop for WatchGuard {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
    }
}

/// Start watching the served folder and keep it woven until told to stop.
///
/// The watcher starts BEFORE the first weave, for the same reason the
/// headless loop's does: the first pass writes output files early, and an
/// edit made to one of them during startup must arrive late, not vanish.
pub fn spawn(state: LocalState) -> Result<WatchGuard> {
    let root = state
        .index
        .root()
        .canonicalize()
        .with_context(|| format!("cannot watch {}", state.index.root().display()))?;

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<PathBuf>();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        let Ok(event) = res else { return };
        if !matches!(
            event.kind,
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
        ) {
            return;
        }
        for path in event.paths {
            let _ = tx.send(path);
        }
    })
    .context("failed to start watching for file changes")?;
    watcher
        .watch(&root, RecursiveMode::Recursive)
        .with_context(|| format!("failed to watch {}", root.display()))?;

    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
    let task = tokio::spawn(run_loop(state, root, watcher, rx, stop_rx));
    Ok(WatchGuard {
        stop: Some(stop_tx),
        task: Some(task),
    })
}

async fn run_loop(
    state: LocalState,
    root: PathBuf,
    // Owned here so the OS watch lives exactly as long as the loop.
    _watcher: notify::RecommendedWatcher,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<PathBuf>,
    mut stop: tokio::sync::oneshot::Receiver<()>,
) {
    let config = UpConfig {
        root: root.clone(),
        params: (*state.params).clone(),
        run: false,
        executor: state.executor,
    };
    let mut woven = WovenState::default();
    // Per-document fingerprint of the outputs' bytes, so "files changed" is
    // published only when a batch actually rewrote something — not for every
    // echo the watcher hears.
    let mut output_marks: HashMap<PathBuf, u64> = HashMap::new();

    // Initial pass: weave what is already indexed. An empty folder weaves
    // nothing and simply waits — that is the app's first-run state.
    for (_, rel) in state.index.entries() {
        let doc = root.join(rel);
        if let Err(e) = weave_document(&doc, &config, &mut woven).await {
            log::warn!("initial weave of {} failed: {e:#}", doc.display());
        }
    }
    reconcile_rooms(&state, &root, &woven).await;

    loop {
        let first = tokio::select! {
            received = rx.recv() => match received {
                Some(path) => path,
                None => break,
            },
            _ = &mut stop => break,
        };
        let mut batch = HashSet::new();
        batch.insert(first);
        loop {
            match tokio::time::timeout(DEBOUNCE, rx.recv()).await {
                Ok(Some(path)) => {
                    batch.insert(path);
                }
                Ok(None) => break,
                Err(_) => break,
            }
        }

        // A `.hick` file the index has never seen is a document created by
        // something other than the app — `touch notes.hick`, a git checkout,
        // an agent. Register it so the app can list and open it without a
        // restart; the weave below is `handle_batch`'s job.
        for path in &batch {
            if path.extension().is_some_and(|e| e == "hick")
                && path.is_file()
                && let Ok(rel) = path
                    .canonicalize()
                    .unwrap_or_else(|_| path.clone())
                    .strip_prefix(&root)
                    .map(|r| r.to_string_lossy().replace('\\', "/"))
            {
                state.index.add(&rel);
            }
        }

        if let Err(e) = handle_batch(batch, &config, &mut woven).await {
            log::warn!("up-loop batch failed: {e:#}");
        }
        reconcile_rooms(&state, &root, &woven).await;
        notify_files_changed(&state, &root, &woven, &mut output_marks).await;
    }

    // However the loop ends, the marks come off: `hick run`, git, and every
    // other tool must find ordinary files afterwards.
    woven.release_read_only();
}

/// Tell every open window that this batch may have rewritten output files.
///
/// The loop weaves server-side; without this, a pane showing a generated
/// file learns about the re-weave only on its next focus or run. The event
/// rides the run channel the frontend already listens on, per watched
/// document, so a pane can refetch (and flash) exactly its own files.
async fn notify_files_changed(
    state: &LocalState,
    root: &PathBuf,
    woven: &WovenState,
    marks: &mut HashMap<PathBuf, u64>,
) {
    for doc in woven.doc_paths() {
        let mut mark: u64 = 0xcbf2_9ce4_8422_2325;
        for path in woven.output_paths() {
            let Some(output) = woven.output(&path) else {
                continue;
            };
            if output.doc != doc {
                continue;
            }
            for byte in path.to_string_lossy().as_bytes() {
                mark = (mark ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
            }
            for byte in output.content.as_bytes() {
                mark = (mark ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        if marks.insert(doc.clone(), mark) == Some(mark) {
            continue;
        }
        let Ok(rel) = doc.strip_prefix(root) else {
            continue;
        };
        let rel = rel.to_string_lossy().replace('\\', "/");
        let id = state.index.id_for_path(&rel);
        if state.index.path_of(&id).is_none() {
            continue;
        }
        state
            .rooms
            .publish_run_event(
                &id,
                &serde_json::json!({ "files_changed": true, "doc": id }),
            )
            .await;
    }
}

/// Push every watched document's on-disk text into its live room.
///
/// `apply_external_source` no-ops when the room already holds the text, so
/// this is cheap for the common case and exactly right for the two that
/// matter: a document rewritten by a reverse edit, and a document changed
/// under the app by another program.
async fn reconcile_rooms(state: &LocalState, root: &PathBuf, woven: &WovenState) {
    for doc in woven.doc_paths() {
        let Ok(rel) = doc.strip_prefix(root) else {
            continue;
        };
        let rel = rel.to_string_lossy().replace('\\', "/");
        // Only documents the index knows are rooms anyone can have open.
        let id = state.index.id_for_path(&rel);
        if state.index.path_of(&id).is_none() {
            continue;
        }
        if let Ok(source) = std::fs::read_to_string(&doc) {
            // The room's own persist comes back through the watcher looking
            // exactly like an external edit. Reconciling it would set the
            // room to a text that may already be keystrokes old, reverting
            // what was typed during the weave — so the store's last persist
            // is skipped, and only genuinely external bytes go in.
            if state.store.was_own_write(&id, &source) {
                continue;
            }
            state.rooms.apply_external_source(&id, &source).await;
        }
    }
}
