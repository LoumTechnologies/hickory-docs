//! `hick up` — weave a folder and keep it woven.
//!
//! One command that makes a directory of `.hick` documents behave like
//! ordinary source: every document's outputs are on disk as real files, any
//! editor can open them, and an edit saved in one of them lands back in the
//! document it came from. The loop runs until interrupted.
//!
//! Three things have to be true for that to feel like editing files rather
//! than operating machinery, and each is handled in a different place:
//!
//! * **The loop must not react to itself.** It writes the files it watches.
//!   [`state::WovenState`] remembers the exact bytes of every write and
//!   discards any event whose file still holds them.
//! * **The loop must not be the second writer.** Two `hick up` processes on
//!   one directory would each see the other's writes as user edits. A single
//!   advisory lock on the directory makes the second one refuse to start.
//! * **A refusal must arrive before the typing, not after.** An output file
//!   with no editable byte is marked read-only on disk, so the editor says so
//!   when the file is opened. Everything finer than whole-file is decided on
//!   save by lineage, and a refused save restores the file and explains why.

pub mod reverse;
pub mod state;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use fs2::FileExt;
use notify::{EventKind, RecursiveMode, Watcher};

use crate::{ExecutorChoice, RunMode, expand_docs, output_lineage, run_doc};
use state::{OutputState, WovenState};

/// How long the directory must be quiet before a burst of events is treated
/// as finished.
///
/// Editors do not save a file in one write. Vim writes a backup, truncates,
/// writes, and renames; VS Code writes a sibling and renames over the target.
/// Reacting to the first event in that sequence reads a half-written file.
const WEAVE_DEBOUNCE: Duration = Duration::from_millis(120);

/// The same, for a loop that executes cells. Running is expensive and a burst
/// of saves should collapse into one run, so this waits longer for quiet.
const RUN_DEBOUNCE: Duration = Duration::from_millis(500);

/// What `hick up` was asked to do.
pub struct UpConfig {
    /// The directory (or single document) to keep woven.
    pub root: PathBuf,
    /// Parameter overrides, as every other command takes them.
    pub params: Vec<(String, String)>,
    /// Execute exec cells on every change instead of weaving from recorded
    /// transcripts.
    pub run: bool,
}

impl UpConfig {
    fn mode(&self) -> RunMode {
        if self.run {
            RunMode::Execute
        } else {
            RunMode::Weave
        }
    }

    fn debounce(&self) -> Duration {
        if self.run {
            RUN_DEBOUNCE
        } else {
            WEAVE_DEBOUNCE
        }
    }
}

/// Hold the directory for as long as the holder lives.
///
/// The lock is advisory and process-scoped: it stops a second writer that
/// asks — `hick up`, or the desktop app opening the same folder — which is the
/// case that actually corrupts things. It does not pretend to stop a text
/// editor, which never asks.
///
/// Public because the desktop app takes the same lock: two processes weaving
/// one directory is the same bug whichever front door they came through.
pub struct DirectoryLock {
    _file: std::fs::File,
    path: PathBuf,
}

impl DirectoryLock {
    pub fn acquire(root: &Path) -> Result<Self> {
        let dir = root.join(".hick-cache");
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("failed to create {}", dir.display()))?;
        let path = dir.join("up.lock");
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)
            .with_context(|| format!("failed to open {}", path.display()))?;

        file.try_lock_exclusive().map_err(|_| {
            anyhow::anyhow!(
                "{} is already open in another Hickory Docs process.\n\
                 Two of them would each treat the other's writes as your edits.\n\
                 Close the other one — `hick up` or the desktop app — or work in\n\
                 a different directory.\n\
                 If no other process is running, remove {} and try again.",
                root.display(),
                path.display(),
            )
        })?;
        Ok(Self { _file: file, path })
    }
}

impl Drop for DirectoryLock {
    fn drop(&mut self) {
        // The advisory lock is released with the file handle; the empty file
        // is left behind on purpose so the path in the error message above
        // stays meaningful.
        let _ = &self.path;
    }
}

/// Run the loop until interrupted.
pub async fn run(config: UpConfig) -> Result<()> {
    let root = config
        .root
        .canonicalize()
        .with_context(|| format!("cannot find {}", config.root.display()))?;
    let watch_root = if root.is_dir() {
        root.clone()
    } else {
        root.parent().unwrap_or(Path::new(".")).to_path_buf()
    };

    let _lock = DirectoryLock::acquire(&watch_root)?;

    let mut state = WovenState::default();
    // `expand_docs` already refuses an empty directory, and its message
    // covers the three ways to fix it. Do not add a second one here.
    let docs = expand_docs(&root)?;

    // Start watching BEFORE the first weave, not after.
    //
    // The first pass can take as long as the document's slowest cell, and it
    // puts the output files on disk early — so there is a window in which a
    // user can open and edit a generated file that the loop is not yet
    // listening for. Events are queued from here on and drained once the
    // initial weave is done, so an edit made during startup is late rather
    // than lost.
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<PathBuf>();
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
        .watch(&watch_root, RecursiveMode::Recursive)
        .with_context(|| format!("failed to watch {}", watch_root.display()))?;

    eprintln!(
        "hick up: weaving {} document(s) in {}",
        docs.len(),
        watch_root.display()
    );
    // Under `--run` the first pass can take as long as the document's slowest
    // cell. That matters because the pipeline stages a document's output files
    // to disk *before* executing anything, so those files exist — and can be
    // opened and edited — while the loop is still waiting on the run and has
    // recorded nothing about them. An edit made in that window would be
    // overwritten when the run finished, and would then look like an echo of
    // the loop's own write, so it would disappear with no error at all.
    //
    // A weave-only pass first closes the window: it is fast, it writes the
    // same files, and it leaves the loop with a baseline to recognise a user's
    // edit against.
    if config.run {
        establish_baselines(&docs, &config, &mut state).await;
    }

    let mut failed = 0usize;
    for doc in &docs {
        if let Err(e) = weave_document(doc, &config, &mut state).await {
            eprintln!("error: {e:#}");
            failed += 1;
        }
    }
    report_ready(&state, &config, docs.len(), failed);

    let debounce = config.debounce();
    loop {
        let first = tokio::select! {
            received = rx.recv() => match received {
                Some(path) => path,
                None => break,
            },
            // Ctrl-C has to land here rather than in a signal handler: the
            // read-only marks have to come off before the process goes away,
            // and a handler that runs after `state` is gone cannot do that.
            _ = tokio::signal::ctrl_c() => {
                eprintln!("\nhick up: stopping");
                break;
            }
        };
        let mut batch = HashSet::new();
        batch.insert(first);
        // Collect until the directory has been quiet for a full debounce.
        loop {
            match tokio::time::timeout(debounce, rx.recv()).await {
                Ok(Some(path)) => {
                    batch.insert(path);
                }
                Ok(None) => break,
                Err(_) => break,
            }
        }
        if let Err(e) = handle_batch(batch, &config, &mut state).await {
            eprintln!("error: {e:#}");
        }
    }

    state.release_read_only();
    Ok(())
}

/// Weave one document, write its outputs, and record what was written.
///
/// The document's source is recorded even when the weave fails. A document
/// that does not parse is still a document being watched: recording its text
/// is what makes the *next* save of it register as a change, so fixing the
/// error brings it back without restarting the loop.
async fn weave_document(doc: &Path, config: &UpConfig, state: &mut WovenState) -> Result<()> {
    weave_document_as(doc, config.mode(), config, state).await
}

/// Weave one document in an explicit mode.
///
/// Separate from [`weave_document`] because startup under `--run` needs a fast
/// weave-only pass before the real one — see [`establish_baselines`].
async fn weave_document_as(
    doc: &Path,
    mode: RunMode,
    config: &UpConfig,
    state: &mut WovenState,
) -> Result<()> {
    if let Ok(source) = std::fs::read_to_string(doc) {
        state.record_doc(doc.to_path_buf(), source);
    }
    let run = run_doc(doc, &config.params, mode, ExecutorChoice::Local).await?;

    let base = doc.parent().unwrap_or(Path::new(".")).to_path_buf();
    let mut produced: HashSet<PathBuf> = HashSet::new();

    let mut rel_paths: Vec<&String> = run.result.files.keys().collect();
    rel_paths.sort();
    for rel_path in rel_paths {
        // Binary outputs have no text to diff and no lineage to map through,
        // so they are written by the normal run path and not watched here.
        let Some(hick_exec::node::FileContent::Text(content)) = run.result.files.get(rel_path)
        else {
            continue;
        };
        let provenance = output_lineage(&run, rel_path).unwrap_or_default();
        let full = base.join(rel_path);
        produced.insert(full.clone());
        state.write_output(
            &full,
            OutputState {
                doc: doc.to_path_buf(),
                rel_path: rel_path.clone(),
                content: content.clone(),
                provenance,
            },
        )?;
    }

    state.retain_outputs_of(doc, &produced);
    state.record_doc(doc.to_path_buf(), run.source.clone());

    if config.run && !run.result.never_run.is_empty() {
        eprintln!(
            "  {}: {} cell(s) never ran",
            doc.display(),
            run.result.never_run.len()
        );
    }
    Ok(())
}

/// Weave every document without executing, purely to record what is on disk.
///
/// Errors are swallowed: this is a preparatory pass, and a document that
/// cannot be woven will fail the real pass a moment later with a message
/// worth reading. Reporting it twice would just be noise.
async fn establish_baselines(docs: &[PathBuf], config: &UpConfig, state: &mut WovenState) {
    for doc in docs {
        let _ = weave_document_as(doc, RunMode::Weave, config, state).await;
    }
}

/// Act on one debounced batch of changed paths.
///
/// Saved output files are processed first and documents second: an output
/// edit rewrites the document it came from, so consuming edits before
/// re-weaving means one weave covers both the user's document edits and the
/// edits carried back out of the generated files.
async fn handle_batch(
    batch: HashSet<PathBuf>,
    config: &UpConfig,
    state: &mut WovenState,
) -> Result<()> {
    let mut dirty_docs: HashSet<PathBuf> = HashSet::new();
    let mut saved_outputs: Vec<PathBuf> = Vec::new();

    for path in batch {
        if is_noise(&path) {
            continue;
        }
        let path = path.canonicalize().unwrap_or(path);
        if state.output(&path).is_some() {
            if !state.is_echo(&path) {
                saved_outputs.push(path);
            }
            continue;
        }
        if path.extension().is_some_and(|e| e == "hick") {
            let changed = match std::fs::read_to_string(&path) {
                Ok(current) => state.doc_source(&path) != Some(&current),
                // A document that vanished, or is mid-save and unreadable,
                // is worth one more look on the next event, not an error.
                Err(_) => false,
            };
            if changed {
                dirty_docs.insert(path);
            }
        }
    }

    saved_outputs.sort();
    for path in saved_outputs {
        match consume_output_save(&path, state) {
            Ok(Some(doc)) => {
                dirty_docs.insert(doc);
            }
            Ok(None) => {}
            Err(e) => eprintln!("error: {e:#}"),
        }
    }

    let mut docs: Vec<PathBuf> = dirty_docs.into_iter().collect();
    docs.sort();
    for doc in docs {
        eprintln!("  weaving {}", doc.display());
        if let Err(e) = weave_document(&doc, config, state).await {
            eprintln!("error: {e:#}");
        }
    }
    Ok(())
}

/// Carry one saved output file back into its document.
///
/// Returns the document that changed, or `None` when the save produced no
/// document edit at all (whitespace the weave normalizes, or a save with no
/// change in it).
fn consume_output_save(path: &Path, state: &mut WovenState) -> Result<Option<PathBuf>> {
    let Some(output) = state.output(path) else {
        return Ok(None);
    };
    let doc = output.doc.clone();

    let on_disk = match std::fs::read_to_string(path) {
        Ok(text) => text,
        // A file that is not valid UTF-8 right now is almost always one an
        // editor is part-way through writing. Leave it for the next event.
        Err(_) => return Ok(None),
    };

    let expected: HashMap<String, String> = state
        .doc_paths()
        .into_iter()
        .filter_map(|p| {
            let source = state.doc_source(&p)?.clone();
            Some((p.display().to_string(), source))
        })
        .collect();

    match reverse::source_edits_for_save(&output.content, &on_disk, &output.provenance) {
        Ok(edits) if edits.is_empty() => Ok(None),
        Ok(edits) => match reverse::apply_to_documents(&edits, &expected) {
            Ok(written) => {
                eprintln!(
                    "  {} → {}",
                    path.file_name().unwrap_or_default().to_string_lossy(),
                    written.join(", ")
                );
                Ok(Some(doc))
            }
            // The document moved under the edit — it was computed against
            // spans that no longer describe it, so applying it could put the
            // bytes anywhere.
            //
            // Restoring is not optional here. Leaving the file as the user
            // typed it would strand the two sides disagreeing *permanently*:
            // the document is not dirty, so nothing re-weaves, and no further
            // event ever arrives to reconcile them. A refused edit that
            // silently forks the file from its source is worse than a refused
            // edit, so the file goes back to matching the document and the
            // user is told which change was dropped.
            Err(e) => {
                eprintln!(
                    "refused an edit to {}\n  {e:#}\n  \
                     The file has been restored to match {}.",
                    path.display(),
                    doc.display()
                );
                state.restore_output(path)?;
                Ok(None)
            }
        },
        Err(e) => {
            let message = reverse::refusal_message(path, &doc, &output.content, &e);
            state.restore_output(path)?;
            eprintln!("{message}");
            Ok(None)
        }
    }
}

/// Paths that are never worth reacting to: our own temporaries, editor
/// scratch files, VCS internals, and the transcript cache.
fn is_noise(path: &Path) -> bool {
    if path.components().any(|c| {
        let name = c.as_os_str().to_string_lossy();
        name == ".git" || name == ".hick-cache" || name == "node_modules" || name == "target"
    }) {
        return true;
    }
    let Some(name) = path.file_name().map(|n| n.to_string_lossy().to_string()) else {
        return true;
    };
    // Vim writes `4913` to test whether a directory is writable, and leaves
    // `.swp`/`.swx` and `~` backups beside the file it is editing.
    name.ends_with('~')
        || name.ends_with(".swp")
        || name.ends_with(".swx")
        || name.ends_with(".tmp")
        || name == "4913"
        || name.starts_with(".#")
        || name.starts_with(".goutputstream")
}

/// Say what is being watched, and which files will refuse an edit, before the
/// user goes looking.
fn report_ready(state: &WovenState, config: &UpConfig, total_docs: usize, failed: usize) {
    let mut generated = Vec::new();
    let mut editable = 0usize;
    for path in state.output_paths() {
        match state.output(&path) {
            Some(o) if o.editable() => editable += 1,
            Some(_) => generated.push(path),
            None => {}
        }
    }
    eprintln!(
        "hick up: watching {total_docs} document(s), {editable} editable output file(s){}",
        if config.run {
            ", executing cells on change"
        } else {
            ""
        }
    );
    if failed > 0 {
        eprintln!(
            "  {failed} document(s) did not weave — the errors are above. They are \
             still being\n  watched: fix one and save, and it is woven on the spot."
        );
    }
    if !generated.is_empty() {
        eprintln!(
            "  {} file(s) are entirely generated and marked read-only:",
            generated.len()
        );
        for path in generated.iter().take(5) {
            eprintln!("    {}", path.display());
        }
        if generated.len() > 5 {
            eprintln!("    … and {} more", generated.len() - 5);
        }
    }
    eprintln!("  edit any output file and the change lands in its document. Ctrl-C to stop.");
}
