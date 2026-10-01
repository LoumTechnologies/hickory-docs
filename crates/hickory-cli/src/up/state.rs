//! What `hick up` knows about the files it has written.
//!
//! The loop writes woven outputs and then watches the same directory those
//! writes land in, so every write it makes is an event it will see. The
//! defence is uniform and lives here: remember the exact bytes written to
//! every output, and treat any filesystem event whose content still equals
//! those bytes as an echo of our own write. Nothing else needs to know that
//! the loop is watching itself.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use hickory_lineage::Provenance;

/// One woven output file, as we last wrote it.
pub struct OutputState {
    /// The `.hick` document this file was woven from.
    pub doc: PathBuf,
    /// The document-relative output path (`<hick:file path>`), which is the
    /// key `hickory_lineage` and the pipeline both use.
    pub rel_path: String,
    /// The exact bytes we last wrote. Reverse edits are diffed against this,
    /// never against a re-weave, so what the user edited is what we compare.
    pub content: String,
    /// Byte-precise lineage of `content`.
    pub provenance: Vec<Provenance>,
}

impl OutputState {
    /// Whether any byte of this file maps back to a document span.
    ///
    /// A file with no editable byte at all — a pure exec transcript, a
    /// generated report — can never accept an edit, so the loop marks it
    /// read-only on disk and the editor says so before the user types
    /// rather than after they save.
    pub fn editable(&self) -> bool {
        self.provenance.iter().any(|p| p.origin.source().is_some())
    }
}

/// Everything the loop remembers between events.
#[derive(Default)]
pub struct WovenState {
    /// Absolute output path → what we wrote there.
    outputs: HashMap<PathBuf, OutputState>,
    /// Absolute `.hick` path → its source when we last wove it.
    docs: HashMap<PathBuf, String>,
    /// Output files whose bytes on disk are NOT what the document produces —
    /// axis 3 of docs/specs/freeform/three-axes.md, *diverged* — and why.
    /// A held file is never rewritten by the loop: the bytes came from a
    /// person or from git, and the loop has nothing truer to put in their
    /// place. The state lifts the moment the document catches up — a weave
    /// produces exactly the bytes on disk — or when somebody picks a way
    /// out by name. See
    /// docs/guarantees/authoring/an-output-that-cannot-be-carried-back-is-held.md
    held: HashMap<PathBuf, Diverged>,
}

/// Why a produced file is not what its document produces, and the two
/// other versions a merge needs.
#[derive(Debug, Clone)]
pub struct Diverged {
    pub kind: DivergedKind,
    pub reason: String,
    /// The last bytes the document and the disk agreed on.
    pub base: String,
    /// What the document produces now. Empty when it cannot produce
    /// anything yet (`Kept`).
    pub theirs: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DivergedKind {
    /// Somebody wrote the file and the loop could not carry it back.
    Held,
    /// The document cannot reproduce this file yet (an unrecorded cell), so
    /// what is on disk was left alone.
    Kept,
}

impl DivergedKind {
    pub fn as_str(self) -> &'static str {
        match self {
            DivergedKind::Held => "held",
            DivergedKind::Kept => "kept",
        }
    }
}

impl WovenState {
    /// Snapshot engine-owned paths before a coordinated filesystem act.
    pub fn snapshot(&self) -> HashMap<PathBuf, Vec<u8>> {
        self.docs
            .keys()
            .chain(self.outputs.keys())
            .filter_map(|p| std::fs::read(p).ok().map(|bytes| (p.clone(), bytes)))
            .collect()
    }

    /// An engine-owned CLI/run write is not a reverse edit. Re-adopt its
    /// output bytes, and invalidate documents so the watcher refreshes lineage.
    pub fn adopt_changed(&mut self, before: &HashMap<PathBuf, Vec<u8>>) -> Vec<PathBuf> {
        let mut dirty = HashSet::new();
        for (path, old) in before {
            if let Ok(bytes) = std::fs::read(path) {
                if &bytes != old {
                    if let Some(output) = self.outputs.get_mut(path) {
                        if let Ok(text) = String::from_utf8(bytes) {
                            output.content = text;
                            dirty.insert(output.doc.clone());
                        }
                    } else {
                        dirty.insert(path.clone());
                    }
                }
            }
        }
        let dirty: Vec<_> = dirty.into_iter().collect();
        for doc in &dirty { self.docs.remove(doc); }
        dirty
    }
    pub fn output(&self, path: &Path) -> Option<&OutputState> {
        self.outputs.get(path)
    }

    pub fn doc_source(&self, path: &Path) -> Option<&String> {
        self.docs.get(path)
    }

    pub fn record_doc(&mut self, path: PathBuf, source: String) {
        self.docs.insert(path, source);
    }

    /// The documents currently being watched, in a stable order.
    pub fn doc_paths(&self) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = self.docs.keys().cloned().collect();
        paths.sort();
        paths
    }

    /// The output files currently being watched, in a stable order.
    pub fn output_paths(&self) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = self.outputs.keys().cloned().collect();
        paths.sort();
        paths
    }

    /// Forget any output of `doc` that the latest weave no longer produces.
    ///
    /// Called *after* writing, not before: `write_output` compares against the
    /// entry from the previous weave to decide whether the file on disk holds
    /// an edit nobody has consumed yet, and clearing the entries first would
    /// throw that comparison away. A `<hick:file>` deleted from the document
    /// stops being watched here.
    pub fn retain_outputs_of(&mut self, doc: &Path, produced: &HashSet<PathBuf>) {
        self.outputs
            .retain(|path, state| state.doc != doc || produced.contains(path));
    }

    /// Write an output file and remember the bytes, unless those bytes are
    /// already what is on disk.
    ///
    /// Skipping the identical write is not an optimization: rewriting a file
    /// an editor has open makes it report an external modification and, in
    /// some editors, discard the undo history. The loop must be invisible
    /// when it has nothing to say.
    pub fn write_output(&mut self, root: &Path, path: &Path, state: OutputState) -> Result<bool> {
        let on_disk = std::fs::read_to_string(path).ok();

        // A held file stays held until the document produces exactly what
        // is on disk. Overwriting it here would be the same destruction the
        // hold exists to prevent, one weave later. A KEPT file is different:
        // the document could not produce it before and can now, so the keep
        // ends and the write goes ahead.
        if let Some(diverged) = self.held.get_mut(path) {
            if diverged.kind == DivergedKind::Kept
                || on_disk.as_deref() == Some(state.content.as_str())
            {
                self.held.remove(path);
            } else {
                // What the document produces now, so a merge has its
                // third version.
                diverged.theirs = state.content.clone();
                self.outputs.insert(path.to_path_buf(), state);
                return Ok(false);
            }
        }

        // Never overwrite an edit we have not consumed yet.
        //
        // A weave can take seconds — a `--run` waits for real commands — and a
        // save that arrives while it is running is still sitting in the event
        // queue when it finishes. Writing the freshly woven bytes over it
        // would delete the user's typing *and* make the pending event look
        // like an echo of our own write, so the edit would vanish without a
        // trace. Leaving the file alone costs one extra weave: the pending
        // event is processed as the edit it is, lands in the document, and the
        // re-weave that follows reconciles both sides.
        if let (Some(on_disk), Some(previous)) = (on_disk.as_deref(), self.outputs.get(path))
            && on_disk != previous.content
            && on_disk != state.content
        {
            return Ok(false);
        }

        let unchanged = on_disk.is_some_and(|on_disk| on_disk == state.content);
        if !unchanged {
            // The SECOND output writer, and it needs the same stop as
            // `write_outputs_detailed`. A rule about what reaches disk that
            // is applied to only one of them is a rule with a hole in it —
            // and that gap is what once let a weave destroy committed
            // artifacts.
            crate::history::record(
                root,
                hickory_workspace::history::ActKind::Weave,
                Some(path.display().to_string()),
                &[(path.to_path_buf(), state.content.clone().into_bytes())],
            );
            write_atomic(path, &state.content)?;
        }
        set_read_only(path, !state.editable())?;
        self.outputs.insert(path.to_path_buf(), state);
        Ok(!unchanged)
    }

    /// Keep an output file as it is on disk, and remember why the loop is
    /// not touching it.
    pub fn hold_output(&mut self, path: &Path, reason: String) {
        let base = self
            .outputs
            .get(path)
            .map(|o| o.content.clone())
            .unwrap_or_default();
        self.held.insert(
            path.to_path_buf(),
            Diverged {
                kind: DivergedKind::Held,
                reason,
                theirs: base.clone(),
                base,
            },
        );
    }

    /// Record that a produced file was left as it is because the document
    /// cannot reproduce it yet.
    pub fn keep_output(&mut self, path: &Path, reason: String) {
        if self.held.contains_key(path) {
            return;
        }
        let on_disk = std::fs::read_to_string(path).unwrap_or_default();
        self.held.insert(
            path.to_path_buf(),
            Diverged {
                kind: DivergedKind::Kept,
                reason,
                base: on_disk,
                theirs: String::new(),
            },
        );
    }

    /// Every diverged output, with its reason and the versions a merge needs.
    pub fn held(&self) -> &HashMap<PathBuf, Diverged> {
        &self.held
    }

    /// Write bytes a person chose — a merge's result — over a diverged file.
    /// The next event on the file is then an ordinary save, carried back
    /// where it can be and held where it cannot.
    pub fn resolve_output(&mut self, path: &Path, content: &str) -> Result<()> {
        self.held.remove(path);
        set_read_only(path, false)?;
        write_atomic(path, content)?;
        Ok(())
    }

    /// Put an output file back to the bytes the document produces — the
    /// explicit way out of a hold, asked for by name, never done on the
    /// loop's own initiative. The recorded content is unchanged, so the
    /// write this causes is recognised as an echo and consumes itself.
    pub fn restore_output(&mut self, path: &Path) -> Result<()> {
        self.held.remove(path);
        let Some(state) = self.outputs.get(path) else {
            return Ok(());
        };
        // Clear read-only first: a fully-generated file is 0444 on disk, and
        // the restore has to be able to write through that.
        set_read_only(path, false)?;
        write_atomic(path, &state.content)?;
        set_read_only(path, !state.editable())?;
        Ok(())
    }

    /// Make every output writable again.
    ///
    /// The read-only marking is a live signal from a running loop, not a
    /// property of the files themselves. Leaving it behind would mean `hick
    /// run`, `hick test`, and anything else that writes these files fails
    /// with a permission error once the loop has exited — so the loop clears
    /// it on the way out, however it exits.
    pub fn release_read_only(&self) {
        for path in self.outputs.keys() {
            let _ = set_read_only(path, false);
        }
    }

    /// Whether a filesystem event on `path` is an echo of our own write.
    ///
    /// True when the file's current bytes are exactly what we last wrote
    /// there — which covers both the write itself and the restore that
    /// follows a refused edit.
    pub fn is_echo(&self, path: &Path) -> bool {
        let Some(state) = self.outputs.get(path) else {
            return false;
        };
        match std::fs::read_to_string(path) {
            Ok(on_disk) => on_disk == state.content,
            Err(_) => false,
        }
    }
}

/// Replace a file's contents without ever leaving a partial file visible.
///
/// Write to a sibling temporary file and rename over the target: an editor
/// or another tool reading concurrently sees either the old bytes or the new
/// ones, never half of each. The temporary lives in the same directory
/// because `rename` is only atomic within a filesystem.
pub fn write_atomic(path: &Path, content: &str) -> Result<()> {
    let parent = path.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent)
        .with_context(|| format!("failed to create {}", parent.display()))?;

    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string());
    let tmp = parent.join(format!(".{file_name}.hick-up.tmp"));

    std::fs::write(&tmp, content).with_context(|| format!("failed to write {}", tmp.display()))?;
    match std::fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(e) => {
            // Leaving a stray dotfile behind would be worse than the error.
            let _ = std::fs::remove_file(&tmp);
            Err(e).with_context(|| format!("failed to replace {}", path.display()))
        }
    }
}

/// Mark a file read-only, or writable again.
///
/// This is advisory, and deliberately so: it is a signal editors already
/// know how to show (vim's `[readonly]`, a padlock in VS Code) at the moment
/// the file is opened, rather than a refusal after the user has typed. It
/// does not enforce anything — `map_edits` does that on save.
///
/// On Unix this toggles the write bits and leaves every other bit of the
/// mode alone. `Permissions::set_readonly(false)` would instead make the file
/// world-writable, which is not a thing a tool should do to a file in
/// someone's working tree.
pub fn set_read_only(path: &Path, read_only: bool) -> Result<()> {
    let Ok(meta) = std::fs::metadata(path) else {
        return Ok(());
    };
    let perms = meta.permissions();
    if perms.readonly() == read_only {
        return Ok(());
    }
    let updated = toggled(perms, read_only);
    std::fs::set_permissions(path, updated)
        .with_context(|| format!("failed to set permissions on {}", path.display()))
}

#[cfg(unix)]
fn toggled(perms: std::fs::Permissions, read_only: bool) -> std::fs::Permissions {
    use std::os::unix::fs::PermissionsExt;
    let mode = perms.mode();
    // Strip every write bit to lock, restore the owner's write bit to unlock.
    // Group and other write are not restored: a file this tool locked and
    // unlocked should not come back more permissive than a fresh `0644`.
    let updated = if read_only {
        mode & !0o222
    } else {
        mode | 0o200
    };
    std::fs::Permissions::from_mode(updated)
}

#[cfg(not(unix))]
fn toggled(mut perms: std::fs::Permissions, read_only: bool) -> std::fs::Permissions {
    perms.set_readonly(read_only);
    perms
}
