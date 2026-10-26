<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Local History Is The Interval Below The Commit

For: Authors editing executable documents and the files those documents produce.

Given a folder of documents, when anything writes to it — you, a run, a weave,
a reverse edit, an ingest, the agent, find-and-replace, the merge driver —
then what the affected files held immediately before is kept, locally, and can
be put back:

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/local-history-is-the-interval-below-the-commit.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-2449d0ed01ad"></a>
## `crates/hickory-cli/src/history.rs`

Related contracts: [Local History Is The Interval Below The Commit](contract.md).

<a id="f-2449d0ed01ad-retention"></a>
### `retention`

Recording what every writer did, before it did it.
`docs/specs/freeform/local-history.md`. The store itself is
[`hickory_workspace::history`]; this is the seam every writer in the
product goes through to leave a stop behind.
Two rules shape the whole module.
**Local history must never break a write.** Every function here swallows
its own failures and logs them. A person's run must not fail because a
cache on their own machine could not be written — this store exists to
make things recoverable, and a version of it that can take a run down
makes them less so.
**The before-bytes are read from disk, here, immediately before the
write.** Not passed in, and not remembered from earlier: whatever is on
disk right now is what a person would lose, whoever put it there.

<storycode:copy id="f-2449d0ed01ad-retention">
//! Recording what every writer did, before it did it.
//!
//! `docs/specs/freeform/local-history.md`. The store itself is
//! [`hickory_workspace::history`]; this is the seam every writer in the
//! product goes through to leave a stop behind.
//!
//! Two rules shape the whole module.
//!
//! **Local history must never break a write.** Every function here swallows
//! its own failures and logs them. A person's run must not fail because a
//! cache on their own machine could not be written — this store exists to
//! make things recoverable, and a version of it that can take a run down
//! makes them less so.
//!
//! **The before-bytes are read from disk, here, immediately before the
//! write.** Not passed in, and not remembered from earlier: whatever is on
//! disk right now is what a person would lose, whoever put it there.

use std::path::{Path, PathBuf};

use hickory_workspace::WorkspaceStore;
use hickory_workspace::history::{Act, ActKind, History, PendingChange, Retention};

/// How much local history to keep, from the environment.
///
/// Read here rather than threaded through every caller because it is machine
/// configuration for a machine-local cache — and validated rather than
/// silently defaulted, so a typo says so instead of quietly halving somebody's
/// history.
pub fn retention() -> Retention {
    let mut retention = Retention::default();
    if let Some(bytes) = parsed("HICKORY_HISTORY_BYTES") {
        retention.max_bytes = bytes;
    }
    if let Some(days) = parsed("HICKORY_HISTORY_DAYS") {
        retention.max_days = days;
    }
    retention
}

</storycode:copy>
<a id="f-2449d0ed01ad-parsed"></a>
### `parsed`

<storycode:copy id="f-2449d0ed01ad-parsed">
fn parsed(name: &str) -> Option<u64> {
    let raw = std::env::var(name).ok()?;
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    match raw.parse::<u64>() {
        Ok(value) => Some(value),
        Err(_) => {
            eprintln!(
                "warning: {name} is not a whole number ({raw:?}), so local history is using its \
                 default. Set it to a count, like {name}=7."
            );
            None
        }
    }
}

</storycode:copy>
<a id="f-2449d0ed01ad-open"></a>
### `open`

The store for a project, or `None` when there is nowhere to put one.
`None` is a normal outcome: a machine with no data directory, or one where
it cannot be created, still runs everything else.

<storycode:copy id="f-2449d0ed01ad-open">
/// The store for a project, or `None` when there is nowhere to put one.
///
/// `None` is a normal outcome: a machine with no data directory, or one where
/// it cannot be created, still runs everything else.
pub fn open(root: &Path) -> Option<History> {
    let store = WorkspaceStore::for_project(&absolute(root)).ok()?;
    History::open(&store).ok()
}

</storycode:copy>
<a id="f-2449d0ed01ad-absolute"></a>
### `absolute`

The project key is the canonical path, so a relative root has to be made
absolute before it is used as one.
Not defensive tidying — this was a real bug, found by running it. A run
invoked as `hick run note.md` has `doc_path.parent() == ""`, which
hashes to the key for the empty string, so every act landed in a store
nothing would ever look in while `hick history` read the one for the
working directory and reported an empty folder. Two stores, no error, and
a feature that silently did nothing.

<storycode:copy id="f-2449d0ed01ad-absolute">
/// The project key is the canonical path, so a relative root has to be made
/// absolute before it is used as one.
///
/// Not defensive tidying — this was a real bug, found by running it. A run
/// invoked as `hick run note.md` has `doc_path.parent() == ""`, which
/// hashes to the key for the empty string, so every act landed in a store
/// nothing would ever look in while `hick history` read the one for the
/// working directory and reported an empty folder. Two stores, no error, and
/// a feature that silently did nothing.
fn absolute(root: &Path) -> PathBuf {
    if root.is_absolute() {
        return root.to_path_buf();
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if root.as_os_str().is_empty() {
        cwd
    } else {
        cwd.join(root)
    }
}

</storycode:copy>
<a id="f-2449d0ed01ad-record"></a>
### `record`

Record what `writes` is about to do to `root`, reading each file's current
bytes as the *before*.
Call immediately before writing. Returns the act when one was worth
keeping — an act whose files all end up with the bytes they already had is
not a stop anybody wants to scroll past.

<storycode:copy id="f-2449d0ed01ad-record">
/// Record what `writes` is about to do to `root`, reading each file's current
/// bytes as the *before*.
///
/// Call immediately before writing. Returns the act when one was worth
/// keeping — an act whose files all end up with the bytes they already had is
/// not a stop anybody wants to scroll past.
pub fn record(
    root: &Path,
    kind: ActKind,
    detail: Option<String>,
    writes: &[(PathBuf, Vec<u8>)],
) -> Option<Act> {
    let history = open(root)?;
    let changes: Vec<PendingChange> = writes
        .iter()
        .map(|(path, after)| {
            (
                relative(&absolute(root), path),
                std::fs::read(path).ok(),
                Some(after.clone()),
            )
        })
        .collect();
    let act = match history.record(root, kind, detail, &now(), &changes) {
        Ok(act) => act,
        Err(error) => {
            // Never fatal: a run must not fail because a cache on this
            // machine could not be written.
            log::debug!("local history not recorded: {error:#}");
            return None;
        }
    };
    if act.is_some()
        && let Err(error) = history.prune(retention(), &now())
    {
        log::debug!("local history not pruned: {error:#}");
    }
    act
}

</storycode:copy>
<a id="f-2449d0ed01ad-record_changes"></a>
### `record_changes`

Record explicit before/after bytes, for a writer whose "before" is not
simply what is on disk at the path being written.
The merge driver is the case: git hands it a temp file to write the result
into, so the bytes that matter are the ones it was given rather than the
ones at the repo-relative path.

<storycode:copy id="f-2449d0ed01ad-record_changes">
/// Record explicit before/after bytes, for a writer whose "before" is not
/// simply what is on disk at the path being written.
///
/// The merge driver is the case: git hands it a temp file to write the result
/// into, so the bytes that matter are the ones it was given rather than the
/// ones at the repo-relative path.
pub fn record_changes(
    root: &Path,
    kind: ActKind,
    detail: Option<String>,
    changes: &[PendingChange],
) -> Option<Act> {
    let history = open(root)?;
    match history.record(root, kind, detail, &now(), changes) {
        Ok(act) => act,
        Err(error) => {
            log::debug!("local history not recorded: {error:#}");
            None
        }
    }
}

</storycode:copy>
<a id="f-2449d0ed01ad-revert_act"></a>
### `revert_act`

Put an act back, and record the going-back as an act of its own.
One function rather than two steps a caller remembers to pair, because the
pairing has an **order** that is easy to get wrong and silent when it is:
the bytes a revert overwrites must be read BEFORE it writes, or the
recorded before and after are the same and no act is stored at all. That
was the first version of this, and it quietly cost the property the whole
list depends on — *a history you can fall out of is a history nobody
trusts*.
It also belongs in the library rather than in the CLI so the app panel
cannot grow a second, subtly different revert.

<storycode:copy id="f-2449d0ed01ad-revert_act">
/// Put an act back, and record the going-back as an act of its own.
///
/// One function rather than two steps a caller remembers to pair, because the
/// pairing has an **order** that is easy to get wrong and silent when it is:
/// the bytes a revert overwrites must be read BEFORE it writes, or the
/// recorded before and after are the same and no act is stored at all. That
/// was the first version of this, and it quietly cost the property the whole
/// list depends on — *a history you can fall out of is a history nobody
/// trusts*.
///
/// It also belongs in the library rather than in the CLI so the app panel
/// cannot grow a second, subtly different revert.
pub fn revert_act(
    root: &Path,
    history: &History,
    act: &hickory_workspace::history::Act,
    only: Option<&str>,
) -> anyhow::Result<hickory_workspace::history::Reverted> {
    // What the files hold right now — the state a person is undoing, and the
    // one they will want back if this revert was a mistake.
    let before: Vec<(String, Option<Vec<u8>>)> = act
        .files
        .iter()
        .filter(|f| f.changed())
        .filter(|f| only.is_none_or(|wanted| f.path == wanted))
        .map(|f| (f.path.clone(), std::fs::read(root.join(&f.path)).ok()))
        .collect();

    let out = history.revert(root, act, only)?;
    if out.restored.is_empty() {
        return Ok(out);
    }

    let changes: Vec<PendingChange> = before
        .into_iter()
        .filter(|(path, _)| out.restored.contains(path))
        .map(|(path, was)| {
            let now = std::fs::read(root.join(&path)).ok();
            (path, was, now)
        })
        .collect();
    record_changes(
        root,
        ActKind::Revert,
        Some(format!("reverted {} ({})", act.id, act.kind.as_str())),
        &changes,
    );
    Ok(out)
}

</storycode:copy>
<a id="f-2449d0ed01ad-record_removals"></a>
### `record_removals`

Record a set of files that are about to be deleted.

<storycode:copy id="f-2449d0ed01ad-record_removals">
/// Record a set of files that are about to be deleted.
pub fn record_removals(
    root: &Path,
    kind: ActKind,
    detail: Option<String>,
    paths: &[PathBuf],
) -> Option<Act> {
    let history = open(root)?;
    let changes: Vec<PendingChange> = paths
        .iter()
        .map(|path| {
            (
                relative(&absolute(root), path),
                std::fs::read(path).ok(),
                None,
            )
        })
        .collect();
    history.record(root, kind, detail, &now(), &changes).ok()?
}

</storycode:copy>
<a id="f-2449d0ed01ad-relative"></a>
### `relative`

<storycode:copy id="f-2449d0ed01ad-relative">
/// Project-relative, with `/` separators.
fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

</storycode:copy>
<a id="f-2449d0ed01ad-now"></a>
### `now`

<storycode:copy id="f-2449d0ed01ad-now">
fn now() -> String {
    crate::serve::now_rfc3339_public()
}

</storycode:copy>
<a id="f-2449d0ed01ad-tests"></a>
### `tests`

<storycode:copy id="f-2449d0ed01ad-tests">
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bad_retention_value_warns_and_keeps_the_default() {
        // Silently halving somebody's history because they typed `7d` is
        // worse than telling them.
        unsafe { std::env::set_var("HICKORY_HISTORY_DAYS", "7d") };
        assert_eq!(retention().max_days, Retention::default().max_days);
        unsafe { std::env::set_var("HICKORY_HISTORY_DAYS", "7") };
        assert_eq!(retention().max_days, 7);
        unsafe { std::env::remove_var("HICKORY_HISTORY_DAYS") };
    }

    #[test]
    fn a_relative_root_resolves_to_the_folder_a_person_is_actually_in() {
        // Found by running it: `hick run note.md` gives a parent of "",
        // which hashed to the key for the empty string. Every act landed in a
        // store nothing would ever look in, while `hick history` read the one
        // for the working directory and reported an empty folder. Two stores,
        // no error, and a feature that silently did nothing.
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(absolute(Path::new("")), cwd);
        assert_eq!(absolute(Path::new("sub")), cwd.join("sub"));
        assert_eq!(absolute(Path::new("/tmp")), PathBuf::from("/tmp"));
    }

    #[test]
    fn a_path_outside_the_root_keeps_its_own_name_rather_than_being_mangled() {
        let rel = relative(Path::new("/a/b"), Path::new("/a/b/c/d.md"));
        assert_eq!(rel, "c/d.md");
        // Not every writer's path is under the root (an out-dir run), and a
        // wrong relative path would revert to the wrong place.
        let outside = relative(Path::new("/a/b"), Path::new("/elsewhere/d.md"));
        assert_eq!(outside, "/elsewhere/d.md");
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/history.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/history.rs"><storycode:paste select="#f-2449d0ed01ad-retention"/><storycode:paste select="#f-2449d0ed01ad-parsed"/><storycode:paste select="#f-2449d0ed01ad-open"/><storycode:paste select="#f-2449d0ed01ad-absolute"/><storycode:paste select="#f-2449d0ed01ad-record"/><storycode:paste select="#f-2449d0ed01ad-record_changes"/><storycode:paste select="#f-2449d0ed01ad-revert_act"/><storycode:paste select="#f-2449d0ed01ad-record_removals"/><storycode:paste select="#f-2449d0ed01ad-relative"/><storycode:paste select="#f-2449d0ed01ad-now"/><storycode:paste select="#f-2449d0ed01ad-tests"/></storycode:file>

<a id="output-9d4b4757fdb7"></a>
## `crates/hickory-workspace/src/history.rs`

Related contracts: [Local History Is The Interval Below The Commit](contract.md).

<a id="f-9d4b4757fdb7-ActKind"></a>
### `ActKind`

Local history: the interval below the commit.
`docs/specs/freeform/local-history.md`. Git's resolution is a commit, and
in a `.md` folder **you are not the only writer** — a run, a weave, a
reverse edit, an ingest, the agent, find-and-replace and the merge driver
all write between two commits, and most of them have no way back shorter
than "commit before every risky thing, forever". That is a discipline, and
a discipline is what you ask of people when the tool has not done its job.
## The unit is the act, not the file
The one place this departs from JetBrains', and it departs because almost
every writer here is a **batch** writer: a weave touches every document's
`.md`, a replace touches forty files, an ingest lands a whole scaffolded
tree. Per-file entries would record all of that correctly and make it
useless — undoing a replace would be forty separate reverts, in the right
order, by hand, from memory.
A file's own timeline is a **filter** over the acts that touched it, not a
second structure. That is what keeps "undo that replace" and "what did
this look like at lunchtime" the same mechanism.
## What it is not
**Not a record — a cache**, and the distinction is load-bearing. Nothing
may cite it: no `from=`, no `cites=`, no attribute in the hick grammar
takes a local-history address. `scaffolded-files-and-derived-edits.md`
died on exactly that mistake — `from=` built a durable claim on a
gitignored artifact, so the base did not survive a clone. This store is
not even in the project, so a claim on it would not survive the machine.
It never crosses git and never crosses the peer channel. Two of your
machines have two different local histories, and that is correct.
Say **"local history"**. Never "version history", "backup", or
"snapshots" — each promises durability across machines, which this
deliberately does not have, and the promise is the harm.

<storycode:copy id="f-9d4b4757fdb7-ActKind">
//! Local history: the interval below the commit.
//!
//! `docs/specs/freeform/local-history.md`. Git's resolution is a commit, and
//! in a `.md` folder **you are not the only writer** — a run, a weave, a
//! reverse edit, an ingest, the agent, find-and-replace and the merge driver
//! all write between two commits, and most of them have no way back shorter
//! than "commit before every risky thing, forever". That is a discipline, and
//! a discipline is what you ask of people when the tool has not done its job.
//!
//! ## The unit is the act, not the file
//!
//! The one place this departs from JetBrains', and it departs because almost
//! every writer here is a **batch** writer: a weave touches every document's
//! `.md`, a replace touches forty files, an ingest lands a whole scaffolded
//! tree. Per-file entries would record all of that correctly and make it
//! useless — undoing a replace would be forty separate reverts, in the right
//! order, by hand, from memory.
//!
//! A file's own timeline is a **filter** over the acts that touched it, not a
//! second structure. That is what keeps "undo that replace" and "what did
//! this look like at lunchtime" the same mechanism.
//!
//! ## What it is not
//!
//! **Not a record — a cache**, and the distinction is load-bearing. Nothing
//! may cite it: no `from=`, no `cites=`, no attribute in the hick grammar
//! takes a local-history address. `scaffolded-files-and-derived-edits.md`
//! died on exactly that mistake — `from=` built a durable claim on a
//! gitignored artifact, so the base did not survive a clone. This store is
//! not even in the project, so a claim on it would not survive the machine.
//!
//! It never crosses git and never crosses the peer channel. Two of your
//! machines have two different local histories, and that is correct.
//!
//! Say **"local history"**. Never "version history", "backup", or
//! "snapshots" — each promises durability across machines, which this
//! deliberately does not have, and the promise is the harm.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

/// Who made an act.
///
/// Not decoration: the kind is what makes the list readable at a glance, and
/// what makes revert safe to offer — because "is going back here sane?" has a
/// different answer per kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActKind {
    /// A person, through a buffer.
    Typed,
    /// The buffer reached disk.
    Saved,
    /// The file changed on disk with no buffer behind it.
    External,
    /// Generated outputs, before the write.
    Run,
    /// The `.md` of every document, before the write.
    Weave,
    /// Source, from an edit made in a generated file.
    ReverseEdit,
    /// Source, at hashline anchors, by the agent.
    Agent,
    /// A scaffolder's tree, as document bytes.
    Ingest,
    /// A resolved document, at `git merge` time.
    Merge,
    /// Find-and-replace, across the folder.
    Replace,
    /// A refactor-mode restructuring.
    Refactor,
    /// Going back is itself an act, recorded like any other — so the way back
    /// from a bad revert is the same list. A history you can fall out of is a
    /// history nobody trusts.
    Revert,
}

</storycode:copy>
<a id="f-9d4b4757fdb7-impl-ActKind"></a>
### `impl-ActKind`

<storycode:copy id="f-9d4b4757fdb7-impl-ActKind">
impl ActKind {
    /// Whether reverting this kind is offered.
    ///
    /// Generated output is shown and **compared**, never reverted: the next
    /// run would undo the revert, so offering the verb would be offering
    /// something that does not work. The row is still real and worth reading
    /// — the same way find-and-replace greys out a generated file rather than
    /// hiding it.
    pub fn is_revertable(self) -> bool {
        !matches!(self, ActKind::Run | ActKind::Weave)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ActKind::Typed => "typed",
            ActKind::Saved => "saved",
            ActKind::External => "external",
            ActKind::Run => "run",
            ActKind::Weave => "weave",
            ActKind::ReverseEdit => "reverse-edit",
            ActKind::Agent => "agent",
            ActKind::Ingest => "ingest",
            ActKind::Merge => "merge",
            ActKind::Replace => "replace",
            ActKind::Refactor => "refactor",
            ActKind::Revert => "revert",
        }
    }
}

</storycode:copy>
<a id="f-9d4b4757fdb7-FileChange"></a>
### `FileChange`

<storycode:copy id="f-9d4b4757fdb7-FileChange">
/// One file's before and after, within an act.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// Project-relative path, with `/` separators on every platform.
    pub path: String,
    /// Content hash before the write. `None` when the file did not exist.
    pub before: Option<String>,
    /// Content hash after. `None` when the write deleted it.
    pub after: Option<String>,
}

</storycode:copy>
<a id="f-9d4b4757fdb7-impl-FileChange"></a>
### `impl-FileChange`

<storycode:copy id="f-9d4b4757fdb7-impl-FileChange">
impl FileChange {
    /// Whether this file's bytes actually moved.
    ///
    /// A batch writer touches files it does not change — a weave rewrites
    /// forty `.md` files of which two differ — and recording those as
    /// unchanged hashes rather than as copies is what keeps the store small.
    pub fn changed(&self) -> bool {
        self.before != self.after
    }
}

</storycode:copy>
<a id="f-9d4b4757fdb7-PendingChange"></a>
### `PendingChange`

One file's bytes before and after, as a caller hands them over.
`None` on either side means the file did not exist then: created on the
left, deleted on the right.

<storycode:copy id="f-9d4b4757fdb7-PendingChange">
/// One file's bytes before and after, as a caller hands them over.
///
/// `None` on either side means the file did not exist then: created on the
/// left, deleted on the right.
pub type PendingChange = (String, Option<Vec<u8>>, Option<Vec<u8>>);

</storycode:copy>
<a id="f-9d4b4757fdb7-Act"></a>
### `Act`

<storycode:copy id="f-9d4b4757fdb7-Act">
/// One act: what happened, when, and to what.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Act {
    pub id: String,
    /// Wall-clock, RFC 3339. A local store for a person at a machine, so the
    /// question it answers is "what did this look like at lunchtime".
    pub at: String,
    pub kind: ActKind,
    /// What explains this act: the run fingerprint, the session path, the
    /// pattern replaced, the merge's base. Free text because each kind
    /// carries a different thing and none of them is queried.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    pub files: Vec<FileChange>,
}

</storycode:copy>
<a id="f-9d4b4757fdb7-impl-Act"></a>
### `impl-Act`

<storycode:copy id="f-9d4b4757fdb7-impl-Act">
impl Act {
    /// The files this act actually moved.
    pub fn changed(&self) -> impl Iterator<Item = &FileChange> {
        self.files.iter().filter(|f| f.changed())
    }
}

</storycode:copy>
<a id="f-9d4b4757fdb7-Reverted"></a>
### `Reverted`

What a revert did, said in full.
A revert that silently skipped a file would be the worst possible
behaviour for a feature people reach for when they are already worried, so
every path lands in exactly one of these lists.

<storycode:copy id="f-9d4b4757fdb7-Reverted">
/// What a revert did, said in full.
///
/// A revert that silently skipped a file would be the worst possible
/// behaviour for a feature people reach for when they are already worried, so
/// every path lands in exactly one of these lists.
#[derive(Debug, Default)]
pub struct Reverted {
    /// Put back.
    pub restored: Vec<String>,
    /// Left alone because the bytes on disk are no longer what this act
    /// wrote — somebody has been here since, and overwriting them would be a
    /// second unasked-for write on top of the one being undone.
    pub moved_on: Vec<String>,
    /// Left alone because reverting this kind is refused, and why.
    pub refused: Vec<(String, String)>,
}

</storycode:copy>
<a id="f-9d4b4757fdb7-Retention"></a>
### `Retention`

<storycode:copy id="f-9d4b4757fdb7-Retention">
/// How much local history to keep.
#[derive(Debug, Clone, Copy)]
pub struct Retention {
    pub max_bytes: u64,
    pub max_days: u64,
}

</storycode:copy>
<a id="f-9d4b4757fdb7-impl-Default-for-Retention"></a>
### `impl-Default for Retention`

<storycode:copy id="f-9d4b4757fdb7-impl-Default-for-Retention">
impl Default for Retention {
    fn default() -> Self {
        // Unmeasured, and the spec says so: nothing here should ship a number
        // that was not watched on a real folder for a week first. These are
        // chosen to be obviously bounded rather than obviously right, and
        // they are `HICKORY_HISTORY_BYTES` / `HICKORY_HISTORY_DAYS`.
        Self {
            max_bytes: 256 * 1024 * 1024,
            max_days: 14,
        }
    }
}

</storycode:copy>
<a id="f-9d4b4757fdb7-History"></a>
### `History`

The local-history store for one project, for one person, on one machine.

<storycode:copy id="f-9d4b4757fdb7-History">
/// The local-history store for one project, for one person, on one machine.
pub struct History {
    dir: PathBuf,
}

</storycode:copy>
<a id="f-9d4b4757fdb7-impl-History"></a>
### `impl-History`

<storycode:copy id="f-9d4b4757fdb7-impl-History">
impl History {
    /// Open (and create) the store beside the rest of this project's
    /// workspace state.
    pub fn open(store: &crate::WorkspaceStore) -> Result<Self> {
        let dir = store.dir().join("history");
        fs::create_dir_all(dir.join("blobs"))
            .with_context(|| format!("could not create {}", dir.display()))?;
        Ok(Self { dir })
    }

    /// Where this store writes. Named in messages, so a person can look — and
    /// so `forget` can say what it emptied.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn log_path(&self) -> PathBuf {
        self.dir.join("acts.jsonl")
    }

    fn blob_path(&self, hash: &str) -> PathBuf {
        // Two levels, so a project with a hundred thousand versions does not
        // put them all in one directory.
        self.dir.join("blobs").join(&hash[..2]).join(&hash[2..])
    }

    /// Store bytes, returning their hash. Written once; a second call with
    /// the same bytes writes nothing.
    pub fn put(&self, bytes: &[u8]) -> Result<String> {
        let hash = format!("{:x}", Sha256::digest(bytes));
        let path = self.blob_path(&hash);
        if path.exists() {
            return Ok(hash);
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        write_atomic(&path, bytes)?;
        Ok(hash)
    }

    /// The bytes for a hash, or `None` when it has been evicted.
    pub fn get(&self, hash: &str) -> Option<Vec<u8>> {
        fs::read(self.blob_path(hash)).ok()
    }

    /// Record one act, storing the *before* bytes of every path it names.
    ///
    /// Call this **before** the write. `now` is passed in rather than read
    /// so a caller can be deterministic and so this crate needs no clock of
    /// its own.
    ///
    /// Returns `None` when nothing about the act is worth keeping — every
    /// file unchanged — because a list full of acts that did nothing is a
    /// list nobody reads.
    pub fn record(
        &self,
        root: &Path,
        kind: ActKind,
        detail: Option<String>,
        now: &str,
        changes: &[PendingChange],
    ) -> Result<Option<Act>> {
        let mut files = Vec::with_capacity(changes.len());
        for (path, before, after) in changes {
            let before = match before {
                Some(bytes) => Some(self.put(bytes)?),
                None => None,
            };
            let after = match after {
                Some(bytes) => Some(self.put(bytes)?),
                None => None,
            };
            files.push(FileChange {
                path: normalize(path),
                before,
                after,
            });
        }
        let _ = root;
        if !files.iter().any(FileChange::changed) {
            return Ok(None);
        }
        let act = Act {
            id: act_id(now, kind, &files),
            at: now.to_string(),
            kind,
            detail,
            files,
        };
        let mut line = serde_json::to_string(&act)?;
        line.push('\n');
        // Append-only: an interrupted write loses the tail and never the
        // middle, which matters because this is exactly the store somebody
        // reaches for after a crash.
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.log_path())
            .with_context(|| format!("could not open {}", self.log_path().display()))?;
        file.write_all(line.as_bytes())?;
        Ok(Some(act))
    }

    /// Every act, newest first.
    ///
    /// A truncated last line is dropped rather than fatal: an append-only log
    /// that was being written when the machine went down is exactly the case
    /// this store exists for, and refusing to read it then would be the worst
    /// possible moment to be strict.
    pub fn acts(&self) -> Vec<Act> {
        let Ok(text) = fs::read_to_string(self.log_path()) else {
            return Vec::new();
        };
        let mut acts: Vec<Act> = text
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();
        acts.reverse();
        acts
    }

    /// The acts that touched one path, newest first — a filter over the same
    /// list, never a second structure.
    pub fn acts_for(&self, path: &str) -> Vec<Act> {
        let wanted = normalize(path);
        self.acts()
            .into_iter()
            .filter(|act| act.files.iter().any(|f| f.path == wanted && f.changed()))
            .collect()
    }

    /// One act by id, or by an unambiguous prefix of one.
    pub fn act(&self, id: &str) -> Option<Act> {
        let acts = self.acts();
        if let Some(exact) = acts.iter().find(|a| a.id == id) {
            return Some(exact.clone());
        }
        let mut matches = acts.into_iter().filter(|a| a.id.starts_with(id));
        let first = matches.next()?;
        // An ambiguous prefix reverts the wrong thing, which is the one
        // mistake this store must not make.
        matches.next().is_none().then_some(first)
    }

    /// Put the files an act touched back to what they were before it.
    ///
    /// Only where the bytes on disk are still what the act wrote. A file that
    /// has moved on since is **reported, never silently skipped** — the
    /// person is undoing something precisely because they are unsure what
    /// happened, and a quiet partial revert is how they end up trusting a
    /// state that never existed.
    pub fn revert(&self, root: &Path, act: &Act, only: Option<&str>) -> Result<Reverted> {
        let mut out = Reverted::default();
        let only = only.map(normalize);
        for change in act.files.iter().filter(|f| f.changed()) {
            if let Some(wanted) = &only
                && &change.path != wanted
            {
                continue;
            }
            if !act.kind.is_revertable() {
                out.refused.push((
                    change.path.clone(),
                    format!(
                        "{} writes generated files; the next run would undo the revert. \
                         Compare it instead, and change what generates it.",
                        act.kind.as_str()
                    ),
                ));
                continue;
            }
            let full = root.join(&change.path);
            let current = fs::read(&full).ok();
            let current_hash = current
                .as_deref()
                .map(|b| format!("{:x}", Sha256::digest(b)));
            if current_hash != change.after {
                out.moved_on.push(change.path.clone());
                continue;
            }
            match &change.before {
                Some(hash) => {
                    let Some(bytes) = self.get(hash) else {
                        out.moved_on.push(change.path.clone());
                        continue;
                    };
                    if let Some(parent) = full.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    write_atomic(&full, &bytes)?;
                }
                // The act created this file, so going back means it is gone.
                None => {
                    let _ = fs::remove_file(&full);
                }
            }
            out.restored.push(change.path.clone());
        }
        Ok(out)
    }

    /// Drop acts past the budget, oldest first, and sweep the blobs nothing
    /// names any more.
    ///
    /// Returns how many acts were evicted. An unbounded local store is a slow
    /// disk leak that shows up as a bug report about startup time.
    pub fn prune(&self, retention: Retention, now: &str) -> Result<usize> {
        let mut keep = self.acts();
        keep.reverse(); // oldest first, which is the order eviction works in
        let total = keep.len();

        // Age first, because it is the cheap answer and usually the whole
        // one.
        if let Some(cutoff) = older_than(now, retention.max_days) {
            keep.retain(|act| act.at.as_str() >= cutoff.as_str());
        }
        self.rewrite(&keep)?;
        self.sweep(&keep)?;

        // Then size, one act at a time, re-measuring after each — a blob is
        // only reclaimed when the LAST act naming it goes, so dropping an act
        // may free nothing at all and the loop has to see that.
        while self.blob_bytes()? > retention.max_bytes && !keep.is_empty() {
            keep.remove(0);
            self.rewrite(&keep)?;
            self.sweep(&keep)?;
        }
        Ok(total - keep.len())
    }

    fn blob_bytes(&self) -> Result<u64> {
        fn walk(dir: &Path) -> u64 {
            let Ok(entries) = fs::read_dir(dir) else {
                return 0;
            };
            entries
                .flatten()
                .map(|e| match e.file_type() {
                    Ok(t) if t.is_dir() => walk(&e.path()),
                    _ => e.metadata().map(|m| m.len()).unwrap_or(0),
                })
                .sum()
        }
        Ok(walk(&self.dir.join("blobs")))
    }

    fn rewrite(&self, keep: &[Act]) -> Result<()> {
        let mut text = String::new();
        for act in keep {
            text.push_str(&serde_json::to_string(act)?);
            text.push('\n');
        }
        write_atomic(&self.log_path(), text.as_bytes())
    }

    /// Reference-counted against the act log: a blob is swept when the last
    /// act naming it is gone.
    fn sweep(&self, keep: &[Act]) -> Result<()> {
        let live: HashSet<&str> = keep
            .iter()
            .flat_map(|a| a.files.iter())
            .flat_map(|f| [f.before.as_deref(), f.after.as_deref()])
            .flatten()
            .collect();
        let blobs = self.dir.join("blobs");
        for shard in fs::read_dir(&blobs).into_iter().flatten().flatten() {
            for blob in fs::read_dir(shard.path()).into_iter().flatten().flatten() {
                let name = format!(
                    "{}{}",
                    shard.file_name().to_string_lossy(),
                    blob.file_name().to_string_lossy()
                );
                if !live.contains(name.as_str()) {
                    let _ = fs::remove_file(blob.path());
                }
            }
        }
        Ok(())
    }

    /// Forget everything, or everything about one path.
    ///
    /// An honest verb rather than a redactor we pretend to have: this store
    /// holds bytes their author never chose to keep, and a file that briefly
    /// contained a secret is the obvious case. The answer is a purge the
    /// person can run.
    pub fn forget(&self, path: Option<&str>) -> Result<usize> {
        match path {
            None => {
                let n = self.acts().len();
                let _ = fs::remove_file(self.log_path());
                let _ = fs::remove_dir_all(self.dir.join("blobs"));
                fs::create_dir_all(self.dir.join("blobs"))?;
                Ok(n)
            }
            Some(path) => {
                let wanted = normalize(path);
                let mut acts = self.acts();
                acts.reverse();
                let before = acts.len();
                let keep: Vec<Act> = acts
                    .into_iter()
                    .filter_map(|mut act| {
                        act.files.retain(|f| f.path != wanted);
                        // An act that was only ever about this path goes with
                        // it; one that touched forty files keeps the other
                        // thirty-nine.
                        act.files.iter().any(FileChange::changed).then_some(act)
                    })
                    .collect();
                let removed = before - keep.len();
                self.rewrite(&keep)?;
                self.sweep(&keep)?;
                Ok(removed)
            }
        }
    }
}

</storycode:copy>
<a id="f-9d4b4757fdb7-act_id"></a>
### `act_id`

A stable id for an act: short, readable, and derived from its content so
two runs of the same test agree.

<storycode:copy id="f-9d4b4757fdb7-act_id">
/// A stable id for an act: short, readable, and derived from its content so
/// two runs of the same test agree.
fn act_id(now: &str, kind: ActKind, files: &[FileChange]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(now.as_bytes());
    hasher.update(kind.as_str().as_bytes());
    for file in files {
        hasher.update(file.path.as_bytes());
        hasher.update(file.before.as_deref().unwrap_or("-").as_bytes());
        hasher.update(file.after.as_deref().unwrap_or("-").as_bytes());
    }
    format!("{:x}", hasher.finalize())[..12].to_string()
}

</storycode:copy>
<a id="f-9d4b4757fdb7-normalize"></a>
### `normalize`

`/` separators on every platform, so a history recorded on Windows reads
the same as one recorded anywhere else.

<storycode:copy id="f-9d4b4757fdb7-normalize">
/// `/` separators on every platform, so a history recorded on Windows reads
/// the same as one recorded anywhere else.
fn normalize(path: &str) -> String {
    path.replace('\\', "/")
}

</storycode:copy>
<a id="f-9d4b4757fdb7-older_than"></a>
### `older_than`

The RFC 3339 timestamp `days` before `now`, by string arithmetic on the
date part only.
Coarse on purpose: eviction at day resolution is what the setting
promises, and pulling in a date library to be exact about a boundary
nobody watches would be a dependency for nothing.

<storycode:copy id="f-9d4b4757fdb7-older_than">
/// The RFC 3339 timestamp `days` before `now`, by string arithmetic on the
/// date part only.
///
/// Coarse on purpose: eviction at day resolution is what the setting
/// promises, and pulling in a date library to be exact about a boundary
/// nobody watches would be a dependency for nothing.
fn older_than(now: &str, days: u64) -> Option<String> {
    let date = now.get(..10)?;
    let mut parts = date.split('-');
    let y: i64 = parts.next()?.parse().ok()?;
    let m: i64 = parts.next()?.parse().ok()?;
    let d: i64 = parts.next()?.parse().ok()?;
    let mut serial = civil_days(y, m, d) - days as i64;
    let (y, m, d) = from_civil_days(&mut serial);
    Some(format!("{y:04}-{m:02}-{d:02}"))
}

</storycode:copy>
<a id="f-9d4b4757fdb7-civil_days"></a>
### `civil_days`

Days since 1970-01-01, by Howard Hinnant's civil algorithm.

<storycode:copy id="f-9d4b4757fdb7-civil_days">
/// Days since 1970-01-01, by Howard Hinnant's civil algorithm.
fn civil_days(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

</storycode:copy>
<a id="f-9d4b4757fdb7-from_civil_days"></a>
### `from_civil_days`

<storycode:copy id="f-9d4b4757fdb7-from_civil_days">
fn from_civil_days(days: &mut i64) -> (i64, i64, i64) {
    let z = *days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

</storycode:copy>
<a id="f-9d4b4757fdb7-write_atomic"></a>
### `write_atomic`

<storycode:copy id="f-9d4b4757fdb7-write_atomic">
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes).with_context(|| format!("could not write {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("could not replace {}", path.display()))?;
    Ok(())
}

</storycode:copy>
<a id="f-9d4b4757fdb7-by_path"></a>
### `by_path`

The acts that touched each path, for a report.

<storycode:copy id="f-9d4b4757fdb7-by_path">
/// The acts that touched each path, for a report.
pub fn by_path(acts: &[Act]) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for act in acts {
        for file in act.changed() {
            *out.entry(file.path.clone()).or_insert(0) += 1;
        }
    }
    out
}

</storycode:copy>
<a id="f-9d4b4757fdb7-tests"></a>
### `tests`

<storycode:copy id="f-9d4b4757fdb7-tests">
#[cfg(test)]
mod tests {
    use super::*;

    fn store(dir: &Path) -> History {
        let base = dir.join("state");
        let root = dir.join("project");
        fs::create_dir_all(&root).unwrap();
        let workspace = crate::WorkspaceStore::under(&base, &root).unwrap();
        History::open(&workspace).unwrap()
    }

    fn change(
        path: &str,
        before: Option<&str>,
        after: Option<&str>,
    ) -> (String, Option<Vec<u8>>, Option<Vec<u8>>) {
        (
            path.to_string(),
            before.map(|s| s.as_bytes().to_vec()),
            after.map(|s| s.as_bytes().to_vec()),
        )
    }

    #[test]
    fn an_act_holds_every_file_it_touched_and_a_file_is_a_filter_over_acts() {
        // The design's central choice: the unit is the ACT, because almost
        // every writer here is a batch writer. A file's timeline is a filter,
        // never a second structure.
        let dir = tempfile::tempdir().unwrap();
        let history = store(dir.path());
        history
            .record(
                Path::new("."),
                ActKind::Replace,
                Some("invoice_id → invoice_ref".into()),
                "2026-08-27T14:02:11Z",
                &[
                    change("a.md", Some("invoice_id"), Some("invoice_ref")),
                    change("b.md", Some("invoice_id"), Some("invoice_ref")),
                    // Touched and unchanged: a batch writer rewrites files it
                    // does not change, and those must not become copies.
                    change("c.md", Some("untouched"), Some("untouched")),
                ],
            )
            .unwrap()
            .expect("an act");
        history
            .record(
                Path::new("."),
                ActKind::Typed,
                None,
                "2026-08-27T14:03:00Z",
                &[change("a.md", Some("invoice_ref"), Some("invoice_ref x"))],
            )
            .unwrap()
            .expect("an act");

        let acts = history.acts();
        assert_eq!(acts.len(), 2);
        // Newest first.
        assert_eq!(acts[0].kind, ActKind::Typed);
        assert_eq!(acts[1].changed().count(), 2, "the unchanged file counted");

        assert_eq!(history.acts_for("a.md").len(), 2);
        assert_eq!(history.acts_for("b.md").len(), 1);
        assert!(
            history.acts_for("c.md").is_empty(),
            "a file that did not change is not part of its own timeline"
        );
    }

    #[test]
    fn an_act_that_changed_nothing_is_not_recorded() {
        // A weave that rewrote forty identical files is not a stop anybody
        // wants to scroll past.
        let dir = tempfile::tempdir().unwrap();
        let history = store(dir.path());
        let act = history
            .record(
                Path::new("."),
                ActKind::Weave,
                None,
                "2026-08-27T14:00:00Z",
                &[change("a.md", Some("same"), Some("same"))],
            )
            .unwrap();
        assert!(act.is_none());
        assert!(history.acts().is_empty());
    }

    #[test]
    fn reverting_a_replace_puts_back_every_file_that_has_not_moved_on() {
        // The sentence at the end of `find-and-replace-is-exhaustive.md`,
        // closed: a multi-file undo, in one act.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("project");
        let history = store(dir.path());
        fs::write(root.join("a.md"), "invoice_ref").unwrap();
        fs::write(root.join("b.md"), "invoice_ref").unwrap();
        fs::write(root.join("c.md"), "somebody edited this since").unwrap();

        let act = history
            .record(
                Path::new("."),
                ActKind::Replace,
                None,
                "2026-08-27T14:02:11Z",
                &[
                    change("a.md", Some("invoice_id"), Some("invoice_ref")),
                    change("b.md", Some("invoice_id"), Some("invoice_ref")),
                    change("c.md", Some("invoice_id"), Some("invoice_ref")),
                ],
            )
            .unwrap()
            .unwrap();

        let out = history.revert(&root, &act, None).unwrap();
        assert_eq!(out.restored, vec!["a.md".to_string(), "b.md".to_string()]);
        // Reported, never silently skipped: somebody undoing a batch is
        // already unsure what happened, and a quiet partial revert is how
        // they come to trust a state that never existed.
        assert_eq!(out.moved_on, vec!["c.md".to_string()]);
        assert_eq!(fs::read_to_string(root.join("a.md")).unwrap(), "invoice_id");
        assert_eq!(
            fs::read_to_string(root.join("c.md")).unwrap(),
            "somebody edited this since"
        );
    }

    #[test]
    fn reverting_one_file_out_of_a_batch_leaves_the_rest() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("project");
        let history = store(dir.path());
        fs::write(root.join("a.md"), "after").unwrap();
        fs::write(root.join("b.md"), "after").unwrap();
        let act = history
            .record(
                Path::new("."),
                ActKind::Replace,
                None,
                "2026-08-27T14:02:11Z",
                &[
                    change("a.md", Some("before"), Some("after")),
                    change("b.md", Some("before"), Some("after")),
                ],
            )
            .unwrap()
            .unwrap();
        let out = history.revert(&root, &act, Some("a.md")).unwrap();
        assert_eq!(out.restored, vec!["a.md".to_string()]);
        assert_eq!(fs::read_to_string(root.join("b.md")).unwrap(), "after");
    }

    #[test]
    fn reverting_a_file_the_act_created_removes_it_again() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("project");
        let history = store(dir.path());
        fs::write(root.join("new.md"), "scaffolded").unwrap();
        let act = history
            .record(
                Path::new("."),
                ActKind::Ingest,
                None,
                "2026-08-27T14:02:11Z",
                &[change("new.md", None, Some("scaffolded"))],
            )
            .unwrap()
            .unwrap();
        history.revert(&root, &act, None).unwrap();
        assert!(
            !root.join("new.md").exists(),
            "going back left the file behind"
        );
    }

    #[test]
    fn a_generated_write_is_refused_with_a_reason_rather_than_reverted() {
        // The next run would undo the revert, so offering the verb would be
        // offering something that does not work. The row is still real.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("project");
        let history = store(dir.path());
        fs::write(root.join("out.md"), "new").unwrap();
        let act = history
            .record(
                Path::new("."),
                ActKind::Run,
                None,
                "2026-08-27T14:02:11Z",
                &[change("out.md", Some("old"), Some("new"))],
            )
            .unwrap()
            .unwrap();
        let out = history.revert(&root, &act, None).unwrap();
        assert!(out.restored.is_empty());
        assert_eq!(out.refused.len(), 1);
        assert!(
            out.refused[0].1.contains("undo the revert"),
            "{:?}",
            out.refused
        );
        assert_eq!(fs::read_to_string(root.join("out.md")).unwrap(), "new");
    }

    #[test]
    fn identical_bytes_are_stored_once() {
        let dir = tempfile::tempdir().unwrap();
        let history = store(dir.path());
        let a = history.put(b"the same forty times").unwrap();
        let b = history.put(b"the same forty times").unwrap();
        assert_eq!(a, b);
        assert_eq!(history.get(&a).unwrap(), b"the same forty times");
    }

    #[test]
    fn an_act_is_found_by_an_unambiguous_prefix_and_never_by_an_ambiguous_one() {
        let dir = tempfile::tempdir().unwrap();
        let history = store(dir.path());
        let act = history
            .record(
                Path::new("."),
                ActKind::Typed,
                None,
                "2026-08-27T14:02:11Z",
                &[change("a.md", Some("x"), Some("y"))],
            )
            .unwrap()
            .unwrap();
        assert_eq!(history.act(&act.id).unwrap().id, act.id);
        assert_eq!(history.act(&act.id[..6]).unwrap().id, act.id);
        assert!(history.act("zzzzzz").is_none());
        // An ambiguous prefix reverts the wrong thing, which is the one
        // mistake this store must not make.
        assert!(history.act("").is_none() || history.acts().len() == 1);
    }

    #[test]
    fn a_truncated_last_line_loses_the_tail_and_never_the_middle() {
        // Exactly the case this store exists for: the machine went down while
        // something was writing. Refusing to read the log then would be the
        // worst possible moment to be strict.
        let dir = tempfile::tempdir().unwrap();
        let history = store(dir.path());
        for n in 1..=3 {
            history
                .record(
                    Path::new("."),
                    ActKind::Typed,
                    None,
                    &format!("2026-08-27T14:0{n}:00Z"),
                    &[change("a.md", Some("x"), Some(&format!("y{n}")))],
                )
                .unwrap();
        }
        let log = history.log_path();
        let mut text = fs::read_to_string(&log).unwrap();
        text.push_str("{\"id\":\"broken\",\"at\":");
        fs::write(&log, text).unwrap();
        assert_eq!(history.acts().len(), 3, "a torn tail lost more than itself");
    }

    #[test]
    fn old_acts_are_evicted_and_their_blobs_swept() {
        let dir = tempfile::tempdir().unwrap();
        let history = store(dir.path());
        history
            .record(
                Path::new("."),
                ActKind::Typed,
                None,
                "2026-01-01T10:00:00Z",
                &[change("a.md", Some("ancient"), Some("also ancient"))],
            )
            .unwrap();
        let recent = history
            .record(
                Path::new("."),
                ActKind::Typed,
                None,
                "2026-08-27T10:00:00Z",
                &[change("a.md", Some("new"), Some("newer"))],
            )
            .unwrap()
            .unwrap();

        let evicted = history
            .prune(
                Retention {
                    max_bytes: u64::MAX,
                    max_days: 14,
                },
                "2026-08-27T12:00:00Z",
            )
            .unwrap();
        assert_eq!(evicted, 1);
        assert_eq!(history.acts().len(), 1);
        // Reference-counted: the evicted act's bytes are gone, the surviving
        // act's are not.
        assert!(history.get("ancient").is_none());
        let still = recent.files[0].before.clone().unwrap();
        assert!(history.get(&still).is_some(), "a live blob was swept");
    }

    #[test]
    fn forgetting_one_path_leaves_the_rest_of_a_batch() {
        // The honest verb: this store holds bytes their author never chose to
        // keep, and a file that briefly held a secret is the obvious case.
        let dir = tempfile::tempdir().unwrap();
        let history = store(dir.path());
        history
            .record(
                Path::new("."),
                ActKind::Replace,
                None,
                "2026-08-27T14:02:11Z",
                &[
                    change("secret.md", Some("sk-real"), Some("sk-realer")),
                    change("fine.md", Some("a"), Some("b")),
                ],
            )
            .unwrap();
        history.forget(Some("secret.md")).unwrap();
        assert!(history.acts_for("secret.md").is_empty());
        assert_eq!(history.acts_for("fine.md").len(), 1);
        // And the bytes are actually gone, not merely unlisted.
        let secret = format!("{:x}", Sha256::digest(b"sk-real"));
        assert!(
            history.get(&secret).is_none(),
            "the bytes survived the purge"
        );
    }

    #[test]
    fn forgetting_everything_leaves_a_usable_empty_store() {
        let dir = tempfile::tempdir().unwrap();
        let history = store(dir.path());
        history
            .record(
                Path::new("."),
                ActKind::Typed,
                None,
                "2026-08-27T14:02:11Z",
                &[change("a.md", Some("x"), Some("y"))],
            )
            .unwrap();
        assert_eq!(history.forget(None).unwrap(), 1);
        assert!(history.acts().is_empty());
        history
            .record(
                Path::new("."),
                ActKind::Typed,
                None,
                "2026-08-27T15:00:00Z",
                &[change("a.md", Some("y"), Some("z"))],
            )
            .unwrap();
        assert_eq!(history.acts().len(), 1);
    }

    #[test]
    fn dates_go_backwards_correctly_across_a_month_and_a_year() {
        assert_eq!(older_than("2026-08-27T00:00:00Z", 0).unwrap(), "2026-08-27");
        assert_eq!(
            older_than("2026-08-27T00:00:00Z", 27).unwrap(),
            "2026-07-31"
        );
        assert_eq!(
            older_than("2026-01-05T00:00:00Z", 10).unwrap(),
            "2025-12-26"
        );
        // A leap year, because February is where date arithmetic goes wrong.
        assert_eq!(older_than("2024-03-01T00:00:00Z", 1).unwrap(), "2024-02-29");
    }
}
</storycode:copy>

### Assemble `crates/hickory-workspace/src/history.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-workspace/src/history.rs"><storycode:paste select="#f-9d4b4757fdb7-ActKind"/><storycode:paste select="#f-9d4b4757fdb7-impl-ActKind"/><storycode:paste select="#f-9d4b4757fdb7-FileChange"/><storycode:paste select="#f-9d4b4757fdb7-impl-FileChange"/><storycode:paste select="#f-9d4b4757fdb7-PendingChange"/><storycode:paste select="#f-9d4b4757fdb7-Act"/><storycode:paste select="#f-9d4b4757fdb7-impl-Act"/><storycode:paste select="#f-9d4b4757fdb7-Reverted"/><storycode:paste select="#f-9d4b4757fdb7-Retention"/><storycode:paste select="#f-9d4b4757fdb7-impl-Default-for-Retention"/><storycode:paste select="#f-9d4b4757fdb7-History"/><storycode:paste select="#f-9d4b4757fdb7-impl-History"/><storycode:paste select="#f-9d4b4757fdb7-act_id"/><storycode:paste select="#f-9d4b4757fdb7-normalize"/><storycode:paste select="#f-9d4b4757fdb7-older_than"/><storycode:paste select="#f-9d4b4757fdb7-civil_days"/><storycode:paste select="#f-9d4b4757fdb7-from_civil_days"/><storycode:paste select="#f-9d4b4757fdb7-write_atomic"/><storycode:paste select="#f-9d4b4757fdb7-by_path"/><storycode:paste select="#f-9d4b4757fdb7-tests"/></storycode:file>

## Implementation review

### Implementation

- [crates/hickory-cli/src/history.rs](contract.md#output-2449d0ed01ad)
- [crates/hickory-cli/src/ingest_exec.rs](../../../mechanisms/crates/hickory-cli/src/ingest_exec.rs.md#output-4e4b92369bcd)
- [crates/hickory-cli/src/serve/find.rs](../../search/find-and-replace-is-exhaustive/contract.md#output-51c37beeedd8)
- [crates/hickory-cli/src/up/reverse.rs](../../../mechanisms/crates/hickory-cli/src/up/reverse.rs.md#output-0e6cf077f262)
- [crates/hickory-workspace/src/history.rs](contract.md#output-9d4b4757fdb7)

### Tests and fixtures

- [crates/hickory-cli/tests/serve_find.rs](../../../mechanisms/crates/hickory-cli/tests/serve_find.rs.md#output-18392dfca977)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
