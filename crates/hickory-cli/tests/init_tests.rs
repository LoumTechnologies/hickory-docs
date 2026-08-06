//! Integration tests for `hickory init`: hook installation idempotency and
//! the pre-commit drift gate. The gate is exercised by running the installed
//! hook script directly (not via `git commit`) with the built `hickory`
//! binary on PATH.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn hickory_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_hickory"))
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?} failed: {out:?}");
}

fn init_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["config", "user.email", "test@example.com"]);
    git(dir.path(), &["config", "user.name", "Test"]);
    dir
}

fn run_init(dir: &Path) -> Output {
    let out = Command::new(hickory_bin())
        .arg("init")
        .arg(dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "hickory init failed: {out:?}");
    out
}

/// Run the installed pre-commit hook script directly, from the repo root,
/// with the directory containing the built `hickory` binary prepended to
/// PATH (as it would be for a user who `cargo install`ed it).
fn run_hook(repo: &Path) -> Output {
    let hook = repo.join(".git/hooks/pre-commit");
    assert!(hook.exists(), "hook not installed at {}", hook.display());
    let bin_dir = hickory_bin().parent().unwrap().to_path_buf();
    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    Command::new("sh")
        .arg(hook)
        .current_dir(repo)
        .env("PATH", path)
        .output()
        .unwrap()
}

const PASSING_DOC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="passing.md">
# Passing

<hick:container name="c" image="alpine:3.20" />

<hick:exec container="c">
printf 'one\ntwo\n'
<hick:expect match="exact">one
two
</hick:expect>
</hick:exec>
</hick:doc>
"#;

const DRIFTED_DOC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="drifted.md">
# Drifted

<hick:container name="c" image="alpine:3.20" />

<hick:exec container="c">
printf 'one\ntwo\n'
<hick:expect match="exact">one
three
</hick:expect>
</hick:exec>
</hick:doc>
"#;

#[test]
fn init_installs_hook_idempotently() {
    let repo = init_repo();
    run_init(repo.path());
    let hook_path = repo.path().join(".git/hooks/pre-commit");
    let first = std::fs::read_to_string(&hook_path).unwrap();
    assert!(first.contains("### HICKORY ###"));
    assert!(first.contains("### END HICKORY ###"));

    run_init(repo.path());
    let second = std::fs::read_to_string(&hook_path).unwrap();
    assert_eq!(first, second, "second init changed the hook");
    assert_eq!(second.matches("### HICKORY ###").count(), 1);
}

#[test]
fn hook_passes_with_no_hick_docs() {
    let repo = init_repo();
    run_init(repo.path());
    let out = run_hook(repo.path());
    assert!(
        out.status.success(),
        "hook failed in a repo with no .hick docs: {out:?}"
    );
}

#[test]
fn hook_passes_with_clean_doc_and_fails_on_drift() {
    let repo = init_repo();
    run_init(repo.path());

    // A tracked, passing doc whose outputs are committed: hook succeeds.
    std::fs::write(repo.path().join("passing.hick"), PASSING_DOC).unwrap();
    let out = Command::new(hickory_bin())
        .arg("run")
        .arg(repo.path().join("passing.hick"))
        .output()
        .unwrap();
    assert!(out.status.success(), "hickory run failed: {out:?}");
    git(repo.path(), &["add", "."]);
    let out = run_hook(repo.path());
    assert!(
        out.status.success(),
        "hook failed on a clean doc: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Add a tracked drifted doc: hook must fail and say why.
    std::fs::write(repo.path().join("drifted.hick"), DRIFTED_DOC).unwrap();
    git(repo.path(), &["add", "drifted.hick"]);
    let out = run_hook(repo.path());
    assert!(
        !out.status.success(),
        "hook passed despite drift: stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("documentation drift"),
        "missing drift message: {stderr}"
    );
}

#[test]
fn drifted_doc_blocks_commit_via_hook_script() {
    // The full gate: a repo with a drifted tracked doc must fail the
    // pre-commit hook exactly as `git commit` would invoke it.
    let repo = init_repo();
    run_init(repo.path());
    std::fs::write(repo.path().join("drifted.hick"), DRIFTED_DOC).unwrap();
    git(repo.path(), &["add", "."]);
    let out = run_hook(repo.path());
    assert!(!out.status.success(), "drifted repo passed the hook: {out:?}");
}
