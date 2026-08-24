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
