use hick_lang::*;
use hickory_cli::project_revision;
use std::{collections::BTreeMap, path::Path, process::Command};
fn git(root: &Path, args: &[&str]) {
    let result = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}
fn checked(snapshot: &ProjectTree) -> Vec<ProjectDifference> {
    let source = String::from_utf8(snapshot.get("project.md").unwrap().bytes.to_vec()).unwrap();
    let project = evaluate_project(
        "project.md",
        &BTreeMap::from([("project.md".into(), source)]),
        &ProjectTree::default(),
    )
    .unwrap();
    project
        .new
        .compare(&project.inventory.select(snapshot).unwrap())
}
// Guarantee: docs/guarantees/literate-projects/candidate-revisions-never-borrow-working-bytes.md
#[test]
fn index_partial_changes_cannot_borrow_unstaged_source_or_outputs() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.name", "Test"]);
    git(root, &["config", "user.email", "test@example.invalid"]);
    let original = "<hick:project exclude=\".git/\"/><hick:file path=\"app\">a\n</hick:file>";
    std::fs::write(root.join("project.md"), original).unwrap();
    std::fs::write(root.join("app"), "a\n").unwrap();
    git(root, &["add", "project.md", "app"]);
    git(root, &["commit", "-qm", "initial"]);
    assert!(checked(&project_revision::git(root, "HEAD").unwrap()).is_empty());
    std::fs::write(root.join("project.md"), original.replace(">a\n", ">b\n")).unwrap();
    git(root, &["add", "project.md"]);
    std::fs::write(root.join("app"), "b\n").unwrap();
    assert_eq!(
        checked(&project_revision::git(root, ":index").unwrap()).len(),
        1
    );
    assert!(checked(&project_revision::git(root, "HEAD").unwrap()).is_empty());
    git(root, &["add", "app"]);
    assert!(checked(&project_revision::git(root, ":index").unwrap()).is_empty());
    std::fs::write(root.join("project.md"), "broken unstaged source").unwrap();
    assert!(checked(&project_revision::git(root, ":index").unwrap()).is_empty());
}
// Guarantee: docs/guarantees/literate-projects/markdown-constructs-the-complete-project.md
#[tokio::test]
async fn fresh_git_free_folder_runs_the_markdown_closure() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("project.md"),
        include_str!("../../hick-lang/tests/fixtures/literate-project/project.md"),
    )
    .unwrap();
    std::fs::write(
        dir.path().join("library.md"),
        include_str!("../../hick-lang/tests/fixtures/literate-project/library.md"),
    )
    .unwrap();
    let run = hickory_cli::run_doc(
        &dir.path().join("project.md"),
        &[],
        hickory_cli::RunMode::Weave,
        hickory_cli::ExecutorChoice::Local,
    )
    .await
    .unwrap();
    assert!(!dir.path().join("app.ts").exists());
    hickory_cli::write_outputs(&run, None).unwrap();
    assert!(hickory_cli::check_failures(&run, None).unwrap().is_empty());
    std::fs::write(dir.path().join("migration.sql"), "required source").unwrap();
    assert_eq!(hickory_cli::check_failures(&run, None).unwrap().len(), 1);
    assert!(!dir.path().join(".git").exists());
    assert!(!dir.path().join(".hick-cache").exists());
}
// Guarantee: docs/guarantees/literate-projects/candidate-revisions-never-borrow-working-bytes.md
#[test]
fn persisted_crdt_snapshots_are_frozen_not_live_rooms() {
    use yrs::{ReadTxn, Text, Transact};
    let doc = yrs::Doc::new();
    let text = doc.get_or_insert_text("source");
    text.insert(&mut doc.transact_mut(), 0, "🦀 source");
    let bytes = doc
        .transact()
        .encode_state_as_update_v1(&yrs::StateVector::default());
    let snapshot = project_revision::persisted_crdt(&[("project.md".into(), bytes)]).unwrap();
    text.insert(&mut doc.transact_mut(), 0, "later edit ");
    assert_eq!(
        &*snapshot.get("project.md").unwrap().bytes,
        "🦀 source".as_bytes()
    );
}

// Guarantee: docs/guarantees/literate-projects/candidate-revisions-never-borrow-working-bytes.md
#[test]
#[ignore = "requires a real Mercurial CLI; run the dedicated provider recipe"]
fn mercurial_revision_uses_the_shared_tree_and_exact_committed_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let hg = |args: &[&str]| {
        let output = Command::new("hg")
            .current_dir(root)
            .env("HGPLAIN", "1")
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    hg(&["init"]);
    let source = "<hick:project/><hick:file path=\"app\">a\n</hick:file>";
    std::fs::write(root.join("project.md"), source).unwrap();
    std::fs::write(root.join("app"), "a\n").unwrap();
    hg(&["add", "project.md", "app"]);
    hg(&["commit", "-u", "test@example.invalid", "-m", "initial"]);
    std::fs::write(root.join("project.md"), "unstaged source").unwrap();
    std::fs::write(root.join("app"), "unstaged output").unwrap();
    assert!(checked(&project_revision::mercurial(root, "tip").unwrap()).is_empty());
}
// Guarantee: docs/guarantees/literate-projects/candidate-revisions-never-borrow-working-bytes.md
#[test]
fn candidate_git_scope_is_the_chosen_project_directory() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.name", "Test"]);
    git(root, &["config", "user.email", "test@example.invalid"]);
    std::fs::create_dir(root.join("scope")).unwrap();
    std::fs::write(
        root.join("scope/project.md"),
        "<hick:project/><hick:file path=\"app\">a\n</hick:file>",
    )
    .unwrap();
    std::fs::write(root.join("scope/app"), "a\n").unwrap();
    std::fs::write(root.join("outside"), "outside project").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "scoped"]);
    assert!(checked(&project_revision::git(&root.join("scope"), "HEAD").unwrap()).is_empty());
    assert!(checked(&project_revision::git(&root.join("scope"), ":index").unwrap()).is_empty());
}
// Guarantee: docs/guarantees/literate-projects/candidate-revisions-never-borrow-working-bytes.md
#[test]
fn automatic_candidate_selection_checks_scoped_projects_and_unexplained_outputs() {
    use hick_lang::{ProjectEntry, ProjectTree};
    let mut snapshot = ProjectTree::default();
    snapshot
        .insert("unrelated.txt", ProjectEntry::file(b"outside".to_vec()))
        .unwrap();
    snapshot
        .insert(
            "app/project.md",
            ProjectEntry::file(
                b"<hick:project sources=\"part.md\"/><hick:file path=\"app.ts\">ok\n</hick:file>"
                    .to_vec(),
            ),
        )
        .unwrap();
    snapshot
        .insert(
            "app/part.md",
            ProjectEntry::file(b"<hick:project entry=\"project.md\"/>".to_vec()),
        )
        .unwrap();
    snapshot
        .insert("app/app.ts", ProjectEntry::file(b"ok\n".to_vec()))
        .unwrap();
    hickory_cli::project_index::verify_snapshot(&snapshot).unwrap();
    snapshot
        .insert("app/omitted.asset", ProjectEntry::file(vec![0, 255]))
        .unwrap();
    assert!(
        hickory_cli::project_index::verify_snapshot(&snapshot)
            .unwrap_err()
            .to_string()
            .contains("omitted.asset")
    );
}

// Guarantee: docs/guarantees/literate-projects/candidate-revisions-never-borrow-working-bytes.md
#[test]
fn automatic_index_check_and_hook_list_ignore_dirty_project_working_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git(root, &["init", "-q"]);
    std::fs::write(
        root.join("project.md"),
        "<hick:project sources=\"part.md\" exclude=\"ordinary.md\"/><hick:file path=\"app\">staged\n</hick:file>",
    )
    .unwrap();
    std::fs::write(root.join("part.md"), "<hick:project entry=\"project.md\"/>").unwrap();
    std::fs::write(root.join("ordinary.md"), "Ordinary note").unwrap();
    std::fs::write(root.join("app"), "staged\n").unwrap();
    git(root, &["add", "."]);
    std::fs::write(root.join("project.md"), "unparseable unstaged <hick:file>").unwrap();
    std::fs::write(root.join("part.md"), "unrelated dirty source").unwrap();
    std::fs::write(root.join("app"), "dirty output").unwrap();
    let list = dir.path().join(".git/ordinary-list");
    hickory_cli::project_index::verify(root, Some(&list)).unwrap();
    assert_eq!(std::fs::read(&list).unwrap(), b"ordinary.md\0");
    git(root, &["add", "app"]);
    assert!(
        hickory_cli::project_index::verify(root, None)
            .unwrap_err()
            .to_string()
            .contains("app")
    );
}
// Capture external tools' edits without choosing an existing concept as owner.
#[test]
fn cli_captures_a_multi_document_edit_as_a_new_concept_before_acceptance() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("project.md"), "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"foundation.md logging.md\"/></h:doc>").unwrap();
    let foundation = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file id=\"base\" path=\"app.py\">print('hello')\n</h:file></h:doc>";
    let logging = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:patch id=\"logging\" after=\"base\">\n--- a/app.py\n+++ b/app.py\n@@ -1 +1 @@\n-print('hello')\n+print('logged')\n</h:patch></h:doc>";
    std::fs::write(root.join("foundation.md"), foundation).unwrap();
    std::fs::write(root.join("logging.md"), logging).unwrap();
    let run = |args: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_hick"))
            .arg("project")
            .args(args)
            .arg("--root")
            .arg(root)
            .output()
            .unwrap()
    };
    let success = |args: &[&str]| {
        let output = run(args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    };
    success(&["materialize", "project.md"]);
    assert_eq!(
        std::fs::read_to_string(root.join("app.py")).unwrap(),
        "print('logged')\n"
    );
    // This is exactly what a naive editor or formatter does: write a file.
    std::fs::write(root.join("app.py"), "print('private')\n").unwrap();
    let refused = run(&["materialize", "project.md"]);
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("pending output"));
    assert_eq!(
        std::fs::read_to_string(root.join("foundation.md")).unwrap(),
        foundation
    );
    assert_eq!(
        std::fs::read_to_string(root.join("logging.md")).unwrap(),
        logging
    );
    success(&["capture", "project.md"]);
    let listed = success(&["edits", "project.md"]);
    let id = listed.split_once(':').unwrap().0;
    let patch = success(&["edits", "project.md", "--id", id]);
    assert!(patch.contains("+print('private')"));
    success(&[
        "new-patch",
        "project.md",
        "--id",
        id,
        "--document",
        "privacy.md",
        "--patch-id",
        "privacy",
        "--explanation",
        "Keep sensitive values out of logs.",
    ]);
    assert_eq!(
        std::fs::read_to_string(root.join("logging.md")).unwrap(),
        logging
    );
    assert!(
        std::fs::read_to_string(root.join("privacy.md"))
            .unwrap()
            .contains("Keep sensitive values out of logs.")
    );
    assert!(success(&["edits", "project.md"]).contains(id));
    success(&["accept", "project.md", "--id", id]);
    assert!(success(&["edits", "project.md"]).contains("No pending captures"));
    success(&["verify", "project.md"]);
    success(&["materialize", "project.md"]);
    assert_eq!(
        std::fs::read_to_string(root.join("app.py")).unwrap(),
        "print('private')\n"
    );
}
// The installed hook must inspect exactly what Git will commit, not repaired
// working files. Exercise real commits, including a custom hook's early exit.
#[test]
fn installed_project_hook_checks_staged_pairs_and_fails_closed() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.name", "Test"]);
    git(root, &["config", "user.email", "test@example.invalid"]);
    git(root, &["config", "core.hooksPath", ".custom hooks"]);
    std::fs::create_dir(root.join(".custom hooks")).unwrap();
    let hook = root.join(".custom hooks/pre-commit");
    std::fs::write(
        &hook,
        "#!/bin/sh\nprintf ran > .git/custom-hook-ran\nexit 0\n",
    )
    .unwrap();
    let project = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"part.md\" /></h:doc>";
    let source = |value: &str| {
        format!(
            "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"generated file.txt\">{value}\n</h:file></h:doc>"
        )
    };
    std::fs::write(root.join("project.md"), project).unwrap();
    std::fs::write(root.join("part.md"), source("before")).unwrap();
    std::fs::write(root.join("generated file.txt"), "before\n").unwrap();
    git(
        root,
        &["add", "project.md", "part.md", "generated file.txt"],
    );
    let install = || {
        let output = Command::new(env!("CARGO_BIN_EXE_hick"))
            .current_dir(root)
            .args(["project", "install-hook"])
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
    };
    install();
    let installed = std::fs::read(&hook).unwrap();
    install();
    assert_eq!(std::fs::read(&hook).unwrap(), installed);
    assert!(!root.join("AGENTS.md").exists());
    git(root, &["commit", "-qm", "consistent before"]);
    assert!(root.join(".git/custom-hook-ran").exists());
    let commit = || {
        Command::new("git")
            .current_dir(root)
            .args(["commit", "--allow-empty", "-qm", "candidate"])
            .output()
            .unwrap()
    };
    // Worktree agrees, but staged output still says before: refuse.
    std::fs::write(root.join("part.md"), source("after")).unwrap();
    std::fs::write(root.join("generated file.txt"), "after\n").unwrap();
    git(root, &["add", "part.md"]);
    let refused = commit();
    assert!(!refused.status.success(), "{refused:?}");
    assert!(String::from_utf8_lossy(&refused.stderr).contains("Commit blocked"));
    assert_eq!(
        std::fs::read_to_string(root.join("generated file.txt")).unwrap(),
        "after\n"
    );
    git(root, &["add", "generated file.txt"]);
    git(root, &["commit", "-qm", "consistent after"]);
    // An output-only staged edit cannot borrow its repaired working bytes.
    std::fs::write(root.join("generated file.txt"), "wrong\n").unwrap();
    git(root, &["add", "generated file.txt"]);
    std::fs::write(root.join("generated file.txt"), "after\n").unwrap();
    assert!(!commit().status.success());
    git(root, &["add", "generated file.txt"]);
    // A missing staged output or source cannot borrow the still-present file.
    git(root, &["rm", "--cached", "generated file.txt"]);
    assert!(!commit().status.success());
    git(root, &["add", "generated file.txt"]);
    git(root, &["rm", "--cached", "part.md"]);
    assert!(!commit().status.success());
    git(root, &["add", "part.md"]);
    git(root, &["rm", "--cached", "project.md"]);
    assert!(!commit().status.success());
    git(root, &["add", "project.md"]);
    // Dirty unstaged source and output must not reject a consistent index.
    std::fs::write(
        root.join("project.md"),
        format!("{project}\nReviewed prose\n"),
    )
    .unwrap();
    git(root, &["add", "project.md"]);
    std::fs::write(root.join("project.md"), "broken working document <h:file>").unwrap();
    std::fs::write(root.join("generated file.txt"), "dirty working output").unwrap();
    let accepted = commit();
    assert!(accepted.status.success(), "{accepted:?}");
    assert_eq!(
        std::fs::read_to_string(root.join("generated file.txt")).unwrap(),
        "dirty working output"
    );
    // Tool removal is a blocking error, never a successful skipped check.
    let missing = root.join("no-such-verifier");
    git(
        root,
        &[
            "config",
            "hickory.projectVerifier",
            missing.to_str().unwrap(),
        ],
    );
    let refused = commit();
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("Commit blocked"));
    // Installing into another interpreter must refuse without changing it.
    let python_hook = "#!/usr/bin/env python3\nraise SystemExit(0)\n";
    std::fs::write(&hook, python_hook).unwrap();
    let refused = Command::new(env!("CARGO_BIN_EXE_hick"))
        .current_dir(root)
        .args(["project", "install-hook"])
        .output()
        .unwrap();
    assert!(!refused.status.success());
    assert_eq!(std::fs::read_to_string(&hook).unwrap(), python_hook);
}
