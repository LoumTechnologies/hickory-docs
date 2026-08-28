//! `.hick` merges go through one path, and a clone can tell whether they do.
//!
//! See `docs/specs/freeform/provenance-across-versions.md`. A merge is the
//! richest recording site there is — the one moment when **all three versions
//! are in hand**, base, ours and theirs — richer than a watched edit (one
//! before, one after) and richer than a pre-commit repair (two sides).
//! Merging through the tool is therefore not merely a way to avoid punching a
//! hole in continuity; it is where the strongest correspondence available
//! anywhere in the system would be produced.
//!
//! **Make it automatic rather than a team rule.** `hick init` writes
//! `*.hick merge=hick` into `.gitattributes` and defines the driver, so
//! `git merge` on the command line runs the same code the app's merge tab
//! will — one path, no discipline required.
//!
//! One sharp edge, of the kind this product refuses elsewhere: **the driver
//! definition lives in `.git/config`, not in the repository**, because it is
//! an executable command and git will not let a clone hand you one. So every
//! clone must run `hick init`, and **an undefined driver makes git silently
//! fall back to the default line merge** — no warning, no marker, nothing to
//! notice. That is why [`status`] exists and why it is checked at project
//! open and in `hick test` rather than in the pre-commit hook: `hick init`
//! installs the hook, so a clone that never ran it has neither the driver nor
//! the thing that would report the driver missing — which is precisely the
//! clone the check exists for. Checking at commit time would also tell you
//! after the damage.
//!
//! **What the driver does today is a three-way merge of the document text,
//! and no more.** The document-aware merge and the correspondence it would
//! record are the next steps in that design; what this step buys is that
//! every `.hick` merge is OURS, that a merged document is checked for
//! parseability before it is accepted as clean, and that a repository can
//! answer whether any of it is wired up. Claiming more would be claiming the
//! recording site is recording.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context as _, Result};
use serde::Serialize;

/// The driver's name in `.gitattributes` and `.git/config`.
pub const DRIVER: &str = "hick";

/// The line `hick init` writes into `.gitattributes`.
pub const ATTRIBUTES_LINE: &str = "*.hick merge=hick";

/// The driver for files a document GENERATES, which is a different problem
/// from merging a document.
///
/// A generated file is a function of its inputs, so it has no merge of its
/// own: the merge happens in the document, and the output follows. Left to
/// git, one edit to one record produced three conflicts in the warehouse
/// example — the document, its woven markdown, and the C# file the document
/// writes — carrying the same two lines each time. Two of those are not a
/// person's to resolve: hand-editing generated text is the act this product
/// refuses everywhere else, and a resolution typed there is discarded by the
/// next `hick run` without saying so.
///
/// So this driver **does not merge**. It keeps ours, exits clean, and says
/// the file is generated. What makes that safe rather than lossy is the drift
/// gate that already exists: `hick test` compares every generated file
/// against what its document produces, so a merge that left the wrong bytes
/// there cannot reach a commit unnoticed. Take either side, re-run, and the
/// check confirms it.
pub const GENERATED_DRIVER: &str = "hick-generated";

/// What every generated path is marked with.
///
/// `linguist-generated=true` is the convention GitHub reads: the file is
/// collapsed in a pull request and left out of the repository's language
/// statistics. `-diff` is deliberately NOT set — the diff of a generated file
/// is worth reading when you are checking that a generator did what you
/// meant, which is the whole review model this product is built around.
pub const GENERATED_ATTRS: &str = "linguist-generated=true merge=hick-generated";

/// Markers around the managed list of generated paths in `.gitattributes`.
const GENERATED_BEGIN: &str = "# BEGIN HICKORY GENERATED OUTPUTS (managed by `hick init`)";
const GENERATED_END: &str = "# END HICKORY GENERATED OUTPUTS";

/// Whether this repository actually merges `.hick` documents through hick.
#[derive(Debug, Clone, Serialize)]
pub struct MergeDriverStatus {
    /// False when the directory is not a git work tree at all.
    pub repository: bool,
    /// `.gitattributes` routes `*.hick` at this driver. Committed, so a clone
    /// has it.
    pub attributes: bool,
    /// `merge.hick.driver` is defined in this clone's config. NOT committed —
    /// git will not let a repository hand a clone an executable command — so
    /// this is the half that a fresh clone is missing.
    pub configured: bool,
    /// One sentence naming what is missing and what to do, or confirming.
    pub summary: String,
}

impl MergeDriverStatus {
    /// Whether `.hick` merges are going through hick right now.
    pub fn ok(&self) -> bool {
        !self.repository || (self.attributes && self.configured)
    }
}

fn git(dir: &Path, args: &[&str]) -> Option<std::process::Output> {
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .ok()
}

/// Read whether the driver is wired up, for `dir`.
pub fn status(dir: &Path) -> MergeDriverStatus {
    let inside = git(dir, &["rev-parse", "--is-inside-work-tree"])
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !inside {
        return MergeDriverStatus {
            repository: false,
            attributes: false,
            configured: false,
            summary: "Not a git repository, so there are no merges to route.".to_string(),
        };
    }

    // Ask git what it would actually DO with a `.hick` path, rather than
    // reading `.gitattributes` ourselves: attributes can come from several
    // files and from `info/attributes`, and the question is what git resolves.
    let attributes = git(dir, &["check-attr", "merge", "--", "a.hick"])
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .ends_with(&format!(": {DRIVER}"))
        })
        .unwrap_or(false);

    let configured = git(dir, &["config", "--get", &format!("merge.{DRIVER}.driver")])
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false);

    let summary = match (attributes, configured) {
        (true, true) => format!(
            "`.hick` documents merge through hick: `{ATTRIBUTES_LINE}` is in \
             .gitattributes and `merge.{DRIVER}.driver` is defined in this clone."
        ),
        (true, false) => format!(
            "`.gitattributes` routes `*.hick` at the `{DRIVER}` merge driver, but \
             this clone has not defined it — so git is SILENTLY falling back to \
             its line merge, with no warning and nothing to notice afterwards.\n  \
             The definition is an executable command, which git will not let a \
             repository hand a clone, so it cannot be committed.\n  \
             Next step: run `hick init` in this repository."
        ),
        (false, true) => format!(
            "This clone defines the `{DRIVER}` merge driver, but nothing routes \
             `*.hick` at it.\n  \
             Next step: run `hick init`, which writes `{ATTRIBUTES_LINE}` into \
             .gitattributes — that half IS committed, so it reaches everyone."
        ),
        (false, false) => format!(
            "`.hick` documents merge with git's line merge, not hick's.\n  \
             Next step: run `hick init` in this repository. It writes \
             `{ATTRIBUTES_LINE}` into .gitattributes and defines the driver in \
             this clone; every clone has to run it, because the definition is an \
             executable command git will not carry."
        ),
    };

    MergeDriverStatus {
        repository: true,
        attributes,
        configured,
        summary,
    }
}

// ---------------------------------------------------------------------------
// Installation
// ---------------------------------------------------------------------------

/// Append `*.hick merge=hick` to `.gitattributes` if it is not there.
/// Returns true when the file changed.
pub fn ensure_attributes(path: &Path) -> Result<bool> {
    let existing = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e).with_context(|| format!("failed to read {}", path.display())),
    };
    if existing.lines().any(|l| l.trim() == ATTRIBUTES_LINE) {
        return Ok(false);
    }
    let mut next = existing;
    if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
    }
    next.push_str(ATTRIBUTES_LINE);
    next.push('\n');
    std::fs::write(path, next).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(true)
}

/// Rewrite the managed block of generated paths in `.gitattributes`.
///
/// The list is derived, not remembered: every `.hick` document in the tree is
/// read for what it declares it writes — its `weave` target, each
/// `hick:file path=`, and each `hick:volume output=` directory, which becomes
/// a `dir/**` pattern because a volume's contents are named by the program
/// that wrote them rather than by the document.
///
/// A block that has gone stale degrades safely: a generated file added since
/// the last `hick init` merges the way it used to, which is the way
/// everything did before this existed. That is the same bargain
/// `declared_outputs` already makes by reading attributes instead of weaving.
pub fn ensure_generated_attributes(path: &Path, paths: &[String]) -> Result<bool> {
    let existing = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e).with_context(|| format!("failed to read {}", path.display()))?,
    };

    let mut block = String::new();
    if !paths.is_empty() {
        block.push_str(GENERATED_BEGIN);
        block.push('\n');
        for pattern in paths {
            // A pattern with whitespace has to be quoted, which is the one
            // piece of gitattributes syntax a generated path routinely needs.
            if pattern.contains(char::is_whitespace) {
                block.push_str(&format!("\"{pattern}\" {GENERATED_ATTRS}\n"));
            } else {
                block.push_str(&format!("{pattern} {GENERATED_ATTRS}\n"));
            }
        }
        block.push_str(GENERATED_END);
        block.push('\n');
    }

    let next = replace_block(&existing, &block);
    if next == existing {
        return Ok(false);
    }
    std::fs::write(path, next).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(true)
}

/// Swap the managed block for `block`, or append it when there is none.
fn replace_block(existing: &str, block: &str) -> String {
    let mut out = String::new();
    let mut skipping = false;
    let mut replaced = false;
    for line in existing.lines() {
        if line.trim() == GENERATED_BEGIN {
            skipping = true;
            out.push_str(block);
            replaced = true;
            continue;
        }
        if skipping {
            if line.trim() == GENERATED_END {
                skipping = false;
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    if !replaced && !block.is_empty() {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(block);
    }
    out
}

/// Define `merge.hick-generated.*` in this clone's config.
///
/// Separate from the document driver for the same reason it is a separate
/// driver: this one is told not to merge.
pub fn ensure_generated_driver_config(root: &Path) -> Result<bool> {
    let exe = current_exe_path();
    let command = format!("{exe} merge-generated --path %P");
    ensure_config(
        root,
        &[
            (
                format!("merge.{GENERATED_DRIVER}.name"),
                "hick generated output (kept, then regenerated)".to_string(),
            ),
            (format!("merge.{GENERATED_DRIVER}.driver"), command),
        ],
    )
}

/// This binary's path, for a command git will run with git's own `PATH`.
fn current_exe_path() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(str::to_string))
        .unwrap_or_else(|| "hick".to_string())
}

/// Set each key that is not already the wanted value. True when any changed.
fn ensure_config(root: &Path, want: &[(String, String)]) -> Result<bool> {
    let mut changed = false;
    for (key, value) in want {
        let current = git(root, &["config", "--get", key])
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
        if current.as_deref() == Some(value.as_str()) {
            continue;
        }
        let out = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["config", key, value])
            .output()
            .context("failed to run git config")?;
        if !out.status.success() {
            anyhow::bail!(
                "could not define {key}: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        changed = true;
    }
    Ok(changed)
}

/// Define `merge.hick.*` in this clone's config. Returns true when it changed.
///
/// The command names this binary by the path it is running from, because a
/// merge git starts has whatever `PATH` git inherited, which is not
/// necessarily the one the person who ran `hick init` had.
pub fn ensure_driver_config(root: &Path) -> Result<bool> {
    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(str::to_string))
        .unwrap_or_else(|| "hick".to_string());
    // %O base, %A ours (and the file the result MUST be written to), %B
    // theirs, %L conflict-marker size, %P the path in the work tree.
    let command =
        format!("{exe} merge-driver --base %O --ours %A --theirs %B --marker-size %L --path %P");
    let want = [
        (
            format!("merge.{DRIVER}.name"),
            "hick document merge".to_string(),
        ),
        (format!("merge.{DRIVER}.driver"), command),
    ];
    let mut changed = false;
    for (key, value) in want {
        let current = git(root, &["config", "--get", &key])
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
        if current.as_deref() == Some(value.as_str()) {
            continue;
        }
        let out = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["config", &key, &value])
            .output()
            .context("failed to run git config")?;
        if !out.status.success() {
            anyhow::bail!(
                "could not define {key}: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        changed = true;
    }
    Ok(changed)
}

// ---------------------------------------------------------------------------
// The driver itself
// ---------------------------------------------------------------------------

/// What one merge produced.
pub enum MergeOutcome {
    /// Merged cleanly; `ours` now holds the result.
    Clean,
    /// Conflicted; `ours` holds the result with conflict markers, which is
    /// git's contract for a driver that exits non-zero.
    Conflicted { reason: String },
}

/// Run the merge driver over the three files git handed us.
///
/// The result is always written to `ours`, which is the file git reads back —
/// on a clean merge and on a conflicted one alike.
pub fn run(
    base: &Path,
    ours: &Path,
    theirs: &Path,
    marker_size: usize,
    path: &str,
) -> Result<MergeOutcome> {
    // Three-way merge of the document text. `git merge-file` is the same
    // algorithm git would have used anyway, invoked deliberately rather than
    // fallen back into: the difference this step makes is that the result
    // passes through here, where the check below lives and where a recorded
    // correspondence will be written.
    // What `ours` held before git's algorithm rewrote it in place. The merge
    // driver is a writer whose way back today is `git merge --abort`, and
    // only if you have not moved since — so the bytes are kept here, keyed by
    // the repo-relative path the result will land at rather than by the temp
    // file git handed over.
    let before = std::fs::read(ours).ok();

    let marker = format!("--marker-size={}", marker_size.clamp(7, 80));
    let out = Command::new("git")
        .args([
            "merge-file",
            &marker,
            "-L",
            "ours",
            "-L",
            "base",
            "-L",
            "theirs",
        ])
        .arg(ours)
        .arg(base)
        .arg(theirs)
        .output()
        .context("failed to run `git merge-file`")?;
    let conflicts = match out.status.code() {
        Some(n) if n >= 0 => n,
        _ => {
            anyhow::bail!(
                "the three-way merge of {path} failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )
        }
    };
    if conflicts == 0
        && let Ok(root) = std::env::current_dir()
    {
        crate::history::record_changes(
            &root,
            hickory_workspace::history::ActKind::Merge,
            Some(format!("merged {path}")),
            &[(path.to_string(), before, std::fs::read(ours).ok())],
        );
    }
    if conflicts > 0 {
        return Ok(MergeOutcome::Conflicted {
            reason: format!(
                "{conflicts} conflicting region(s) in {path}. The markers are in \
                 the file; resolving them here is what makes the resolution part \
                 of the document rather than something a text editor did to it."
            ),
        });
    }

    // A clean line merge can still produce a document that does not parse —
    // two sides adding different elements around one another is the ordinary
    // way. Reporting that as CLEAN would hand back a `.hick` file nothing can
    // read, so it is reported as a conflict for a person to look at. This is
    // the one thing this driver does that git's fallback cannot.
    let merged = std::fs::read_to_string(ours)
        .with_context(|| format!("could not read the merged {path}"))?;
    if let Err(e) = hick_lang::parse(&merged) {
        return Ok(MergeOutcome::Conflicted {
            reason: format!(
                "{path} merged without overlapping edits, but the result is not a \
                 readable document ({e}).\n  \
                 Two sides can each be correct and still not compose — git's line \
                 merge has no way to know that, which is exactly why this driver \
                 exists. The merged text is in the file; nothing was silently \
                 accepted."
            ),
        });
    }

    Ok(MergeOutcome::Clean)
}

/// The path `hick init` writes attributes into.
pub fn attributes_path(root: &Path) -> PathBuf {
    root.join(".gitattributes")
}

#[cfg(test)]
mod generated_attribute_tests {
    use super::*;

    fn write(dir: &Path, body: &str) -> PathBuf {
        let path = dir.join(".gitattributes");
        std::fs::write(&path, body).unwrap();
        path
    }

    #[test]
    fn the_block_is_rewritten_in_place_and_leaves_the_rest_alone() {
        let dir = tempfile::tempdir().unwrap();
        let path = write(dir.path(), "*.hick merge=hick\n*.png binary\n");

        assert!(ensure_generated_attributes(&path, &["out.md".to_string()]).unwrap());
        let first = std::fs::read_to_string(&path).unwrap();
        assert!(first.starts_with("*.hick merge=hick\n*.png binary\n"));
        assert!(first.contains(&format!("out.md {GENERATED_ATTRS}")));

        // Re-running with the same list is a no-op, so `hick init` stays
        // idempotent and does not churn the file.
        assert!(!ensure_generated_attributes(&path, &["out.md".to_string()]).unwrap());

        // A changed list replaces the block rather than appending a second
        // one — the whole reason it is delimited.
        assert!(ensure_generated_attributes(&path, &["other.md".to_string()]).unwrap());
        let second = std::fs::read_to_string(&path).unwrap();
        assert_eq!(second.matches(GENERATED_BEGIN).count(), 1);
        assert!(second.contains("other.md"));
        assert!(!second.contains("out.md"));
        assert!(second.contains("*.png binary"));
    }

    #[test]
    fn an_empty_list_removes_the_block_entirely() {
        let dir = tempfile::tempdir().unwrap();
        let path = write(dir.path(), "*.hick merge=hick\n");
        ensure_generated_attributes(&path, &["out.md".to_string()]).unwrap();
        assert!(ensure_generated_attributes(&path, &[]).unwrap());
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(!body.contains(GENERATED_BEGIN));
        assert_eq!(body.trim(), "*.hick merge=hick");
    }

    #[test]
    fn a_path_with_a_space_is_quoted() {
        // gitattributes splits on whitespace, so an unquoted path with a
        // space silently matches nothing.
        let dir = tempfile::tempdir().unwrap();
        let path = write(dir.path(), "");
        ensure_generated_attributes(&path, &["my notes.md".to_string()]).unwrap();
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(
            body.contains(&format!("\"my notes.md\" {GENERATED_ATTRS}")),
            "{body}"
        );
    }
}
