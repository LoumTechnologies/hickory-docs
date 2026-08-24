//! Re-ingesting a scaffold is a three-way merge with a real base.
//!
//! Protects `docs/guarantees/authoring/a-re-ingest-is-a-three-way-merge.md`.
//!
//! The piece worth reading the test for is where the BASE comes from. The
//! document holds *ours* and records the run's `sha256`, but the original
//! bytes are gone — your four lines overwrote them, and a hash verifies
//! rather than reconstructs. Git holds them: the base is this document at the
//! commit that introduced that fingerprint. That is
//! `expression-and-log.md`'s division of labour applied to somebody else's
//! bytes — the document describes the present, git holds the past.

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
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

fn repo(dir: &Path) {
    git(dir, &["init", "-q", "-b", "master"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "T"]);
    git(dir, &["config", "commit.gpgsign", "false"]);
    std::fs::write(dir.join(".gitignore"), "obj/\n").unwrap();
}

/// Commit without the drift gate: these tests are about the merge, and the
/// documents here are mid-edit by construction.
fn commit(dir: &Path, message: &str) {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "--no-verify", "-m", message]);
}

/// A cell that writes `main.txt` and `lib.txt` with the given bodies.
fn doc_source(main: &str, lib: &str) -> String {
    let write = format!(
        "mkdir -p out && printf '{main}\\n' > out/main.txt && printf '{lib}\\n' > out/lib.txt"
    );
    format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:container name="sdk" />
<hick:volume name="project" output="app" />

<hick:exec container="sdk" mount="project:out">
<hick:copy id="scaffold">
{write}
</hick:copy>
</hick:exec>
</hick:doc>
"##
    )
}

fn ingest(dir: &Path, doc: &Path) -> std::process::Output {
    hick()
        .current_dir(dir)
        .args(["ingest", "--from", "#scaffold"])
        .arg(doc)
        .output()
        .expect("run hick")
}

/// Set the scaffolder up so it produces `main`/`lib` on the next run, without
/// touching the ingested block.
fn set_run_output(doc: &Path, main: &str, lib: &str) {
    let source = std::fs::read_to_string(doc).unwrap();
    let start = source.find("mkdir -p out").unwrap();
    let end = source[start..].find('\n').unwrap() + start;
    let next = format!(
        "mkdir -p out && printf '{main}\\n' > out/main.txt && printf '{lib}\\n' > out/lib.txt"
    );
    let mut updated = source.clone();
    updated.replace_range(start..end, &next);
    std::fs::write(doc, updated).unwrap();
}

/// Edit one ingested file's body inside the document — an ordinary edit to
/// ordinary document bytes, which is the whole point of ingest.
fn edit_ingested(doc: &Path, path: &str, from: &str, to: &str) {
    let source = std::fs::read_to_string(doc).unwrap();
    let marker = format!("<hick:file path=\"app/{path}\">");
    let at = source.find(&marker).expect("the file block") + marker.len();
    let end = source[at..].find("</hick:file>").unwrap() + at;
    let body = &source[at..end];
    assert!(body.contains(from), "expected {from:?} in {body:?}");
    let next = body.replace(from, to);
    let mut updated = source.clone();
    updated.replace_range(at..end, &next);
    std::fs::write(doc, updated).unwrap();
}

fn ingested_body(doc: &Path, path: &str) -> String {
    let source = std::fs::read_to_string(doc).unwrap();
    let marker = format!("<hick:file path=\"app/{path}\">");
    let at = source.find(&marker).expect("the file block") + marker.len();
    let end = source[at..].find("</hick:file>").unwrap() + at;
    source[at..end].to_string()
}

#[test]
fn a_re_ingest_keeps_your_edit_and_takes_the_runs_change() {
    // The case the whole design exists for: the SDK changed one file, you
    // changed another, and neither loses.
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path());
    let doc = dir.path().join("app.hick");
    std::fs::write(&doc, doc_source("hello", "lib-v1")).unwrap();

    let out = ingest(dir.path(), &doc);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    // The ingest is one commit; your four lines are the next. That ordering
    // is what makes the base recoverable.
    commit(dir.path(), "ingest the scaffold");

    edit_ingested(&doc, "main.txt", "hello", "hello, mine");
    commit(dir.path(), "my four lines");

    // A newer SDK: lib.txt changed, main.txt did not.
    set_run_output(&doc, "hello", "lib-v2");
    let out = ingest(dir.path(), &doc);
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        out.status.success(),
        "{}\n{stdout}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert_eq!(ingested_body(&doc, "main.txt").trim(), "hello, mine");
    assert_eq!(ingested_body(&doc, "lib.txt").trim(), "lib-v2");
    assert!(
        stdout.contains("Merged against the ingest committed in"),
        "{stdout}"
    );
    // One `<hick:ingested>`, not two: a cell must not claim two runs made it.
    let source = std::fs::read_to_string(&doc).unwrap();
    assert_eq!(source.matches("<hick:ingested").count(), 1, "{source}");
}

#[test]
fn a_re_ingest_with_no_committed_base_says_so_rather_than_inventing_one() {
    // A hash records WHICH run it was; it does not reconstruct the bytes.
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path());
    let doc = dir.path().join("app.hick");
    std::fs::write(&doc, doc_source("hello", "lib-v1")).unwrap();
    assert!(ingest(dir.path(), &doc).status.success());
    // Deliberately NOT committed.

    let before = std::fs::read_to_string(&doc).unwrap();
    let out = ingest(dir.path(), &doc);
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(!out.status.success(), "{stderr}");
    assert!(stderr.contains("no BASE"), "{stderr}");
    assert!(
        stderr.contains("commit the existing ingest first"),
        "{stderr}"
    );
    assert_eq!(std::fs::read_to_string(&doc).unwrap(), before);
}

#[test]
fn a_file_the_run_stopped_producing_is_kept_when_you_had_changed_it() {
    // Deleting somebody's edit because a scaffolder changed its mind is not a
    // decision a tool gets to make.
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path());
    let doc = dir.path().join("app.hick");
    std::fs::write(&doc, doc_source("hello", "lib-v1")).unwrap();
    assert!(ingest(dir.path(), &doc).status.success());
    commit(dir.path(), "ingest");

    edit_ingested(&doc, "lib.txt", "lib-v1", "lib-mine");
    commit(dir.path(), "my edit to lib");

    // The new SDK stops emitting lib.txt entirely.
    let source = std::fs::read_to_string(&doc).unwrap();
    let start = source.find("mkdir -p out").unwrap();
    let end = source[start..].find('\n').unwrap() + start;
    let mut updated = source.clone();
    updated.replace_range(
        start..end,
        "mkdir -p out && printf 'hello\\n' > out/main.txt",
    );
    std::fs::write(&doc, updated).unwrap();

    let out = ingest(dir.path(), &doc);
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(out.status.success(), "{stdout}");
    assert_eq!(ingested_body(&doc, "lib.txt").trim(), "lib-mine");
    assert!(stdout.contains("you HAD changed them, so kept"), "{stdout}");
}

#[test]
fn a_file_the_run_stopped_producing_is_removed_when_you_had_not() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path());
    let doc = dir.path().join("app.hick");
    std::fs::write(&doc, doc_source("hello", "lib-v1")).unwrap();
    assert!(ingest(dir.path(), &doc).status.success());
    commit(dir.path(), "ingest");

    let source = std::fs::read_to_string(&doc).unwrap();
    let start = source.find("mkdir -p out").unwrap();
    let end = source[start..].find('\n').unwrap() + start;
    let mut updated = source.clone();
    updated.replace_range(
        start..end,
        "mkdir -p out && printf 'hello\\n' > out/main.txt",
    );
    std::fs::write(&doc, updated).unwrap();

    let out = ingest(dir.path(), &doc);
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(out.status.success(), "{stdout}");
    let source = std::fs::read_to_string(&doc).unwrap();
    assert!(!source.contains("path=\"app/lib.txt\""), "{source}");
    assert!(stdout.contains("so removed"), "{stdout}");
}

#[test]
fn a_file_the_run_introduces_is_added() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path());
    let doc = dir.path().join("app.hick");
    std::fs::write(&doc, doc_source("hello", "lib-v1")).unwrap();
    assert!(ingest(dir.path(), &doc).status.success());
    commit(dir.path(), "ingest");

    let source = std::fs::read_to_string(&doc).unwrap();
    let start = source.find("mkdir -p out").unwrap();
    let end = source[start..].find('\n').unwrap() + start;
    let mut updated = source.clone();
    updated.replace_range(
        start..end,
        "mkdir -p out && printf 'hello\\n' > out/main.txt && printf 'lib-v1\\n' > out/lib.txt && printf 'new\\n' > out/extra.txt",
    );
    std::fs::write(&doc, updated).unwrap();

    let out = ingest(dir.path(), &doc);
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(out.status.success(), "{stdout}");
    assert_eq!(ingested_body(&doc, "extra.txt").trim(), "new");
    assert!(stdout.contains("added by this run"), "{stdout}");
}

#[test]
fn both_sides_changing_one_place_conflicts_in_the_document_and_exits_nonzero() {
    // A scaffolder randomises things, so some conflicts here are noise rather
    // than disagreement — which is why the report says so, and why volatile
    // regions are a later step designed against real false conflicts rather
    // than imagined ones.
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path());
    let doc = dir.path().join("app.hick");
    std::fs::write(&doc, doc_source("hello", "lib-v1")).unwrap();
    assert!(ingest(dir.path(), &doc).status.success());
    commit(dir.path(), "ingest");

    edit_ingested(&doc, "main.txt", "hello", "mine");
    commit(dir.path(), "my edit");
    set_run_output(&doc, "theirs", "lib-v1");

    let out = ingest(dir.path(), &doc);
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(!out.status.success(), "{stdout}");
    assert!(stdout.contains("CONFLICT app/main.txt"), "{stdout}");
    assert!(stdout.contains("randomises"), "{stdout}");
    // The markers are IN the document, where the resolution belongs.
    let body = ingested_body(&doc, "main.txt");
    assert!(body.contains("<<<<<<<"), "{body}");
    assert!(body.contains("this run"), "{body}");
}

#[test]
fn nothing_is_recorded_unless_continuity_is_on() {
    // No continuity, no journal. The switch is the whole feature, not just
    // its drawing.
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path());
    let doc = dir.path().join("app.hick");
    std::fs::write(&doc, doc_source("hello", "lib-v1")).unwrap();
    assert!(ingest(dir.path(), &doc).status.success());
    commit(dir.path(), "ingest");
    set_run_output(&doc, "hello", "lib-v2");
    let out = ingest(dir.path(), &doc);
    assert!(out.status.success());
    assert!(!dir.path().join(".hick-journal").exists());
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains("correspondence(s) recorded"),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn with_continuity_on_a_re_ingest_records_a_diff_precise_correspondence() {
    // A re-ingest holds all three sides at one moment, so it is a recording
    // site. Its precision is DIFF and that is not a lesser byte-precision:
    // two runs of a scaffolder share no history, so no byte-precise thread
    // exists to record even with the tool watching the whole time. This is
    // the case that forces the distinction the two coarseness categories
    // would otherwise hide.
    let dir = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    repo(dir.path());
    let doc = dir.path().join("app.hick");
    std::fs::write(&doc, doc_source("hello", "lib-v1")).unwrap();

    let with_state = |args: &[&str]| {
        hick()
            .env("HICKORY_STATE_DIR", state.path())
            .current_dir(dir.path())
            .args(args)
            .arg(&doc)
            .output()
            .expect("run hick")
    };

    assert!(
        with_state(&["ingest", "--from", "#scaffold"])
            .status
            .success()
    );
    commit(dir.path(), "ingest");

    // Turn continuity on for this project, for this user.
    let store = hickory_workspace::WorkspaceStore::under(state.path(), dir.path()).unwrap();
    assert!(!store.continuity(), "off by default");
    store.set_continuity(true).unwrap();

    set_run_output(&doc, "hello", "lib-v2");
    let out = with_state(&["ingest", "--from", "#scaffold"]);
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(out.status.success(), "{stdout}");
    assert!(stdout.contains("correspondence(s) recorded"), "{stdout}");

    let journal = dir.path().join(".hick-journal/correspondences.jsonl");
    let body = std::fs::read_to_string(&journal).expect("the journal");
    assert!(body.contains("\"precision\":\"diff\""), "{body}");
    assert!(body.contains("\"site\":\"reingest\""), "{body}");
    // Above the floor, so the address it is keyed on can still be replaced.
    assert!(body.contains("\"provisional\":true"), "{body}");
    // And the base end names the commit it was recovered from.
    assert!(body.contains("\"commit\""), "{body}");
}

#[test]
fn hick_init_keeps_the_journal_out_of_git_by_default() {
    // The journal is a RECORD, not a cache, so it MAY be committed — and that
    // one line is the only thing deciding whether CI can check anything here.
    // The default is private, matching continuity being off by default.
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path());
    assert!(
        hick()
            .arg("init")
            .arg(dir.path())
            .output()
            .unwrap()
            .status
            .success()
    );
    let ignore = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert!(ignore.contains(".hick-journal/"), "{ignore}");
    assert!(ignore.contains("sessions/"), "{ignore}");
}
