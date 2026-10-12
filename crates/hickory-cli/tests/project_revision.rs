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
