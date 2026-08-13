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
}

impl WovenState {
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
    pub fn write_output(&mut self, path: &Path, state: OutputState) -> Result<bool> {
        let on_disk = std::fs::read_to_string(path).ok();

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
            write_atomic(path, &state.content)?;
        }
        set_read_only(path, !state.editable())?;
        self.outputs.insert(path.to_path_buf(), state);
        Ok(!unchanged)
    }

    /// Put an output file back to the bytes we last wrote, after refusing an
    /// edit. The recorded content is unchanged, so the write this causes is
    /// recognised as an echo and consumes itself.
    pub fn restore_output(&self, path: &Path) -> Result<()> {
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
