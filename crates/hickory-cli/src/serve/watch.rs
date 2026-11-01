//! Read-only filesystem observation beside the editor rooms.
//!
//! Index new documents and reconcile external source saves. Neither startup,
//! discovery, output edits nor refresh commands mint generation authority.
//! Publication belongs to the explicit root/entry generation action.
//! The caller owns directory coordination; this observer takes no second lock.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use anyhow::{Context as _, Result};
use notify::{EventKind, RecursiveMode, Watcher as _};

use super::LocalState;
use crate::up::state::WovenState;
use crate::up::UpCommand;

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

/// Observe the selected scope until stopped. Refresh the read-only index after
/// registration so changes between preparation and native activation are seen.
pub fn spawn(state: LocalState) -> Result<WatchGuard> {
    let root = state
        .index
        .root()
        .canonicalize()
        .with_context(|| format!("cannot watch {}", state.index.root().display()))?;

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<PathBuf>();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        let Ok(event) = res else { return };
        if matches!(event.kind, EventKind::Access(_)) {
            return;
        }
        for path in event.paths {
            let _ = tx.send(path);
        }
    })
    // The same diagnosis `hick up` gives. Without it the app reported the
    // OS's own words — `Too many open files (os error 24)` — for the
    // per-user inotify INSTANCE cap, which reads like a descriptor leak in
    // hick and is actually a machine-wide budget every editor and language
    // server spends from. Two near-identical blocks, and only the CLI's
    // explained itself; the app is the copy a downloader meets first.
    .map_err(crate::up::watch_error)
    .context("failed to start watching for file changes")?;
    watcher
        .watch(&root, if state.folder_open { RecursiveMode::Recursive } else { RecursiveMode::NonRecursive })
        .map_err(crate::up::watch_error)
        .with_context(|| format!("failed to watch {}", root.display()))?;

    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
    let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel::<UpCommand>();
    *state.up_commands.lock().expect("up command inbox") = Some(command_tx);
    let task = tokio::spawn(run_loop(state, root, watcher, rx, command_rx, stop_rx));
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
    mut commands: tokio::sync::mpsc::UnboundedReceiver<UpCommand>,
    mut stop: tokio::sync::oneshot::Receiver<()>,
) {
    let mut woven = WovenState::default();
    // Per-document fingerprint of the outputs' bytes, so "files changed" is
    // published only when a batch actually rewrote something — not for every
    // echo the watcher hears.
    let mut output_marks: HashMap<PathBuf, u64> = HashMap::new();

    // Refresh after watcher installation: a file may appear between prepare's
    // inventory and the native watch becoming active. This pass only indexes.
    discover_changed_path(&state, &root, state.folder_open);
    // Observation installs no generation context and publishes no outputs.
    reconcile_rooms(&state, &root, &woven).await;
    publish_held(&state, &root, &woven);

    loop {
        let first = tokio::select! {
            received = rx.recv() => match received {
                Some(path) => path,
                None => break,
            },
            command = commands.recv() => {
                match command {
                    Some(UpCommand::Resolve { request, reply }) => {
                        if !reply.is_closed() {
                            let result = woven.resolve_output(&request).map_err(|e| format!("{e:#}"));
                            let _ = reply.send(result);
                        }
                    }
                    Some(UpCommand::Reweave(_)) => {}
                    None => {}
                }
                publish_held(&state, &root, &woven);
                notify_files_changed(&state, &root, &woven, &mut output_marks).await;
                continue;
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

        // A `.md` file the index has never seen is a document created by
        // something other than the app — `touch notes.md`, a git checkout,
        // an agent. Register it so the app can list and open it without a
        // restart. Observation never generates outputs.
        for path in &batch { discover_changed_path(&state, path, state.folder_open); }

        // Source saves and discovery events only reconcile editor rooms.
        reconcile_rooms(&state, &root, &woven).await;
        publish_held(&state, &root, &woven);
        notify_files_changed(&state, &root, &woven, &mut output_marks).await;
    }

    // However the loop ends, the marks come off: `hick run`, git, and every
    // other tool must find ordinary files afterwards.
    woven.release_read_only();
}
/// Copy the loop's held set where the routes can read it, as root-relative
/// paths.
pub(crate) fn publish_held(state: &LocalState, root: &PathBuf, woven: &WovenState) {
    let mut held = HashMap::new();
    for (path, diverged) in woven.held() {
        if let Ok(rel) = path.strip_prefix(root) {
            held.insert(
                rel.to_string_lossy().replace('\\', "/"),
                super::DivergedOutput {
                    kind: diverged.kind.as_str(),
                    reason: diverged.reason.clone(),
                    base: diverged.base.clone(),
                    theirs: diverged.theirs.clone(),
                },
            );
        }
    }
    *state.held.lock().expect("held outputs") = held;
}

/// Tell every open window that this batch may have rewritten output files.
///
/// Fingerprinted output observations can notify a generated-file pane.
/// An opening observer does not create those publication observations. The event
/// rides the run channel the frontend already listens on, per watched
/// document, so a pane can refetch (and flash) exactly its own files.
pub(crate) async fn notify_files_changed(
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
pub(crate) async fn reconcile_rooms(state: &LocalState, root: &PathBuf, _woven: &WovenState) {
    for doc in state
        .index
        .entries()
        .iter()
        .filter_map(|(id, _)| state.index.absolute(id))
    {
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
/// Directory notifications are common on macOS. A folder observer may refresh
/// that subtree's index, but this helper can neither evaluate nor publish files.
pub(crate) fn discover_changed_path(state: &LocalState, changed: &std::path::Path, folder_authorized: bool) {
    let canonical = changed.canonicalize().unwrap_or_else(|_| changed.to_path_buf());
    let changed = canonical.as_path();
    if crate::up::is_noise(changed) {
        return;
    }
    let root = state.index.root();
    let scope = if changed.starts_with(root) {
        changed
    } else if root.starts_with(changed) && changed.is_dir() {
        root
    } else {
        return;
    };
    let candidates = if scope.is_dir() {
        if !folder_authorized {
            return;
        }
        match hick_literate::discover_default_documents(scope) {
            Ok(paths) => paths,
            Err(error) => {
                log::warn!("could not refresh document index: {error:#}");
                return;
            }
        }
    } else {
        vec![scope.to_path_buf()]
    };
    for path in candidates {
        if !folder_authorized {
            let Ok(relative) = path.strip_prefix(root) else { continue; };
            let id = state.index.id_for_path(&relative.to_string_lossy().replace('\\', "/"));
            if state.index.path_of(&id).is_none() { continue; }
        }
        if !path.is_file() {
            continue;
        }
        let path = path.canonicalize().unwrap_or(path);
        if let Ok(relative) = path.strip_prefix(root) {
            let _ = state
                .index
                .discover(&relative.to_string_lossy().replace('\\', "/"));
        }
    }
}
