//! Integration tests for `hick init`: hook installation idempotency and
//! the pre-commit drift gate. The gate is exercised by running the installed
//! hook script directly (not via `git commit`) with the built `hick`
//! binary on PATH.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn hickory_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_hick"))
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
    assert!(out.status.success(), "hick init failed: {out:?}");
    out
}

/// Run the installed pre-commit hook script directly, from the repo root,
/// with the directory containing the built `hick` binary prepended to
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

const FALSE_CLAIM_DOC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="false-claim.md">
# False claim

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
fn hook_passes_with_clean_doc_and_fails_on_a_false_claim() {
    let repo = init_repo();
    run_init(repo.path());

    // A tracked, passing doc whose outputs are committed: hook succeeds.
    std::fs::write(repo.path().join("passing.hick"), PASSING_DOC).unwrap();
    let out = Command::new(hickory_bin())
        .arg("run")
        .arg(repo.path().join("passing.hick"))
        .output()
        .unwrap();
    assert!(out.status.success(), "hick run failed: {out:?}");
    git(repo.path(), &["add", "."]);
    let out = run_hook(repo.path());
    assert!(
        out.status.success(),
        "hook failed on a clean doc: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Add a tracked doc whose <hick:expect> is false: the hook must fail
    // and name THAT outcome. This document does not drift — its committed
    // output is absent, not stale — so a blanket "documentation drift"
    // message would send the author to the wrong fix (regenerate) for a
    // failure that must never be regenerated away.
    std::fs::write(repo.path().join("false-claim.hick"), FALSE_CLAIM_DOC).unwrap();
    git(repo.path(), &["add", "false-claim.hick"]);
    let out = run_hook(repo.path());
    assert!(
        !out.status.success(),
        "hook passed despite a false claim: stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("FAILED EXPECTATION (exit 3)"),
        "hook did not name the failed-expectation outcome: {stderr}"
    );
    assert!(
        !stderr.contains("documentation drift"),
        "hook still reports every failure as drift: {stderr}"
    );
}

#[test]
fn failing_doc_blocks_commit_via_hook_script() {
    // The full gate: a repo with a tracked failing doc must fail the
    // pre-commit hook exactly as `git commit` would invoke it.
    let repo = init_repo();
    run_init(repo.path());
    std::fs::write(repo.path().join("false-claim.hick"), FALSE_CLAIM_DOC).unwrap();
    git(repo.path(), &["add", "."]);
    let out = run_hook(repo.path());
    assert!(
        !out.status.success(),
        "failing repo passed the hook: {out:?}"
    );
}

/// The hook must name DRIFT for a stale committed output, and must not
/// describe it with the failed-expectation wording. Before this, every
/// non-zero exit printed "documentation drift", which was accurate for
/// exactly this case and misleading for the other two.
#[test]
fn hook_names_drift_when_a_committed_output_is_stale() {
    let repo = init_repo();
    run_init(repo.path());

    std::fs::write(repo.path().join("passing.hick"), PASSING_DOC).unwrap();
    let out = Command::new(hickory_bin())
        .arg("run")
        .arg(repo.path().join("passing.hick"))
        .output()
        .unwrap();
    assert!(out.status.success(), "hick run failed: {out:?}");
    git(repo.path(), &["add", "."]);

    // Every expectation still holds; only the woven output is out of date.
    std::fs::write(repo.path().join("passing.md"), "stale hand-edited bytes\n").unwrap();
    let out = run_hook(repo.path());
    assert!(
        !out.status.success(),
        "hook passed despite a stale committed output: {out:?}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("found DRIFT (exit 1)"),
        "hook did not name the drift outcome: {stderr}"
    );
    assert!(
        !stderr.contains("FAILED EXPECTATION"),
        "hook confused drift with a false claim: {stderr}"
    );
}

/// `hick init` registers the MCP server for harnesses that read the
/// project's `.mcp.json` — and must never damage what is already there.
///
/// Protects docs/guarantees/agent/byo-agent-tool-surface.md.
#[test]
fn init_registers_the_mcp_server_without_clobbering_other_entries() {
    let repo = init_repo();
    let mcp_path = repo.path().join(".mcp.json");

    // A repo that already registers another server. Re-running init must be
    // additive: an init that deleted someone's server would make the command
    // unsafe to re-run, which is the one property it promises.
    std::fs::write(
        &mcp_path,
        r#"{"mcpServers":{"other":{"command":"other-server","args":[]}}}"#,
    )
    .unwrap();

    run_init(repo.path());
    let parsed: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&mcp_path).unwrap()).unwrap();
    assert_eq!(parsed["mcpServers"]["hick"]["command"], "hick");
    assert_eq!(parsed["mcpServers"]["hick"]["args"][0], "mcp");
    assert_eq!(
        parsed["mcpServers"]["other"]["command"], "other-server",
        "init destroyed an unrelated MCP server"
    );

    // Idempotent: a second run rewrites nothing.
    let first = std::fs::read_to_string(&mcp_path).unwrap();
    run_init(repo.path());
    assert_eq!(first, std::fs::read_to_string(&mcp_path).unwrap());
}

/// The managed AGENTS.md section teaches the tool surface, because an outside
/// agent that only learns "edit .hick files" will hand-edit text and lose
/// every guarantee the tools exist to provide.
#[test]
fn the_agents_section_points_coding_agents_at_the_document_tools() {
    let repo = init_repo();
    run_init(repo.path());
    let agents = std::fs::read_to_string(repo.path().join("AGENTS.md")).unwrap();
    for expected in [
        "hick doc read",
        "hick doc edit-output",
        "hick doc verify",
        "hick mcp",
        "HICKORY_SESSION",
    ] {
        assert!(
            agents.contains(expected),
            "AGENTS.md never mentions {expected}"
        );
    }
}
