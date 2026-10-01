//! Where the app remembers what you had open, and what you had not saved yet.
//!
//! Two kinds of thing live here, and they share a home for one reason:
//!
//!  - **UI state** — which tabs are open in which panes, where each tab's
//!    prose measure sits. Losing it is an annoyance.
//!  - **Drafts** — the contents of buffers with unsaved changes, written down
//!    when the app closes so reopening it puts the reader back exactly where
//!    they were, mid-sentence. Losing one is losing work.
//!
//! ## Why not in the project folder
//!
//! Because git would eventually commit it. Not through malice — through a
//! `git add -A` on a machine where `hick init` never ran, or a project opened
//! straight from a clone, or a `.gitignore` someone rewrote. A draft is
//! unfinished work that its author has not decided to keep; a UI layout is
//! one person's window arrangement. Neither belongs in anybody's history, and
//! "we wrote a gitignore entry" is a promise that a tool cannot keep on
//! somebody else's machine.
//!
//! So this writes under the platform's own per-user state directory, keyed by
//! the project's canonical path. Git cannot reach it by construction, which
//! needs no discipline from anyone.
//!
//! ## Why the base bytes are recorded with every draft
//!
//! A draft is one side of a merge waiting to happen. The file on disk may
//! have moved on while the app was closed — another branch checked out, a
//! `git pull`, an edit in another editor. Recording what the file looked like
//! when the draft was taken turns that from a two-way "these differ, you
//! sort it out" into a three-way merge, which resolves every region only one
//! side touched and asks about nothing else. See `apps/web/src/lib/merge.ts`.

pub mod history;

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

/// A UI blob larger than this is not a window layout, it is a bug or an
/// attempt to use this as a database. Refused with a message rather than
/// written, so the failure is visible now instead of as a slow startup later.
pub const MAX_UI_BYTES: usize = 1024 * 1024;

/// A draft larger than this is refused for the same reason. Generous: it is
/// roughly a 20MB document, far past anything anyone edits by hand.
pub const MAX_DRAFT_BYTES: usize = 20 * 1024 * 1024;

/// One buffer's unsaved contents, and what it was unsaved *from*.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Draft {
    /// Project-relative path of the file being edited.
    pub path: String,
    /// The buffer as the reader left it.
    pub contents: String,
    /// The file's contents when this editing session started — the common
    /// ancestor a three-way merge needs. Empty for a buffer that had no file
    /// behind it yet.
    pub base: String,
    /// Milliseconds since the epoch, as the app reported them. Advisory: it
    /// orders drafts for display and nothing depends on it being accurate.
    pub saved_at: u64,
}

/// The continuity switch, as it sits on disk.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct ContinuitySetting {
    enabled: bool,
}

/// Per-project state, under the user's own data directory.
#[derive(Debug, Clone)]
pub struct WorkspaceStore {
    dir: PathBuf,
}

impl WorkspaceStore {
    /// The store for one project root.
    ///
    /// The key is a readable slug plus a hash of the canonical path: the slug
    /// so a person poking around their data directory can tell which project
    /// a folder belongs to, and the hash so two projects called `notes` do not
    /// share one.
    pub fn for_project(root: &Path) -> Result<Self> {
        Self::under(&data_root()?, root)
    }

    /// The same, under an explicit base — what the tests use, and the seam
    /// that keeps this crate from needing a real home directory to be tested.
    pub fn under(base: &Path, root: &Path) -> Result<Self> {
        let canonical = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        let dir = base.join("workspaces").join(project_key(&canonical));
        fs::create_dir_all(dir.join("drafts"))
            .with_context(|| format!("could not create {}", dir.display()))?;
        Ok(Self { dir })
    }

    /// Where this store writes. Named in error messages, so a person can go
    /// and look.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// A window's recovery state is separate from every other window's.
    /// Slot zero retains the existing layout and drafts on first upgrade.
    pub fn window(mut self, slot: &str) -> Result<Self> {
        if slot != "0" {
            if slot.is_empty() || !slot.bytes().all(|b| b.is_ascii_digit()) {
                bail!("invalid window slot");
            }
            self.dir = self.dir.join("windows").join(slot);
            fs::create_dir_all(self.dir.join("drafts"))?;
        }
        Ok(self)
    }

    // -- UI state ----------------------------------------------------------

    fn ui_path(&self) -> PathBuf {
        self.dir.join("workspace.json")
    }

    /// The stored UI state, or `None` when there is none or it is unreadable.
    ///
    /// Deliberately opaque: which tabs are open in which panes is a shape the
    /// UI owns, and mirroring it in Rust would be two definitions to keep in
    /// step for no benefit. What this crate owns is that it is stored
    /// somewhere safe, atomically, per project.
    ///
    /// A corrupt file reads as `None` rather than as an error. The worst case
    /// is a window that opens with default tabs; refusing to start because a
    /// layout file is damaged would be a far worse trade.
    /// Whether CONTINUITY is on for this project, for this user.
    ///
    /// Continuity is the fourth provenance family — *what was this before* —
    /// and it is **off by default**, with the whole of it on this one switch:
    /// no ribbon, no journal, no pre-commit repair. An earlier design made
    /// the overlay opt-in and the bookkeeping mandatory, which was incoherent:
    /// it taxed every commit to feed a feature most people never turn on, and
    /// taxed hardest the workflow this product is built around.
    ///
    /// It lives here rather than in a `HICKORY_` variable because it is a
    /// preference and not machine configuration — and here rather than in the
    /// browser, because the things that WRITE a journal are the server, the
    /// merge driver and the pre-commit hook, none of which can read
    /// `localStorage`. Keyed by the project's canonical path, like everything
    /// else in this store.
    ///
    /// A missing or unreadable file reads as OFF. A preference nobody has
    /// expressed is not a reason to start writing records into their
    /// repository.
    pub fn continuity(&self) -> bool {
        let path = self.dir.join("continuity.json");
        let Ok(raw) = fs::read_to_string(&path) else {
            return false;
        };
        serde_json::from_str::<ContinuitySetting>(&raw)
            .map(|s| s.enabled)
            .unwrap_or(false)
    }

    /// Turn continuity on or off for this project, for this user.
    pub fn set_continuity(&self, enabled: bool) -> Result<()> {
        fs::create_dir_all(&self.dir)
            .with_context(|| format!("creating {}", self.dir.display()))?;
        let path = self.dir.join("continuity.json");
        let body = serde_json::to_string_pretty(&ContinuitySetting { enabled })
            .context("encoding the continuity setting")?;
        write_atomically(&path, body.as_bytes())
    }

    pub fn load_ui(&self) -> Option<serde_json::Value> {
        let text = fs::read_to_string(self.ui_path()).ok()?;
        serde_json::from_str(&text).ok()
    }

    /// Replace the stored UI state.
    pub fn save_ui(&self, state: &serde_json::Value) -> Result<()> {
        let text = serde_json::to_string(state).context("could not encode the UI state")?;
        if text.len() > MAX_UI_BYTES {
            bail!(
                "this UI state is {} bytes, past the {MAX_UI_BYTES}-byte limit.\n  \
                 A window layout is a few kilobytes; something is putting document \
                 contents in it. Drafts belong in the draft store, not here.",
                text.len()
            );
        }
        write_atomically(&self.ui_path(), text.as_bytes())
    }

    /// Forget the stored UI state.
    pub fn clear_ui(&self) -> Result<()> {
        remove_if_present(&self.ui_path())
    }

    // -- Drafts ------------------------------------------------------------

    fn drafts_dir(&self) -> PathBuf {
        self.dir.join("drafts")
    }

    fn draft_path(&self, path: &str) -> PathBuf {
        // Hashed rather than sanitised: a project-relative path contains
        // separators, and every scheme for flattening them into a filename
        // either collides or is unreadable anyway. The real path is inside
        // the file.
        self.drafts_dir()
            .join(format!("{}.json", hex(path.as_bytes())))
    }

    /// Every draft this project is holding, oldest first.
    ///
    /// Unreadable entries are skipped rather than failing the call: one
    /// damaged draft must not hide the other nine.
    pub fn list_drafts(&self) -> Vec<Draft> {
        let mut drafts: Vec<Draft> = match fs::read_dir(self.drafts_dir()) {
            Ok(entries) => entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().extension().is_some_and(|e| e == "json"))
                .filter_map(|entry| fs::read_to_string(entry.path()).ok())
                .filter_map(|text| serde_json::from_str::<Draft>(&text).ok())
                .collect(),
            Err(_) => Vec::new(),
        };
        drafts.sort_by(|a, b| {
            a.saved_at
                .cmp(&b.saved_at)
                .then_with(|| a.path.cmp(&b.path))
        });
        drafts
    }

    /// One draft, by the path it is for.
    pub fn load_draft(&self, path: &str) -> Option<Draft> {
        let text = fs::read_to_string(self.draft_path(path)).ok()?;
        serde_json::from_str(&text).ok()
    }

    /// Write (or replace) the draft for `path`.
    pub fn save_draft(&self, draft: &Draft) -> Result<()> {
        if draft.path.is_empty() {
            bail!("a draft needs the path of the file it is for");
        }
        if draft.contents.len() > MAX_DRAFT_BYTES {
            bail!(
                "this buffer is {} bytes, past the {MAX_DRAFT_BYTES}-byte draft limit.\n  \
                 Save the file itself; a buffer this size is not something to hold \
                 unsaved across a restart.",
                draft.contents.len()
            );
        }
        let text = serde_json::to_string(draft).context("could not encode the draft")?;
        write_atomically(&self.draft_path(&draft.path), text.as_bytes())
    }

    /// Forget the draft for `path`. Not an error when there is none — the
    /// caller discards on every save, and most saves have nothing to discard.
    pub fn discard_draft(&self, path: &str) -> Result<()> {
        remove_if_present(&self.draft_path(path))
    }
}

/// Override for where this crate writes. One knob, named the way every other
/// knob in this product is (`.instructions/config-and-environments.md`).
///
/// It exists because two different people need it: someone running the tool
/// from a USB stick who wants their state to travel with it, and the test
/// suite, which must never write into the developer's real home directory to
/// prove that a draft round-trips.
pub const STATE_DIR_VAR: &str = "HICKORY_STATE_DIR";

/// The platform's per-user state directory, with our folder under it.
///
/// State rather than config, deliberately: a window layout and a pile of
/// unsaved drafts are things the app accumulated, not things the user
/// configured, and the platforms that distinguish the two put them in
/// different places.
pub fn data_root() -> Result<PathBuf> {
    if let Some(override_dir) = std::env::var_os(STATE_DIR_VAR).filter(|v| !v.is_empty()) {
        return Ok(PathBuf::from(override_dir));
    }
    let base = dirs::data_dir().context(
        "could not find this platform's application-data directory, so there is \
         nowhere to remember open tabs and unsaved drafts.\n  \
         On Linux this is $XDG_DATA_HOME or ~/.local/share; on macOS \
         ~/Library/Application Support; on Windows %APPDATA%.\n  \
         The app still works — it will just forget the window layout between runs.\n  \
         Set HICKORY_STATE_DIR to name a directory yourself.",
    )?;
    Ok(base.join("hickory"))
}

/// The per-user state directory, for callers that keep state which is NOT
/// per project — machine identity and the fleet's key list, which belong to
/// the machine and not to any folder it has open.
pub fn state_root() -> Result<PathBuf> {
    let root = data_root()?;
    fs::create_dir_all(&root).with_context(|| format!("could not create {}", root.display()))?;
    Ok(root)
}

/// A readable slug plus a hash, so a person can recognise the folder and two
/// projects with the same name still get their own.
fn project_key(canonical: &Path) -> String {
    let slug: String = canonical
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "project".to_string())
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .take(40)
        .collect();
    let digest = hex(canonical.to_string_lossy().as_bytes());
    let slug = if slug.is_empty() { "project" } else { &slug };
    format!("{slug}-{}", &digest[..16])
}

fn hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Write through a temp file and rename.
///
/// The whole point of this store is surviving the app closing, which includes
/// the app closing badly. A half-written draft is worse than no draft: it
/// would restore as truncated text that looks like the reader's own work.
fn write_atomically(path: &Path, bytes: &[u8]) -> Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(dir).with_context(|| format!("could not create {}", dir.display()))?;
    let tmp = path.with_extension("partial");
    {
        let mut file =
            fs::File::create(&tmp).with_context(|| format!("could not write {}", tmp.display()))?;
        file.write_all(bytes)?;
        // Without this the rename can land before the bytes do, which on a
        // power loss leaves an empty file where a draft should be.
        file.sync_all()?;
    }
    fs::rename(&tmp, path)
        .with_context(|| format!("could not move {} into place", tmp.display()))?;
    Ok(())
}

fn remove_if_present(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e).with_context(|| format!("could not remove {}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(base: &Path, project: &Path) -> WorkspaceStore {
        WorkspaceStore::under(base, project).expect("the store opens")
    }

    fn fixture() -> (tempfile::TempDir, tempfile::TempDir) {
        (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap())
    }

    #[test]
    fn nothing_is_written_inside_the_project() {
        // The whole reason this crate exists. A draft is unfinished work and a
        // layout is one person's window arrangement; neither may end up in
        // anybody's git history, and a `.gitignore` entry is a promise this
        // tool cannot keep on somebody else's machine.
        let (base, project) = fixture();
        let s = store(base.path(), project.path());
        s.save_ui(&serde_json::json!({ "tabs": ["a.hick"] }))
            .unwrap();
        s.save_draft(&Draft {
            path: "notes.hick".into(),
            contents: "unsaved".into(),
            base: "".into(),
            saved_at: 1,
        })
        .unwrap();

        let inside: Vec<_> = fs::read_dir(project.path()).unwrap().collect();
        assert!(
            inside.is_empty(),
            "the project folder must be untouched, found {inside:?}"
        );
        assert!(s.dir().starts_with(base.path()));
    }

    #[test]
    fn the_state_directory_can_be_named_rather_than_discovered() {
        // The seam the test suite uses, and the one a person running from a
        // USB stick uses. Asserted through `for_project`, because the point is
        // that the DEFAULT path is what the variable replaces.
        let base = tempfile::tempdir().unwrap();
        let project = tempfile::tempdir().unwrap();
        // SAFETY: single-threaded test; the variable is read once, below.
        unsafe { std::env::set_var(STATE_DIR_VAR, base.path()) };
        let s = WorkspaceStore::for_project(project.path()).expect("the store opens");
        assert!(s.dir().starts_with(base.path()), "{}", s.dir().display());
        unsafe { std::env::remove_var(STATE_DIR_VAR) };
    }

    #[test]
    fn ui_state_round_trips() {
        let (base, project) = fixture();
        let s = store(base.path(), project.path());
        assert!(s.load_ui().is_none(), "nothing stored yet");
        let state = serde_json::json!({ "panes": [{ "tabs": ["a"], "wrap": 72 }] });
        s.save_ui(&state).unwrap();
        assert_eq!(s.load_ui(), Some(state));
        s.clear_ui().unwrap();
        assert!(s.load_ui().is_none());
    }

    #[test]
    fn a_damaged_layout_opens_default_tabs_rather_than_refusing_to_start() {
        let (base, project) = fixture();
        let s = store(base.path(), project.path());
        s.save_ui(&serde_json::json!({ "ok": true })).unwrap();
        fs::write(s.dir().join("workspace.json"), "{not json").unwrap();
        assert!(s.load_ui().is_none());
    }

    #[test]
    fn a_ui_blob_carrying_a_document_is_refused_with_a_reason() {
        let (base, project) = fixture();
        let s = store(base.path(), project.path());
        let huge = serde_json::json!({ "oops": "x".repeat(MAX_UI_BYTES + 1) });
        let err = s.save_ui(&huge).unwrap_err().to_string();
        assert!(err.contains("Drafts belong in the draft store"), "{err}");
    }

    #[test]
    fn a_draft_keeps_the_bytes_it_was_taken_from() {
        // Without the base there is no three-way merge, only a two-way
        // comparison that has to ask about every difference.
        let (base, project) = fixture();
        let s = store(base.path(), project.path());
        let draft = Draft {
            path: "notes/today.hick".into(),
            contents: "half a sentence".into(),
            base: "what was on disk".into(),
            saved_at: 42,
        };
        s.save_draft(&draft).unwrap();
        assert_eq!(s.load_draft("notes/today.hick"), Some(draft));
    }

    #[test]
    fn drafts_list_oldest_first_and_a_damaged_one_hides_nothing() {
        let (base, project) = fixture();
        let s = store(base.path(), project.path());
        for (path, at) in [("c.hick", 3u64), ("a.hick", 1), ("b.hick", 2)] {
            s.save_draft(&Draft {
                path: path.into(),
                contents: path.into(),
                base: String::new(),
                saved_at: at,
            })
            .unwrap();
        }
        fs::write(s.dir().join("drafts").join("garbage.json"), "{{{").unwrap();
        let paths: Vec<_> = s.list_drafts().into_iter().map(|d| d.path).collect();
        assert_eq!(paths, vec!["a.hick", "b.hick", "c.hick"]);
    }

    #[test]
    fn discarding_a_draft_that_is_not_there_is_not_an_error() {
        // The caller discards on every save, and most saves have nothing to
        // discard.
        let (base, project) = fixture();
        let s = store(base.path(), project.path());
        s.discard_draft("never-existed.hick").unwrap();
    }

    #[test]
    fn a_path_with_separators_does_not_collide_or_escape() {
        let (base, project) = fixture();
        let s = store(base.path(), project.path());
        for path in ["a/b.hick", "a-b.hick", "../outside.hick"] {
            s.save_draft(&Draft {
                path: path.into(),
                contents: path.into(),
                base: String::new(),
                saved_at: 1,
            })
            .unwrap();
        }
        assert_eq!(s.list_drafts().len(), 3);
        assert_eq!(s.load_draft("a/b.hick").unwrap().contents, "a/b.hick");
        // `..` in a document path must not write outside the draft folder.
        let stray = base.path().join("outside.hick.json");
        assert!(!stray.exists(), "a draft escaped its folder");
    }

    #[test]
    fn two_projects_with_the_same_name_do_not_share_a_store() {
        let base = tempfile::tempdir().unwrap();
        let parent = tempfile::tempdir().unwrap();
        let one = parent.path().join("one/notes");
        let two = parent.path().join("two/notes");
        fs::create_dir_all(&one).unwrap();
        fs::create_dir_all(&two).unwrap();
        let a = store(base.path(), &one);
        let b = store(base.path(), &two);
        assert_ne!(a.dir(), b.dir());
        // ...and both are still recognisable to a person looking at them.
        assert!(
            a.dir()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("notes-")
        );
    }

    #[test]
    fn a_partial_write_never_replaces_a_good_draft() {
        // The store exists to survive the app closing, including closing
        // badly; a truncated draft would restore as text that looks like the
        // reader's own work.
        let (base, project) = fixture();
        let s = store(base.path(), project.path());
        let good = Draft {
            path: "a.hick".into(),
            contents: "the good one".into(),
            base: String::new(),
            saved_at: 1,
        };
        s.save_draft(&good).unwrap();
        // A leftover temp file from a crashed write is ignored entirely.
        fs::write(s.dir().join("drafts").join("stale.partial"), "half").unwrap();
        assert_eq!(s.load_draft("a.hick"), Some(good));
        assert_eq!(s.list_drafts().len(), 1);
    }
}
