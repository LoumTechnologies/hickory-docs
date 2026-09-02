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
use std::time::{Duration, Instant, SystemTime};

use anyhow::{Context, Result};
use fs2::FileExt;
use notify::{EventKind, RecursiveMode, Watcher};

use crate::{ExecutorChoice, RunMode, expand_docs, output_lineage, run_doc};
use state::{OutputState, WovenState};

/// Turn a `notify` failure into an error that names the limit that was hit.
///
/// The raw messages are the OS's, and they describe the wrong thing. Linux
/// reports the per-user inotify **instance** cap as `Too many open files
/// (os error 24)`, which reads like a descriptor leak in `hick` — it is not;
/// it is a machine-wide budget that editors, language servers, file managers,
/// and every other `hick up` are spending at the same time. The **watch** cap
/// arrives as `No space left on device (os error 28)`, which reads like a full
/// disk. Neither is guessable from the message, both are one `sysctl` away,
/// and someone hitting either one is on their own machine with nobody to
/// debug it for them.
/// What a window can ask a running loop to do, besides react to files.
#[derive(Debug)]
pub enum UpCommand {
    /// Overwrite a held output with what its document produces — the one
    /// path that writes over held bytes, taken only when asked by name.
    Regenerate(PathBuf),
}

pub(crate) fn watch_error(err: notify::Error) -> anyhow::Error {
    let raw = err.to_string();
    let io_kind = match &err.kind {
        notify::ErrorKind::Io(e) => Some(e.raw_os_error()),
        _ => None,
    };
    let hint = match io_kind.flatten() {
        // EMFILE / ENFILE — out of inotify instances (or file descriptors).
        Some(24) | Some(23) if cfg!(target_os = "linux") => Some(
            "This is the per-user inotify instance limit, not a bug in hick — every \
             editor, language server, and file watcher on this machine spends from \
             the same budget.\n\
             \n\
             Check it and raise it:\n\
             \x20 cat /proc/sys/fs/inotify/max_user_instances    # often 128\n\
             \x20 sudo sysctl fs.inotify.max_user_instances=512\n\
             \n\
             To keep it across reboots, put `fs.inotify.max_user_instances=512` in \
             /etc/sysctl.d/99-inotify.conf. Closing a few watchers (editors, `hick \
             up` in other worktrees) frees instances immediately.",
        ),
        Some(24) | Some(23) => Some(
            "The process is out of file descriptors. Raise the limit (`ulimit -n`) \
             or close some watchers — other editors and `hick up` runs each hold \
             descriptors for the directories they watch.",
        ),
        // ENOSPC — out of inotify watches, which is not about disk space.
        Some(28) if cfg!(target_os = "linux") => Some(
            "Despite the message, this is the inotify watch limit rather than disk \
             space: watching a directory recursively takes one watch per \
             subdirectory, so a tree with a large node_modules/ or target/ can \
             exhaust it.\n\
             \n\
             Check it and raise it:\n\
             \x20 cat /proc/sys/fs/inotify/max_user_watches\n\
             \x20 sudo sysctl fs.inotify.max_user_watches=524288\n\
             \n\
             Pointing `hick up` at the directory that holds the documents, rather \
             than the repository root, watches far less.",
        ),
        _ => None,
    };
    match hint {
        Some(hint) => anyhow::anyhow!("{raw}\n\n{hint}"),
        None => anyhow::Error::new(err),
    }
}

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

/// How long to wait for the files in a batch to stop changing, and how often to
/// look while waiting.
///
/// The debounce above answers "has the directory been quiet", which is not the
/// same question as "has this file finished being written" — and an editor that
/// truncates and then dribbles its buffer in stalls for longer than any quiet
/// period you would want to wait on every save. When that happens the batch
/// fires on a half-written file, the loop maps it back, and the document loses
/// whatever had not been written yet.
///
/// `ingest.rs` already answers the same question for a file arriving in an
/// inbox (`has_settled`, size-stability polling). This is that idea applied to
/// the one other place the loop reads a file somebody else is writing.
const BATCH_SETTLE_BUDGET: Duration = Duration::from_millis(2_000);
const BATCH_SETTLE_POLL: Duration = Duration::from_millis(40);
/// How long everything must hold still before the batch is believed.
///
/// Two consecutive equal polls are NOT enough, and that is the whole subtlety:
/// a writer that pauses between chunks looks identical either side of a short
/// poll, so "unchanged since 40ms ago" reports a half-written file as finished.
/// Stability has to be measured as a continuous window longer than the pauses a
/// writer takes, not as a single comparison.
///
/// 300ms is a heuristic and worth naming as one. It comfortably exceeds the
/// gaps a real editor leaves — those are sub-millisecond, since an editor
/// writes its buffer in one call or a few back-to-back — while staying under
/// what a person notices on save. A writer that stalls longer than this between
/// chunks still defeats it; `BATCH_SETTLE_BUDGET` bounds the damage, and the
/// events its later writes produce bring the loop back to correct the document.
const BATCH_SETTLE_STABLE_FOR: Duration = Duration::from_millis(300);

/// What every path in the batch looks like right now: size and modification
/// time, or `None` where it does not exist.
///
/// Size alone would miss a rewrite that happens to land on the same length, and
/// mtime alone is too coarse on filesystems with second granularity. Together
/// they are cheap and catch both.
fn fingerprints(batch: &HashSet<PathBuf>) -> HashMap<PathBuf, Option<(u64, Option<SystemTime>)>> {
    batch
        .iter()
        .map(|path| {
            let stat = std::fs::metadata(path)
                .ok()
                .map(|m| (m.len(), m.modified().ok()));
            (path.clone(), stat)
        })
        .collect()
}

/// Wait until nothing in `batch` is still being written.
///
/// Costs `BATCH_SETTLE_STABLE_FOR` in the common case, because that is how long
/// a file nobody is touching has to sit still before it is believed.
///
/// Returns whether everything held still. A `false` means something is STILL
/// being written, and the caller must not read it: mapping a half-written file
/// back into the document is exactly what
/// `docs/guarantees/authoring/an-output-edit-lands-in-its-document.md`
/// forbids, and a save arriving in small chunks takes far longer than any
/// budget worth waiting by default.
///
/// Handling the batch anyway was the old answer, on the reasoning that later
/// events would correct the document. They do — but the guarantee is about
/// what the document contains *meanwhile*, and "it heals a moment later" is
/// not the same promise. Dropping the batch is not an option either: the
/// update would be lost outright if the writer finished in the gap. So the
/// caller waits instead, absorbing new events as they arrive.
async fn settle_batch(batch: &HashSet<PathBuf>) -> bool {
    let deadline = Instant::now() + BATCH_SETTLE_BUDGET;
    let mut last = fingerprints(batch);
    let mut unchanged_since = Instant::now();
    // How far apart this writer's own writes have been. A fixed window is a
    // guess about somebody else's editor, and the wrong guess is silent: a
    // save dribbled out in chunks 200ms apart looks FINISHED to a 300ms
    // window the moment one chunk is late, which on a loaded machine is
    // routine. Waiting three times the longest gap seen so far asks the
    // writer how patient to be instead of assuming. Costs nothing in the
    // common case — a save nobody is still writing produces no gaps at all.
    //
    // **This narrows the window; it does not close it.** Nothing watching
    // from outside can distinguish "paused mid-save" from "finished" — a
    // writer that stalls longer than the tolerance is indistinguishable from
    // one that is done, and no constant makes that false. What is bounded is
    // the exposure: a stall has to exceed three times the writer's own
    // established cadence before a partial file can be read, and the next
    // event corrects the document. An editor that writes to a temp file and
    // renames has no exposure at all, which is why most do it.
    let mut last_change = Instant::now();
    let mut longest_gap = Duration::ZERO;
    while Instant::now() < deadline {
        tokio::time::sleep(BATCH_SETTLE_POLL).await;
        let current = fingerprints(batch);
        if current == last {
            if unchanged_since.elapsed() >= BATCH_SETTLE_STABLE_FOR.max(longest_gap * 3) {
                return true;
            }
        } else {
            longest_gap = longest_gap.max(last_change.elapsed());
            last_change = Instant::now();
            last = current;
            unchanged_since = Instant::now();
        }
    }
    false
}

/// What `hick up` was asked to do.
pub struct UpConfig {
    /// The directory (or single document) to keep woven.
    pub root: PathBuf,
    /// Parameter overrides, as every other command takes them.
    pub params: Vec<(String, String)>,
    /// Execute exec cells on every change instead of weaving from recorded
    /// transcripts.
    pub run: bool,
    /// Executor for `--run` (`HICKORY_EXECUTOR`, default sandbox). The
    /// always-on loop must not be quietly less confined than a one-shot
    /// `hick run` of the same document.
    pub executor: ExecutorChoice,
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

    // Drain the inbox BEFORE listing documents, so a transcript waiting there
    // when the loop starts becomes a note that the first weave covers rather
    // than one that sits unwoven until the next file event.
    let inbox = crate::ingest::InboxConfig::from_env()?;
    drain_inbox(&watch_root, &inbox);

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
    .map_err(watch_error)
    .context("failed to start watching for file changes")?;
    watcher
        .watch(&watch_root, RecursiveMode::Recursive)
        .map_err(watch_error)
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
        // Quiet is not the same as finished. Wait for the files themselves to
        // stop changing before anything reads them — however long that takes.
        // A save written in small chunks outlasts any fixed budget, and
        // reading one mid-write puts half a file into somebody's document.
        let mut waited = Duration::ZERO;
        while !settle_batch(&batch).await {
            waited += BATCH_SETTLE_BUDGET;
            // Whatever is being written is still being written. Take anything
            // new with it rather than leaving it for a later cycle.
            while let Ok(path) = rx.try_recv() {
                batch.insert(path);
            }
            // Said once, not every round: a file that never stops changing
            // would otherwise look like the loop had silently stopped working.
            if waited == BATCH_SETTLE_BUDGET * 3 {
                eprintln!(
                    "hick up: still being written, waiting before weaving: {}",
                    batch
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
        // A file dropped in the inbox becomes a note, and the note joins this
        // batch so it is woven in the same cycle rather than the next one.
        if batch
            .iter()
            .any(|path| path.starts_with(inbox.inbox(&watch_root)))
        {
            batch.extend(drain_inbox(&watch_root, &inbox));
        }
        if let Err(e) = handle_batch(batch, &config, &mut state).await {
            eprintln!("error: {e:#}");
        }
    }

    state.release_read_only();
    Ok(())
}

/// Turn everything in the inbox into notes, returning the notes written.
///
/// Reports every outcome, including skips: a file that silently stayed in the
/// inbox with no explanation is how someone concludes the feature is broken.
/// A failure here is printed and never propagated — an unreadable drop must not
/// take down a loop that is keeping someone's notes woven.
fn drain_inbox(root: &Path, config: &crate::ingest::InboxConfig) -> Vec<PathBuf> {
    let outcomes = match crate::ingest::ingest_inbox(root, config) {
        Ok(outcomes) => outcomes,
        Err(e) => {
            eprintln!("hick up: could not read the inbox: {e:#}");
            return Vec::new();
        }
    };
    let mut written = Vec::new();
    for outcome in &outcomes {
        eprintln!("hick up: {outcome}");
        if let crate::ingest::Outcome::Ingested { note, .. } = outcome {
            written.push(note.clone());
        }
    }
    written
}

/// Weave one document, write its outputs, and record what was written.
///
/// The document's source is recorded even when the weave fails. A document
/// that does not parse is still a document being watched: recording its text
/// is what makes the *next* save of it register as a change, so fixing the
/// error brings it back without restarting the loop.
pub(crate) async fn weave_document(
    doc: &Path,
    config: &UpConfig,
    state: &mut WovenState,
) -> Result<()> {
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
    let run = run_doc(doc, &config.params, mode, config.executor).await?;

    let base = doc.parent().unwrap_or(Path::new(".")).to_path_buf();
    let mut produced: HashSet<PathBuf> = HashSet::new();

    // The same rule `write_outputs_detailed` applies, applied here too because
    // this loop has its own writer: a weave that cannot find a cell's
    // recording must not write `[never run]` over the artifact a real run
    // produced. `hick up` weaves on every save, so without this the loop
    // destroyed a chart the moment anyone typed a word — while the app's live
    // preview, which executes, went on rendering it. See
    // `docs/guarantees/verification/a-weave-without-a-recording-keeps-the-artifact.md`.
    let missing_recording = run.result.outputs_missing_a_recording();
    let weave_target = run.doc.weave_path.as_deref();

    let mut rel_paths: Vec<&String> = run.result.files.keys().collect();
    rel_paths.sort();
    for rel_path in rel_paths {
        // Binary outputs have no text to diff and no lineage to map through,
        // so they are written by the normal run path and not watched here.
        let Some(hick_exec::node::FileContent::Text(content)) = run.result.files.get(rel_path)
        else {
            continue;
        };
        let full = crate::contained_output_path(&base, rel_path)?;
        // Not tracked either: these bytes are not ours. `up` re-adopts the
        // file the moment a run gives it something real to say.
        if missing_recording.contains(rel_path.as_str()) && full.exists() {
            continue;
        }
        // The weave target too. It used to be exempt as "this weave's own
        // report", and the report it wrote over a committed rendering was
        // `[never run]` in place of recorded output — the truth about the
        // recordings the weave found, and a destruction of the ones the file
        // held. A rendering that lags is consistent with itself; a run brings
        // it forward. Only when it already exists: a first weave has nothing
        // to keep.
        if Some(rel_path.as_str()) == weave_target && !missing_recording.is_empty() && full.exists()
        {
            eprintln!(
                "  kept {}: {} cell(s) have no recording; run the document to bring it forward",
                full.display(),
                run.result.never_run.len()
            );
            continue;
        }
        let provenance = output_lineage(&run, rel_path).unwrap_or_default();
        produced.insert(full.clone());
        state.write_output(
            &base,
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
    // Included/upstream files are documents too: provenance can name them as
    // an edit's destination, and the staleness guard in
    // `reverse::apply_to_documents` can only protect a document whose
    // at-weave-time text was recorded. The table's paths are canonical, which
    // is also how provenance spells them.
    for spliced in &run.result.span_files {
        if let Ok(source) = std::fs::read_to_string(spliced) {
            state.record_doc(PathBuf::from(spliced), source);
        }
    }

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
pub(crate) async fn handle_batch(
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
            // The file is HELD, not restored. This used to put the file back
            // to match the document, on the argument that a silent fork is
            // worse than a refused edit. The argument was right about
            // "silent" and wrong about the remedy: restoring destroyed what
            // a person typed, and what `git checkout` and `git pull` wrote,
            // a second after they wrote it. The bytes on disk came from
            // somebody; the loop has nothing truer to put in their place. So
            // they stay, the file is marked held — the tree and the pane say
            // so, loudly — and the hold lifts when the document catches up
            // or when the person asks for the file to be regenerated.
            Err(e) => {
                let reason = format!("{e:#}");
                eprintln!(
                    "holding {}: the edit could not be carried into {}\n  {reason}\n  \
                     The file is left as it is and marked held; edit the document, or \
                     regenerate the file from it.",
                    path.display(),
                    doc.display()
                );
                state.hold_output(path, reason);
                Ok(None)
            }
        },
        Err(e) => {
            let message = reverse::refusal_message(path, &doc, &output.content, &e);
            eprintln!("{message}");
            state.hold_output(path, reverse::refusal_reason(&e));
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

#[cfg(test)]
mod settle_tests {
    use super::{BATCH_SETTLE_STABLE_FOR, settle_batch};
    use std::collections::HashSet;
    use std::io::Write as _;
    use std::path::PathBuf;
    use std::time::Duration;

    /// A writer that stutters: several chunks close together, then one late.
    ///
    /// This is the shape that defeated the fixed window, and it is not exotic
    /// — a 200ms cadence stalling to 400ms once is routine on a loaded
    /// machine. The old 300ms window declared the file finished during that
    /// one late gap and a half-written save reached the document. The window
    /// now waits twice the longest gap the writer has actually shown, so the
    /// writer sets the patience rather than a constant guessing at it.
    ///
    /// The first gap has to be short enough to be OBSERVED: nothing can know a
    /// writer is mid-stream before seeing it write once, which is why the
    /// batch exists at all — an event is what put the file here.
    #[tokio::test(flavor = "multi_thread")]
    async fn a_writer_that_stutters_is_still_waited_for() {
        let dir = tempfile::tempdir().expect("a temp dir");
        let path = dir.path().join("dribbled.txt");
        std::fs::write(&path, b"first-chunk-----").expect("create");

        // Under the window, under the window, then well over it.
        let gaps = [
            BATCH_SETTLE_STABLE_FOR / 2,
            BATCH_SETTLE_STABLE_FOR / 2,
            BATCH_SETTLE_STABLE_FOR + Duration::from_millis(100),
        ];
        let expected = (gaps.len() as u64 + 1) * 16;
        let writer = {
            let path = path.clone();
            std::thread::spawn(move || {
                let mut file = std::fs::OpenOptions::new()
                    .append(true)
                    .open(&path)
                    .expect("open");
                for gap in gaps {
                    std::thread::sleep(gap);
                    file.write_all(b"0123456789abcdef").expect("write");
                    file.flush().expect("flush");
                }
            })
        };

        let batch: HashSet<PathBuf> = [path.clone()].into_iter().collect();
        while !settle_batch(&batch).await {}
        let settled_len = std::fs::metadata(&path).expect("stat").len();

        writer.join().expect("the writer finished");
        assert_eq!(
            settled_len, expected,
            "settled while the file was still being written: saw {settled_len} \
             of {expected} bytes"
        );
    }

    /// The common case must not pay for the uncommon one: a file nobody is
    /// touching still settles in about the fixed window.
    #[tokio::test(flavor = "multi_thread")]
    async fn a_file_nobody_is_writing_settles_promptly() {
        let dir = tempfile::tempdir().expect("a temp dir");
        let path = dir.path().join("quiet.txt");
        std::fs::write(&path, b"done").expect("create");
        let batch: HashSet<PathBuf> = [path].into_iter().collect();

        let start = std::time::Instant::now();
        while !settle_batch(&batch).await {}
        assert!(
            start.elapsed() < BATCH_SETTLE_STABLE_FOR * 3,
            "a quiet file took {:?} to settle",
            start.elapsed()
        );
    }
}

#[cfg(test)]
mod watch_error_tests {
    use super::watch_error;

    fn os(code: i32) -> notify::Error {
        notify::Error::io(std::io::Error::from_raw_os_error(code))
    }

    #[test]
    fn emfile_names_the_inotify_instance_limit_not_a_leak() {
        let msg = format!("{:#}", watch_error(os(24)));
        assert!(
            msg.contains("Too many open files"),
            "keeps the OS text: {msg}"
        );
        if cfg!(target_os = "linux") {
            assert!(msg.contains("max_user_instances"), "names the knob: {msg}");
            assert!(msg.contains("sysctl"), "says how to raise it: {msg}");
        }
    }

    #[test]
    fn enospc_says_it_is_watches_and_not_disk_space() {
        if !cfg!(target_os = "linux") {
            return;
        }
        let msg = format!("{:#}", watch_error(os(28)));
        assert!(msg.contains("max_user_watches"), "names the knob: {msg}");
        assert!(
            msg.contains("rather than disk space"),
            "corrects the misleading message: {msg}"
        );
    }

    #[test]
    fn an_unrelated_error_is_passed_through_unchanged() {
        // ENOENT has nothing to do with a limit; inventing advice for it would
        // teach the reader to skip the advice that matters.
        let msg = format!("{:#}", watch_error(os(2)));
        assert!(!msg.contains("sysctl"), "no spurious advice: {msg}");
    }
}
