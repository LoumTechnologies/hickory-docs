<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/init_tests.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7418ab26c01b"></a>
## `crates/hickory-cli/tests/init_tests.rs`

Related contracts: [Use the same document tools from an external coding agent](../../../../guarantees/agent/byo-agent-tool-surface/contract.md), [The Pre-Commit Hook `hick init` Installs Names Which Outcome Blocked The Commit](../../../../guarantees/verification/the-installed-hook-names-the-outcome/contract.md).

<a id="f-7418ab26c01b-common"></a>
### `common`

Integration tests for `hick init`: hook installation idempotency and
the pre-commit drift gate. The gate is exercised by running the installed
hook script directly (not via `git commit`) with the built `hick`
binary on PATH.
The hook is POSIX shell on every platform, because that is what git runs it
with: on Windows git invokes hooks through the `sh` that Git for Windows
ships, not through `cmd`. So `sh` is the honest driver here even on a
machine whose *cells* run through `cmd.exe /C` — and the documents below
are the half that has to change, because a cell inside one of them does get
cmd.

<storycode:copy id="f-7418ab26c01b-common">
//! Integration tests for `hick init`: hook installation idempotency and
//! the pre-commit drift gate. The gate is exercised by running the installed
//! hook script directly (not via `git commit`) with the built `hick`
//! binary on PATH.
//!
//! The hook is POSIX shell on every platform, because that is what git runs it
//! with: on Windows git invokes hooks through the `sh` that Git for Windows
//! ships, not through `cmd`. So `sh` is the honest driver here even on a
//! machine whose *cells* run through `cmd.exe /C` — and the documents below
//! are the half that has to change, because a cell inside one of them does get
//! cmd.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

mod common;
</storycode:copy>
<a id="f-7418ab26c01b-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-7418ab26c01b-use_declaration">
use common::echo_lines;

</storycode:copy>
<a id="f-7418ab26c01b-hickory_bin"></a>
### `hickory_bin`

<storycode:copy id="f-7418ab26c01b-hickory_bin">
fn hickory_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_hick"))
}

</storycode:copy>
<a id="f-7418ab26c01b-git"></a>
### `git`

<storycode:copy id="f-7418ab26c01b-git">
fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?} failed: {out:?}");
}

</storycode:copy>
<a id="f-7418ab26c01b-init_repo"></a>
### `init_repo`

<storycode:copy id="f-7418ab26c01b-init_repo">
fn init_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["config", "user.email", "test@example.com"]);
    git(dir.path(), &["config", "user.name", "Test"]);
    dir
}

</storycode:copy>
<a id="f-7418ab26c01b-run_init"></a>
### `run_init`

<storycode:copy id="f-7418ab26c01b-run_init">
fn run_init(dir: &Path) -> Output {
    let out = Command::new(hickory_bin())
        .arg("init")
        .arg(dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "hick init failed: {out:?}");
    out
}

</storycode:copy>
<a id="f-7418ab26c01b-run_hook"></a>
### `run_hook`

Run the installed pre-commit hook script directly, from the repo root,
with the directory containing the built `hick` binary prepended to
PATH (as it would be for a user who `cargo install`ed it).

<storycode:copy id="f-7418ab26c01b-run_hook">
/// Run the installed pre-commit hook script directly, from the repo root,
/// with the directory containing the built `hick` binary prepended to
/// PATH (as it would be for a user who `cargo install`ed it).
fn run_hook(repo: &Path) -> Output {
    let hook = repo.join(".git/hooks/pre-commit");
    assert!(hook.exists(), "hook not installed at {}", hook.display());
    let bin_dir = hickory_bin().parent().unwrap().to_path_buf();
    // `join_paths`, not a `:`-joined string. The separator is `;` on Windows,
    // so joining with `:` there produced one unparseable first entry: `hick`
    // would not be found, the hook would take its "hick not found on PATH;
    // skipping" branch, and every test below asserting that the hook BLOCKS a
    // commit would have been asserting against a hook that checked nothing.
    let mut entries = vec![bin_dir];
    entries.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let path = std::env::join_paths(entries).expect("a PATH the hook can read");
    Command::new("sh")
        .arg(hook)
        .current_dir(repo)
        .env("PATH", path)
        .output()
        .unwrap()
}

</storycode:copy>
<a id="f-7418ab26c01b-passing_doc"></a>
### `passing_doc`

A document that says what its cell really prints, and one that does not.
Two things here are per-platform. The cell is written with `echo_lines`,
because `printf` is not a cmd builtin and cmd sequences with `&` where `sh`
would background. And the match mode is `regex-lines` rather than `exact`:
`exact` compares byte for byte, cmd's `echo` always writes CRLF, and a
document in git is LF — so an exact expectation cannot hold on Windows for
a line-ending reason (issue #18). `regex-lines` compares line by line,
which is the claim these two documents are actually making: the first says
the cell prints `one` then `two`, the second says `one` then `three`, and
only one of them is true. Which of the four outcomes the hook reports —
what every assertion in this file is about — is unchanged by the mode.

<storycode:copy id="f-7418ab26c01b-passing_doc">
/// A document that says what its cell really prints, and one that does not.
///
/// Two things here are per-platform. The cell is written with `echo_lines`,
/// because `printf` is not a cmd builtin and cmd sequences with `&` where `sh`
/// would background. And the match mode is `regex-lines` rather than `exact`:
/// `exact` compares byte for byte, cmd's `echo` always writes CRLF, and a
/// document in git is LF — so an exact expectation cannot hold on Windows for
/// a line-ending reason (issue #18). `regex-lines` compares line by line,
/// which is the claim these two documents are actually making: the first says
/// the cell prints `one` then `two`, the second says `one` then `three`, and
/// only one of them is true. Which of the four outcomes the hook reports —
/// what every assertion in this file is about — is unchanged by the mode.
fn passing_doc() -> String {
    doc("passing", &["one", "two"])
}

</storycode:copy>
<a id="f-7418ab26c01b-false_claim_doc"></a>
### `false_claim_doc`

<storycode:copy id="f-7418ab26c01b-false_claim_doc">
fn false_claim_doc() -> String {
    doc("false-claim", &["one", "three"])
}

</storycode:copy>
<a id="f-7418ab26c01b-doc"></a>
### `doc`

<storycode:copy id="f-7418ab26c01b-doc">
fn doc(name: &str, expected: &[&str]) -> String {
    let command = echo_lines(&["one", "two"]);
    let expectation = expected.join("\n");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.mdorydocs.com/1.0" >
# {name}

<hick:file path="{name}.txt">generated bytes
</hick:file>

<hick:container name="c" image="alpine:3.20" />

<hick:exec container="c">
{command}
<hick:expect match="regex-lines">{expectation}
</hick:expect>
</hick:exec>
</hick:doc>
"#
    )
}

</storycode:copy>
<a id="f-7418ab26c01b-init_installs_hook_idempotently"></a>
### `init_installs_hook_idempotently`

<storycode:copy id="f-7418ab26c01b-init_installs_hook_idempotently">
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

</storycode:copy>
<a id="f-7418ab26c01b-hook_passes_with_no_hick_docs"></a>
### `hook_passes_with_no_hick_docs`

<storycode:copy id="f-7418ab26c01b-hook_passes_with_no_hick_docs">
#[test]
fn hook_passes_with_no_hick_docs() {
    let repo = init_repo();
    run_init(repo.path());
    let out = run_hook(repo.path());
    assert!(
        out.status.success(),
        "hook failed in a repo with no .md docs: {out:?}"
    );
}

</storycode:copy>
<a id="f-7418ab26c01b-hook_passes_with_clean_doc_and_fails_on_a_false_claim"></a>
### `hook_passes_with_clean_doc_and_fails_on_a_false_claim`

<storycode:copy id="f-7418ab26c01b-hook_passes_with_clean_doc_and_fails_on_a_false_claim">
#[test]
fn hook_passes_with_clean_doc_and_fails_on_a_false_claim() {
    let repo = init_repo();
    run_init(repo.path());

    // A tracked, passing doc whose outputs are committed: hook succeeds.
    std::fs::write(repo.path().join("passing.md"), passing_doc()).unwrap();
    let out = Command::new(hickory_bin())
        .arg("run")
        .arg(repo.path().join("passing.md"))
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
    std::fs::write(repo.path().join("false-claim.md"), false_claim_doc()).unwrap();
    git(repo.path(), &["add", "false-claim.md"]);
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

</storycode:copy>
<a id="f-7418ab26c01b-failing_doc_blocks_commit_via_hook_script"></a>
### `failing_doc_blocks_commit_via_hook_script`

<storycode:copy id="f-7418ab26c01b-failing_doc_blocks_commit_via_hook_script">
#[test]
fn failing_doc_blocks_commit_via_hook_script() {
    // The full gate: a repo with a tracked failing doc must fail the
    // pre-commit hook exactly as `git commit` would invoke it.
    let repo = init_repo();
    run_init(repo.path());
    std::fs::write(repo.path().join("false-claim.md"), false_claim_doc()).unwrap();
    git(repo.path(), &["add", "."]);
    let out = run_hook(repo.path());
    assert!(
        !out.status.success(),
        "failing repo passed the hook: {out:?}"
    );
}

</storycode:copy>
<a id="f-7418ab26c01b-hook_names_drift_when_a_committed_output_is_stale"></a>
### `hook_names_drift_when_a_committed_output_is_stale`

The hook must name DRIFT for a stale committed output, and must not
describe it with the failed-expectation wording. Before this, every
non-zero exit printed "documentation drift", which was accurate for
exactly this case and misleading for the other two.

<storycode:copy id="f-7418ab26c01b-hook_names_drift_when_a_committed_output_is_stale">
/// The hook must name DRIFT for a stale committed output, and must not
/// describe it with the failed-expectation wording. Before this, every
/// non-zero exit printed "documentation drift", which was accurate for
/// exactly this case and misleading for the other two.
#[test]
fn hook_names_drift_when_a_committed_output_is_stale() {
    let repo = init_repo();
    run_init(repo.path());

    std::fs::write(repo.path().join("passing.md"), passing_doc()).unwrap();
    let out = Command::new(hickory_bin())
        .arg("run")
        .arg(repo.path().join("passing.md"))
        .output()
        .unwrap();
    assert!(out.status.success(), "hick run failed: {out:?}");
    git(repo.path(), &["add", "."]);

    // Every expectation still holds; only the woven output is out of date.
    std::fs::write(repo.path().join("passing.txt"), "stale hand-edited bytes\n").unwrap();
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

</storycode:copy>
[Read the test `init_registers_the_mcp_server_without_clobbering_other_entries` beside its requirement](../../../../guarantees/agent/byo-agent-tool-surface/contract.md#f-7418ab26c01b-init_registers_the_mcp_server_without_clobbering_other_entries).
<a id="f-7418ab26c01b-the_agents_section_points_coding_agents_at_the_document_tools"></a>
### `the_agents_section_points_coding_agents_at_the_document_tools`

The managed AGENTS.md section teaches the tool surface, because an outside
agent that only learns "edit .md files" will hand-edit text and lose
every guarantee the tools exist to provide.

<storycode:copy id="f-7418ab26c01b-the_agents_section_points_coding_agents_at_the_document_tools">
/// The managed AGENTS.md section teaches the tool surface, because an outside
/// agent that only learns "edit .md files" will hand-edit text and lose
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

</storycode:copy>
[Read the test `hook_checks_document_names_with_spaces_and_newlines` beside its requirement](../../../../guarantees/verification/the-installed-hook-names-the-outcome/contract.md#f-7418ab26c01b-hook_checks_document_names_with_spaces_and_newlines).
[Read the test `hook_names_an_unrecorded_frozen_cell` beside its requirement](../../../../guarantees/verification/the-installed-hook-names-the-outcome/contract.md#f-7418ab26c01b-hook_names_an_unrecorded_frozen_cell).

### Assemble `crates/hickory-cli/tests/init_tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/init_tests.rs"><storycode:paste select="#f-7418ab26c01b-common"/><storycode:paste select="#f-7418ab26c01b-use_declaration"/><storycode:paste select="#f-7418ab26c01b-hickory_bin"/><storycode:paste select="#f-7418ab26c01b-git"/><storycode:paste select="#f-7418ab26c01b-init_repo"/><storycode:paste select="#f-7418ab26c01b-run_init"/><storycode:paste select="#f-7418ab26c01b-run_hook"/><storycode:paste select="#f-7418ab26c01b-passing_doc"/><storycode:paste select="#f-7418ab26c01b-false_claim_doc"/><storycode:paste select="#f-7418ab26c01b-doc"/><storycode:paste select="#f-7418ab26c01b-init_installs_hook_idempotently"/><storycode:paste select="#f-7418ab26c01b-hook_passes_with_no_hick_docs"/><storycode:paste select="#f-7418ab26c01b-hook_passes_with_clean_doc_and_fails_on_a_false_claim"/><storycode:paste select="#f-7418ab26c01b-failing_doc_blocks_commit_via_hook_script"/><storycode:paste select="#f-7418ab26c01b-hook_names_drift_when_a_committed_output_is_stale"/><storycode:paste select="#f-7418ab26c01b-init_registers_the_mcp_server_without_clobbering_other_entries"/><storycode:paste select="#f-7418ab26c01b-the_agents_section_points_coding_agents_at_the_document_tools"/><storycode:paste select="#f-7418ab26c01b-hook_checks_document_names_with_spaces_and_newlines"/><storycode:paste select="#f-7418ab26c01b-hook_names_an_unrecorded_frozen_cell"/></storycode:file>

</storycode:doc>
