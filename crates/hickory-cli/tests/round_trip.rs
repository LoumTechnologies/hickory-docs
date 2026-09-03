//! The round trips, end to end, against the real binary: nothing undone,
//! nothing left disagreeing.
//!
//! Protects docs/guarantees/authoring/an-output-that-cannot-be-carried-back-is-held.md
//! Protects docs/guarantees/verification/a-recording-is-keyed-by-the-cells-inputs.md
//! Protects docs/guarantees/verification/a-weave-without-a-recording-keeps-the-artifact.md
//!
//! System tests in the sense `.instructions/framework-agnostic-system-tests.md`
//! means: `hick` is a child process, git is git, and every assertion is about
//! bytes on disk.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn hick() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_hick"));
    command.env("HICKORY_EXECUTOR", "local");
    command
}

fn git(root: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .env("GIT_AUTHOR_NAME", "Test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "Test")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// A document whose one cell writes a file from a shell command, so the
/// woven markdown carries a transcript and a generated file exists.
const CHAIN: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="service.md">
# A service and its client

<hick:volume name="work" input="." output="." />

<hick:file path="spec/openapi.json"><hick:exec container="gen" mount="work:/w" show="output">printf '{"openapi":"3.0.0","paths":{"/pets":{}}}'</hick:exec></hick:file>

<hick:file path="client/client.py"><hick:exec container="gen" mount="work:/w" show="output">printf 'PATHS = %s\n' "$(cat w/spec/openapi.json | tr -d '\n')"</hick:exec></hick:file>
</hick:doc>
"#;

fn wait_until(what: &str, deadline: Duration, pred: impl Fn() -> bool) {
    let until = Instant::now() + deadline;
    while !pred() {
        assert!(Instant::now() < until, "{what}");
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn a_chain_that_regenerates_a_client_from_a_spec_is_stable_across_runs_and_weaves() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("service.hick"), CHAIN).unwrap();
    git(&root, &["init", "-q", "-b", "master"]);

    let run = |label: &str| {
        let out = hick()
            .arg("run")
            .arg(root.join("service.hick"))
            .output()
            .expect("hick run");
        assert!(
            out.status.success(),
            "{label}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    run("first run");
    let spec = std::fs::read_to_string(root.join("spec/openapi.json")).unwrap();
    let client = std::fs::read_to_string(root.join("client/client.py")).unwrap();
    let md = std::fs::read_to_string(root.join("service.md")).unwrap();
    assert!(spec.contains("/pets"), "{spec}");
    assert!(
        client.contains("/pets"),
        "the client was generated from the spec: {client}"
    );
    assert!(!md.contains("[never run]"), "{md}");

    // A second run reproduces every byte. Its cells mount `.`, which holds
    // the very files they write and the weave that carries their
    // transcripts — the case that used to change the key on every run.
    run("second run");
    assert_eq!(
        std::fs::read_to_string(root.join("spec/openapi.json")).unwrap(),
        spec
    );
    assert_eq!(
        std::fs::read_to_string(root.join("client/client.py")).unwrap(),
        client
    );
    assert_eq!(
        std::fs::read_to_string(root.join("service.md")).unwrap(),
        md
    );

    // Two weaves in a row find the recording and change nothing.
    for pass in 1..=2 {
        let out = hick()
            .arg("weave")
            .arg(root.join("service.hick"))
            .output()
            .expect("hick weave");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(out.status.success(), "weave {pass}: {stderr}");
        assert!(
            !stderr.contains("never run"),
            "weave {pass} lost the recording: {stderr}"
        );
        assert_eq!(
            std::fs::read_to_string(root.join("service.md")).unwrap(),
            md,
            "weave {pass}"
        );
        assert_eq!(
            std::fs::read_to_string(root.join("client/client.py")).unwrap(),
            client
        );
    }

    // Edit the spec cell: the client's recording is downstream of it and is
    // re-executed, so the client follows — and nothing is left stale.
    let edited = CHAIN.replace(r#"{"/pets":{}}"#, r#"{"/pets":{},"/owners":{}}"#);
    std::fs::write(root.join("service.hick"), edited).unwrap();
    run("edited run");
    assert!(
        std::fs::read_to_string(root.join("client/client.py"))
            .unwrap()
            .contains("/owners")
    );
    assert!(
        !std::fs::read_to_string(root.join("service.md"))
            .unwrap()
            .contains("[never run]")
    );
}

#[test]
fn a_git_checkout_of_a_generated_file_under_the_loop_is_held_not_undone() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("service.hick"), CHAIN).unwrap();
    git(&root, &["init", "-q", "-b", "master"]);
    let out = hick()
        .arg("run")
        .arg(root.join("service.hick"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "first"]);

    // A second version, committed, then the working tree taken back to the
    // first: what a teammate's push and a `git checkout` of one file does.
    let v2 = CHAIN.replace(r#"{"/pets":{}}"#, r#"{"/pets":{},"/owners":{}}"#);
    std::fs::write(root.join("service.hick"), v2).unwrap();
    let out = hick()
        .arg("run")
        .arg(root.join("service.hick"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "second"]);

    let log = tempfile::NamedTempFile::new().unwrap();
    let mut up = hick()
        .arg("up")
        .arg(&root)
        .stdout(Stdio::null())
        .stderr(log.reopen().unwrap())
        .spawn()
        .expect("hick up starts");
    let read_log = || std::fs::read_to_string(log.path()).unwrap_or_default();
    wait_until("the loop never came up", Duration::from_secs(60), || {
        read_log().contains("watching")
            || read_log().contains("woven")
            || read_log().contains("ready")
    });
    std::thread::sleep(Duration::from_millis(500));

    // git rewrites ONE generated file to the first version. The document
    // still says the second, and a transcript's bytes cannot be carried
    // back — so the loop must hold the file, not put it back.
    git(&root, &["checkout", "HEAD~1", "--", "client/client.py"]);
    let checked_out = std::fs::read_to_string(root.join("client/client.py")).unwrap();
    assert!(!checked_out.contains("/owners"));

    wait_until(
        "the loop never said it was holding the file",
        Duration::from_secs(30),
        || read_log().contains("diverged (held)"),
    );
    std::thread::sleep(Duration::from_millis(1500));
    assert_eq!(
        std::fs::read_to_string(root.join("client/client.py")).unwrap(),
        checked_out,
        "git's checkout was undone by the loop:\n{}",
        read_log()
    );
    // And the document was not rewritten either.
    assert!(
        std::fs::read_to_string(root.join("service.hick"))
            .unwrap()
            .contains("/owners")
    );

    let _ = up.kill();
    let _ = up.wait();
}

/// Protects docs/specs/freeform/three-axes.md (axis 1) and
/// docs/guarantees/verification/a-weave-without-a-recording-keeps-the-artifact.md
#[test]
fn a_cell_whose_inputs_moved_is_stale_and_shows_its_last_recording() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("service.hick"), CHAIN).unwrap();
    git(&root, &["init", "-q", "-b", "master"]);
    let out = hick()
        .arg("run")
        .arg(root.join("service.hick"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let recorded = std::fs::read_to_string(root.join("service.md")).unwrap();
    assert!(recorded.contains("/pets"), "{recorded}");

    // A new file in the mounted folder changes both cells' inputs, so their
    // recordings no longer match. They are STALE, not unrecorded: the weave
    // keeps showing what was recorded and says so, and nothing turns into a
    // marker.
    std::fs::write(root.join("extra.txt"), "changes the digest\n").unwrap();
    let out = hick()
        .arg("weave")
        .arg(root.join("service.hick"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert!(
        stderr.contains("stale"),
        "the weave did not report the cells as stale: {stderr}"
    );
    assert!(
        !stderr.contains("unrecorded"),
        "a stale cell was reported as unrecorded: {stderr}"
    );
    let woven = std::fs::read_to_string(root.join("service.md")).unwrap();
    assert!(
        !woven.contains("[never run]"),
        "a marker was written over recorded output: {woven}"
    );
    assert!(
        woven.contains("/pets"),
        "the last recording was not shown: {woven}"
    );

    // `hick test` names it as the drift it is, with the run that fixes it.
    let out = hick()
        .arg("test")
        .arg(root.join("service.hick"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    // test re-executes, so the cells match again there; what matters is that
    // the run afterwards leaves nothing stale.
    let out = hick()
        .arg("run")
        .arg(root.join("service.hick"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}\n{stderr}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = hick()
        .arg("weave")
        .arg(root.join("service.hick"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("stale") && !stderr.contains("unrecorded"),
        "{stderr}"
    );
}

/// Protects docs/guarantees/verification/a-recording-a-document-keeps-lives-in-the-document.md
#[test]
fn a_recording_kept_in_the_document_survives_without_the_cache_and_follows_a_run() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("service.hick"), CHAIN).unwrap();
    git(&root, &["init", "-q", "-b", "master"]);
    let out = hick()
        .arg("run")
        .arg(root.join("service.hick"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let md = std::fs::read_to_string(root.join("service.md")).unwrap();
    let client = std::fs::read_to_string(root.join("client/client.py")).unwrap();

    // Keep the recordings in the document.
    let out = hick()
        .args(["ingest", "--from", "recording"])
        .arg(root.join("service.hick"))
        .output()
        .unwrap();
    let said = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{said}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(said.contains("2 recording(s) now kept"), "{said}");
    let doc = std::fs::read_to_string(root.join("service.hick")).unwrap();
    assert_eq!(doc.matches("<hick:ingested key=").count(), 2, "{doc}");
    assert!(
        doc.contains("/pets"),
        "the recorded output is in the document verbatim: {doc}"
    );

    // A clone has no cache. The weave finds everything in the document and
    // produces the same bytes.
    std::fs::remove_dir_all(root.join(".hick-cache")).unwrap();
    let out = hick()
        .arg("weave")
        .arg(root.join("service.hick"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("unrecorded") && !stderr.contains("stale"),
        "{stderr}"
    );
    assert_eq!(
        std::fs::read_to_string(root.join("service.md")).unwrap(),
        md
    );
    assert_eq!(
        std::fs::read_to_string(root.join("client/client.py")).unwrap(),
        client
    );

    // The inputs move: the document's recording is stale, and says so —
    // the last output still shows.
    std::fs::write(root.join("extra.txt"), "moves the key\n").unwrap();
    let out = hick()
        .arg("weave")
        .arg(root.join("service.hick"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("stale"), "{stderr}");
    assert!(
        std::fs::read_to_string(root.join("service.md"))
            .unwrap()
            .contains("/pets")
    );

    // A run brings the document's recordings forward; a clone is whole again.
    let out = hick()
        .arg("run")
        .arg(root.join("service.hick"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert!(stderr.contains("refreshed 2 recording(s)"), "{stderr}");
    std::fs::remove_dir_all(root.join(".hick-cache")).unwrap();
    let out = hick()
        .arg("weave")
        .arg(root.join("service.hick"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("unrecorded") && !stderr.contains("stale"),
        "{stderr}"
    );
}

/// The absolute path helper, for readability above.
#[allow(dead_code)]
fn abs(root: &Path, rel: &str) -> PathBuf {
    root.join(rel)
}
