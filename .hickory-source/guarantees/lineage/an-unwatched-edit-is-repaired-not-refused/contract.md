<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Unwatched Edit Leaves Its Correspondence Behind Before It Is Committed

For: Authors editing generated text and reviewers tracing a result to its source.

Given continuity is on for a project, when a `.md` document is staged whose
edit no recording site saw, then the pre-commit hook reconstructs what moved
and records it at **line** precision — and the commit is never blocked.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/an-unwatched-edit-is-repaired-not-refused.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-dbf85c3272ee"></a>
## `crates/hickory-cli/tests/continuity_repair.rs`

Related contracts: [An Unwatched Edit Leaves Its Correspondence Behind Before It Is Committed](contract.md).

<a id="f-dbf85c3272ee-hick"></a>
### `hick`

The pre-commit repair: an unwatched edit leaves its correspondence behind
before it is committed.
Protects `docs/guarantees/lineage/an-unwatched-edit-is-repaired-not-refused.md`.
The workflow this product is built around — editing a generated file in
your own editor — is by definition unwatched, so an enforcement rule would
tax hardest exactly the thing the product exists for. At commit time both
sides are in hand, which is the same position `refactor/end` is in, so the
hook reconstructs the correspondence instead of rejecting the edit.

<storycode:copy id="f-dbf85c3272ee-hick">
//! The pre-commit repair: an unwatched edit leaves its correspondence behind
//! before it is committed.
//!
//! Protects `docs/guarantees/lineage/an-unwatched-edit-is-repaired-not-refused.md`.
//!
//! The workflow this product is built around — editing a generated file in
//! your own editor — is by definition unwatched, so an enforcement rule would
//! tax hardest exactly the thing the product exists for. At commit time both
//! sides are in hand, which is the same position `refactor/end` is in, so the
//! hook reconstructs the correspondence instead of rejecting the edit.

use std::path::Path;
use std::process::Command;

fn hick(state: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hick"));
    cmd.env("HICKORY_EXECUTOR", "local");
    cmd.env("HICKORY_STATE_DIR", state);
    cmd
}

</storycode:copy>
<a id="f-dbf85c3272ee-git"></a>
### `git`

<storycode:copy id="f-dbf85c3272ee-git">
fn git(dir: &Path, args: &[&str]) -> std::process::Output {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

</storycode:copy>
<a id="f-dbf85c3272ee-BEFORE"></a>
### `BEFORE`

<storycode:copy id="f-dbf85c3272ee-BEFORE">
const BEFORE: &str = "alpha\nbravo\ncharlie\n";
</storycode:copy>
<a id="f-dbf85c3272ee-MOVED"></a>
### `MOVED`

<storycode:copy id="f-dbf85c3272ee-MOVED">
const MOVED: &str = "charlie\nalpha\nbravo\n";

</storycode:copy>
<a id="f-dbf85c3272ee-repo"></a>
### `repo`

<storycode:copy id="f-dbf85c3272ee-repo">
fn repo(dir: &Path) {
    git(dir, &["init", "-q", "-b", "master"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "T"]);
    git(dir, &["config", "commit.gpgsign", "false"]);
}

</storycode:copy>
<a id="f-dbf85c3272ee-journal"></a>
### `journal`

<storycode:copy id="f-dbf85c3272ee-journal">
fn journal(dir: &Path) -> Vec<serde_json::Value> {
    let Ok(body) = std::fs::read_to_string(dir.join(".hick-journal/correspondences.jsonl")) else {
        return Vec::new();
    };
    body.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

</storycode:copy>
<a id="f-dbf85c3272ee-an_unwatched_move_is_recorded_at_commit_time"></a>
### `an_unwatched_move_is_recorded_at_commit_time`

<storycode:copy id="f-dbf85c3272ee-an_unwatched_move_is_recorded_at_commit_time">
#[test]
fn an_unwatched_move_is_recorded_at_commit_time() {
    let dir = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    repo(dir.path());
    let doc = dir.path().join("note.md");
    std::fs::write(&doc, BEFORE).unwrap();
    git(dir.path(), &["add", "-A"]);
    git(dir.path(), &["commit", "-q", "--no-verify", "-m", "base"]);

    let store = hickory_workspace::WorkspaceStore::under(state.path(), dir.path()).unwrap();
    store.set_continuity(true).unwrap();

    // An edit nothing was watching: made in another editor, staged by hand.
    std::fs::write(&doc, MOVED).unwrap();
    git(dir.path(), &["add", "-A"]);

    let out = hick(state.path())
        .arg("repair")
        .arg(dir.path())
        .output()
        .expect("run hick");
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    // It says what it RECORDED, never what it refused.
    assert!(out.status.success(), "{stderr}");
    assert!(stderr.contains("recorded"), "{stderr}");
    assert!(!stderr.to_lowercase().contains("refus"), "{stderr}");

    let entries = journal(dir.path());
    assert!(!entries.is_empty(), "nothing recorded");
    for entry in &entries {
        // Line-precise, because two snapshots of a text file support no more —
        // and recorded ONCE, so it does not decay across later hops.
        assert_eq!(entry["precision"], serde_json::json!("line"), "{entry}");
        assert_eq!(entry["site"], serde_json::json!("repair"), "{entry}");
        assert_ne!(entry["from"]["span"], entry["to"]["span"], "{entry}");
    }
}

</storycode:copy>
<a id="f-dbf85c3272ee-the_repair_never_blocks_a_commit"></a>
### `the_repair_never_blocks_a_commit`

<storycode:copy id="f-dbf85c3272ee-the_repair_never_blocks_a_commit">
#[test]
fn the_repair_never_blocks_a_commit() {
    // A rule with no escape hatch gets the hook disabled entirely, taking the
    // `hick test` gate down with it. So the check IS the repair.
    let dir = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    repo(dir.path());
    std::fs::write(dir.path().join("note.md"), BEFORE).unwrap();
    git(dir.path(), &["add", "-A"]);
    git(dir.path(), &["commit", "-q", "--no-verify", "-m", "base"]);

    let store = hickory_workspace::WorkspaceStore::under(state.path(), dir.path()).unwrap();
    store.set_continuity(true).unwrap();
    std::fs::write(dir.path().join("note.md"), MOVED).unwrap();
    git(dir.path(), &["add", "-A"]);

    let out = hick(state.path())
        .arg("repair")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    // Twice is fine too: it appends, it does not gate.
    assert!(
        hick(state.path())
            .arg("repair")
            .arg(dir.path())
            .output()
            .unwrap()
            .status
            .success()
    );
}

</storycode:copy>
<a id="f-dbf85c3272ee-with_continuity_off_the_repair_does_nothing_at_all"></a>
### `with_continuity_off_the_repair_does_nothing_at_all`

<storycode:copy id="f-dbf85c3272ee-with_continuity_off_the_repair_does_nothing_at_all">
#[test]
fn with_continuity_off_the_repair_does_nothing_at_all() {
    // No continuity, no journal, no check. The workflow this product is built
    // around is unwatched by definition, and taxing every commit to feed a
    // feature most people never turn on is what this refuses.
    let dir = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    repo(dir.path());
    std::fs::write(dir.path().join("note.md"), BEFORE).unwrap();
    git(dir.path(), &["add", "-A"]);
    git(dir.path(), &["commit", "-q", "--no-verify", "-m", "base"]);
    std::fs::write(dir.path().join("note.md"), MOVED).unwrap();
    git(dir.path(), &["add", "-A"]);

    let out = hick(state.path())
        .arg("repair")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).trim().is_empty());
    assert!(!dir.path().join(".hick-journal").exists());
    assert!(journal(dir.path()).is_empty());
}

</storycode:copy>
<a id="f-dbf85c3272ee-a_folder_that_is_not_a_repository_is_not_an_error"></a>
### `a_folder_that_is_not_a_repository_is_not_an_error`

<storycode:copy id="f-dbf85c3272ee-a_folder_that_is_not_a_repository_is_not_an_error">
#[test]
fn a_folder_that_is_not_a_repository_is_not_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    assert!(
        hick(state.path())
            .arg("repair")
            .arg(dir.path())
            .output()
            .unwrap()
            .status
            .success()
    );
}

</storycode:copy>
<a id="f-dbf85c3272ee-hick_init_installs_the_repair_ahead_of_the_drift_gate"></a>
### `hick_init_installs_the_repair_ahead_of_the_drift_gate`

<storycode:copy id="f-dbf85c3272ee-hick_init_installs_the_repair_ahead_of_the_drift_gate">
#[test]
fn hick_init_installs_the_repair_ahead_of_the_drift_gate() {
    let dir = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    repo(dir.path());
    assert!(
        hick(state.path())
            .arg("init")
            .arg(dir.path())
            .output()
            .unwrap()
            .status
            .success()
    );
    let hook = std::fs::read_to_string(dir.path().join(".git/hooks/pre-commit")).unwrap();
    assert!(hook.contains("hick repair"), "{hook}");
    // `|| true`: it must never take the drift gate down with it.
    assert!(hook.contains("hick repair || true"), "{hook}");
    let repair_at = hook.find("hick repair").unwrap();
    let test_at = hook.find("hick test").unwrap();
    assert!(repair_at < test_at, "the repair runs first");
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/continuity_repair.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/continuity_repair.rs"><storycode:paste select="#f-dbf85c3272ee-hick"/><storycode:paste select="#f-dbf85c3272ee-git"/><storycode:paste select="#f-dbf85c3272ee-BEFORE"/><storycode:paste select="#f-dbf85c3272ee-MOVED"/><storycode:paste select="#f-dbf85c3272ee-repo"/><storycode:paste select="#f-dbf85c3272ee-journal"/><storycode:paste select="#f-dbf85c3272ee-an_unwatched_move_is_recorded_at_commit_time"/><storycode:paste select="#f-dbf85c3272ee-the_repair_never_blocks_a_commit"/><storycode:paste select="#f-dbf85c3272ee-with_continuity_off_the_repair_does_nothing_at_all"/><storycode:paste select="#f-dbf85c3272ee-a_folder_that_is_not_a_repository_is_not_an_error"/><storycode:paste select="#f-dbf85c3272ee-hick_init_installs_the_repair_ahead_of_the_drift_gate"/></storycode:file>

## Implementation review

### Implementation

- [crates/hickory-cli/src/continuity.rs](../../../mechanisms/crates/hickory-cli/src/continuity.rs.md#output-09a2939626d6)
- [crates/hickory-cli/src/init.rs](../../../mechanisms/crates/hickory-cli/src/init.rs.md#output-a329ec22d0b8)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)

### Tests and fixtures

- [crates/hickory-cli/tests/continuity_repair.rs](contract.md#output-dbf85c3272ee)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
