//! Replay, the publication floor, and the merge driver.
//!
//! Protects:
//!   docs/guarantees/lineage/replay-recomputes-lineage-at-a-commit.md
//!   docs/guarantees/collaboration/the-publication-floor-is-computed-and-shown.md
//!   docs/guarantees/collaboration/hick-documents-merge-through-hick.md

use std::path::Path;
use std::process::Command;

fn hick() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hick"));
    cmd.env("HICKORY_EXECUTOR", "local");
    cmd
}

fn git(dir: &Path, args: &[&str]) -> std::process::Output {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

fn repo(dir: &Path) {
    git(dir, &["init", "-q", "-b", "master"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "T"]);
    git(dir, &["config", "commit.gpgsign", "false"]);
}

fn commit(dir: &Path, message: &str) -> String {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", message]);
    String::from_utf8_lossy(&git(dir, &["rev-parse", "HEAD"]).stdout)
        .trim()
        .to_string()
}

fn doc(body: &str) -> String {
    format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="note.md">
<hick:file path="note.txt">{body}
</hick:file>
</hick:doc>
"##
    )
}

// ---------------------------------------------------------------------------
// Replay
// ---------------------------------------------------------------------------

#[test]
fn replay_reports_the_lineage_the_document_had_at_a_commit() {
    // Exact lineage is RECOMPUTED at any commit — no new data model, nothing
    // stored — by weaving that commit's document.
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path());
    let path = dir.path().join("note.md");

    std::fs::write(&path, doc("first")).unwrap();
    let first = commit(dir.path(), "first");
    std::fs::write(&path, doc("second")).unwrap();
    commit(dir.path(), "second");

    // As it stands.
    let now = hick()
        .args(["lineage"])
        .arg(&path)
        .args(["--output", "note.txt", "--json"])
        .output()
        .unwrap();
    assert!(now.status.success());

    // And as it stood, without touching the working tree.
    let then = hick()
        .args(["lineage"])
        .arg(&path)
        .args(["--output", "note.txt", "--json", "--at", &first])
        .output()
        .unwrap();
    assert!(
        then.status.success(),
        "{}",
        String::from_utf8_lossy(&then.stderr)
    );
    let spans: serde_json::Value = serde_json::from_slice(&then.stdout).expect("Provenance[] json");
    assert!(spans.as_array().is_some_and(|a| !a.is_empty()));
    // The lineage of the OLD document, not of today's.
    assert_ne!(now.stdout, then.stdout);

    // Replay reads git; it must not check anything out.
    assert_eq!(std::fs::read_to_string(&path).unwrap(), doc("second"));
    let status = git(dir.path(), &["status", "--porcelain"]);
    assert!(status.stdout.is_empty(), "the working tree was disturbed");
}

#[test]
fn history_names_the_commits_the_slider_can_stop_at() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path());
    let path = dir.path().join("note.md");
    std::fs::write(&path, doc("first")).unwrap();
    commit(dir.path(), "the first version");
    std::fs::write(&path, doc("second")).unwrap();
    commit(dir.path(), "the second version");

    let out = hick()
        .args(["lineage"])
        .arg(&path)
        .args(["--output", "note.txt", "--history"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(stdout.contains("the first version"), "{stdout}");
    assert!(stdout.contains("the second version"), "{stdout}");
}

#[test]
fn a_document_past_the_grammar_boundary_says_so_rather_than_failing_obscurely() {
    // Replay weaves an old document with today's binary, which asks for a
    // grammar-compatibility promise this product has not made. The honest
    // claim is "replay works back to the last grammar change", and the edge
    // says that instead of surfacing a parse error nobody can act on.
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path());
    let path = dir.path().join("note.md");
    // A document today's parser refuses: a namespace prefix that is declared
    // nowhere, so nothing binds `hick:`.
    std::fs::write(&path, "<hick:doc>\n<hick:file path=\"x\">y</hick:file>\n").unwrap();
    let bad = commit(dir.path(), "before the grammar settled");
    std::fs::write(&path, doc("fine")).unwrap();
    commit(dir.path(), "modern");

    let out = hick()
        .args(["lineage"])
        .arg(&path)
        .args(["--output", "note.txt", "--at", &bad])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(!out.status.success());
    assert!(
        stderr.contains("Replay works back to the last grammar change"),
        "{stderr}"
    );
    assert!(
        stderr.contains("Nothing is wrong with the commit"),
        "{stderr}"
    );
    assert!(stderr.contains("git show"), "{stderr}");
}

// ---------------------------------------------------------------------------
// The floor
// ---------------------------------------------------------------------------

#[test]
fn with_nothing_published_every_commit_is_a_draft() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path());
    std::fs::write(dir.path().join("a.txt"), "a").unwrap();
    let one = commit(dir.path(), "one");
    std::fs::write(dir.path().join("a.txt"), "b").unwrap();
    let two = commit(dir.path(), "two");

    let floor = hickory_cli::floor::compute(dir.path()).expect("a repository");
    assert!(floor.sha.is_none());
    assert!(floor.is_draft(&one) && floor.is_draft(&two));
    // "Nobody else can be holding them" — publication, never "unpushed",
    // which is about a transport.
    assert!(floor.summary.contains("drafts"), "{}", floor.summary);
}

#[test]
fn the_floor_is_the_merge_base_with_the_published_ref() {
    // Below it, commits are records. Above it is the frontier, and the
    // frontier is derived.
    let origin = tempfile::tempdir().unwrap();
    git(origin.path(), &["init", "-q", "--bare", "-b", "master"]);

    let dir = tempfile::tempdir().unwrap();
    repo(dir.path());
    std::fs::write(dir.path().join("a.txt"), "a").unwrap();
    let published = commit(dir.path(), "published");
    git(
        dir.path(),
        &["remote", "add", "origin", &origin.path().to_string_lossy()],
    );
    git(dir.path(), &["push", "-q", "-u", "origin", "master"]);

    std::fs::write(dir.path().join("a.txt"), "b").unwrap();
    let draft = commit(dir.path(), "still a draft");

    let floor = hickory_cli::floor::compute(dir.path()).expect("a repository");
    assert_eq!(floor.sha.as_deref(), Some(published.as_str()));
    assert!(floor.is_draft(&draft), "{floor:?}");
    assert!(!floor.is_draft(&published), "{floor:?}");
    assert_eq!(floor.drafts.len(), 1);
}

#[test]
fn merging_moves_the_floor_and_yesterdays_drafts_become_records() {
    // Nothing about the document changes at that moment, which is the point:
    // the mutable/immutable boundary lives in the commit graph.
    let origin = tempfile::tempdir().unwrap();
    git(origin.path(), &["init", "-q", "--bare", "-b", "master"]);
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path());
    std::fs::write(dir.path().join("a.txt"), "a").unwrap();
    commit(dir.path(), "published");
    git(
        dir.path(),
        &["remote", "add", "origin", &origin.path().to_string_lossy()],
    );
    git(dir.path(), &["push", "-q", "-u", "origin", "master"]);
    std::fs::write(dir.path().join("a.txt"), "b").unwrap();
    let was_draft = commit(dir.path(), "a draft");
    assert!(
        hickory_cli::floor::compute(dir.path())
            .unwrap()
            .is_draft(&was_draft)
    );

    git(dir.path(), &["push", "-q", "origin", "master"]);
    let after = hickory_cli::floor::compute(dir.path()).unwrap();
    assert!(!after.is_draft(&was_draft), "{after:?}");
    assert!(after.drafts.is_empty(), "{after:?}");
}

#[test]
fn a_folder_that_is_not_a_repository_has_no_floor_and_that_is_not_an_error() {
    let dir = tempfile::tempdir().unwrap();
    assert!(hickory_cli::floor::compute(dir.path()).is_none());
}

// ---------------------------------------------------------------------------
// The merge driver
// ---------------------------------------------------------------------------

#[test]
fn init_routes_hick_documents_at_the_driver_and_defines_it() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path());
    let out = hick().arg("init").arg(dir.path()).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let attrs = std::fs::read_to_string(dir.path().join(".gitattributes")).unwrap();
    assert!(attrs.contains("*.md merge=hick"), "{attrs}");

    let status = hickory_cli::merge_driver::status(dir.path());
    assert!(status.repository);
    assert!(status.attributes, "{}", status.summary);
    assert!(status.configured, "{}", status.summary);
    assert!(status.ok());

    // Idempotent: the second run changes nothing.
    assert!(
        hick()
            .arg("init")
            .arg(dir.path())
            .output()
            .unwrap()
            .status
            .success()
    );
    let attrs2 = std::fs::read_to_string(dir.path().join(".gitattributes")).unwrap();
    assert_eq!(attrs, attrs2);
}

#[test]
fn a_clone_that_never_ran_init_is_told_the_driver_is_missing() {
    // The sharp edge this check exists for: the routing is committed, the
    // driver definition cannot be, and an undefined driver makes git SILENTLY
    // fall back to its line merge.
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path());
    std::fs::write(dir.path().join(".gitattributes"), "*.md merge=hick\n").unwrap();
    commit(dir.path(), "route hick merges");

    let status = hickory_cli::merge_driver::status(dir.path());
    assert!(status.attributes);
    assert!(!status.configured);
    assert!(!status.ok());
    assert!(status.summary.contains("SILENTLY"), "{}", status.summary);
    assert!(status.summary.contains("hick init"), "{}", status.summary);
}

#[test]
fn hick_test_reports_a_missing_driver_without_changing_its_exit_code() {
    // `hick test`'s four exit codes are a contract CI scripts branch on, and
    // CI never merges. So this reports and never fails.
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path());
    std::fs::write(dir.path().join(".gitattributes"), "*.md merge=hick\n").unwrap();
    let path = dir.path().join("note.md");
    std::fs::write(&path, doc("hello")).unwrap();
    // Give the document its committed output, so `test` has nothing to
    // report but the driver.
    assert!(
        hick()
            .arg("run")
            .arg(&path)
            .output()
            .unwrap()
            .status
            .success()
    );

    let out = hick().arg("test").arg(&path).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(out.status.success(), "exit changed: {stderr}");
    assert!(stderr.contains("merge driver"), "{stderr}");
    assert!(stderr.contains("hick init"), "{stderr}");
}

#[test]
fn a_clean_line_merge_that_does_not_parse_is_reported_as_a_conflict() {
    // The one thing this driver does that git's fallback cannot: two sides
    // can each be correct and still not compose, and handing back a `.hick`
    // file nothing can read as CLEAN is worse than saying so.
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("base");
    let ours = dir.path().join("ours");
    let theirs = dir.path().join("theirs");
    std::fs::write(
        &base,
        "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\na\nb\nc\n</hick:doc>\n",
    )
    .unwrap();
    // Ours opens an element near the top; theirs closes the document early
    // near the bottom. Neither edit overlaps the other.
    std::fs::write(&ours, "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n<hick:file path=\"x\">\na\nb\nc\n</hick:doc>\n").unwrap();
    std::fs::write(
        &theirs,
        "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\na\nb\nc\nd\n</hick:doc>\n",
    )
    .unwrap();

    match hickory_cli::merge_driver::run(&base, &ours, &theirs, 7, "note.md").unwrap() {
        hickory_cli::merge_driver::MergeOutcome::Conflicted { reason } => {
            assert!(reason.contains("note.md"), "{reason}");
        }
        hickory_cli::merge_driver::MergeOutcome::Clean => {
            panic!("a document that does not parse was accepted as a clean merge")
        }
    }
}

#[test]
fn a_real_merge_of_two_documents_goes_through_hick() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path());
    let path = dir.path().join("note.md");
    std::fs::write(&path, doc("base")).unwrap();
    commit(dir.path(), "base");
    assert!(
        hick()
            .arg("run")
            .arg(&path)
            .output()
            .unwrap()
            .status
            .success()
    );
    commit(dir.path(), "outputs");
    assert!(
        hick()
            .arg("init")
            .arg(dir.path())
            .output()
            .unwrap()
            .status
            .success()
    );
    commit(dir.path(), "init");

    // `hick init` also installed the pre-commit drift gate, so each side
    // regenerates its outputs before committing — which is the workflow, not
    // a workaround.
    let run_then_commit = |message: &str| {
        assert!(
            hick()
                .arg("run")
                .arg(&path)
                .output()
                .unwrap()
                .status
                .success()
        );
        commit(dir.path(), message);
    };
    git(dir.path(), &["checkout", "-q", "-b", "side"]);
    std::fs::write(&path, doc("theirs")).unwrap();
    run_then_commit("theirs");
    git(dir.path(), &["checkout", "-q", "master"]);
    std::fs::write(dir.path().join("other.txt"), "ours").unwrap();
    run_then_commit("ours");

    let out = Command::new("git")
        .arg("-C")
        .arg(dir.path())
        .args(["merge", "--no-edit", "side"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "merge failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // The merged document still parses — which is the property the driver is
    // there to keep, whatever the merge algorithm underneath it is.
    let merged = std::fs::read_to_string(&path).unwrap();
    assert!(hick_lang::parse(&merged).is_ok(), "{merged}");
}
