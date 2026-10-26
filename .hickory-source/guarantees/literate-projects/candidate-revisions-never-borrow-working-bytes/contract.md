<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Candidate revision checks never borrow working bytes

For: Maintainers reorganizing source while preserving the complete generated program.

Given an exact Git commit/tree or complete stage-zero index, when its literate
project is verified, then both Markdown and expected outputs come from the
captured revision object identities. Unstaged files and ancestor documents
cannot complete a partial candidate. Unmerged index stages and unsupported Git
entry kinds fail. Persisted CRDT input decodes the supplied immutable updates;
subsequent live edits do not change the captured tree.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/literate-projects/candidate-revisions-never-borrow-working-bytes.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Implementation

## Test the behavior: `index_partial_changes_cannot_borrow_unstaged_source_or_outputs`

This is the maintained test fragment from `crates/hickory-cli/tests/project_revision.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-e0a4f3a47871-index_partial_changes_cannot_borrow_unstaged_source_or_outputs"></a>
### `index_partial_changes_cannot_borrow_unstaged_source_or_outputs`

<storycode:copy id="f-e0a4f3a47871-index_partial_changes_cannot_borrow_unstaged_source_or_outputs">
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
</storycode:copy>

## Test the behavior: `persisted_crdt_snapshots_are_frozen_not_live_rooms`

This is the maintained test fragment from `crates/hickory-cli/tests/project_revision.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-e0a4f3a47871-persisted_crdt_snapshots_are_frozen_not_live_rooms"></a>
### `persisted_crdt_snapshots_are_frozen_not_live_rooms`

<storycode:copy id="f-e0a4f3a47871-persisted_crdt_snapshots_are_frozen_not_live_rooms">
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

</storycode:copy>

## Test the behavior: `mercurial_revision_uses_the_shared_tree_and_exact_committed_bytes`

This is the maintained test fragment from `crates/hickory-cli/tests/project_revision.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-e0a4f3a47871-mercurial_revision_uses_the_shared_tree_and_exact_committed_bytes"></a>
### `mercurial_revision_uses_the_shared_tree_and_exact_committed_bytes`

<storycode:copy id="f-e0a4f3a47871-mercurial_revision_uses_the_shared_tree_and_exact_committed_bytes">
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
</storycode:copy>

## Test the behavior: `candidate_git_scope_is_the_chosen_project_directory`

This is the maintained test fragment from `crates/hickory-cli/tests/project_revision.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-e0a4f3a47871-candidate_git_scope_is_the_chosen_project_directory"></a>
### `candidate_git_scope_is_the_chosen_project_directory`

<storycode:copy id="f-e0a4f3a47871-candidate_git_scope_is_the_chosen_project_directory">
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
</storycode:copy>

## Test the behavior: `automatic_candidate_selection_checks_scoped_projects_and_unexplained_outputs`

This is the maintained test fragment from `crates/hickory-cli/tests/project_revision.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-e0a4f3a47871-automatic_candidate_selection_checks_scoped_projects_and_unexplained_outputs"></a>
### `automatic_candidate_selection_checks_scoped_projects_and_unexplained_outputs`

<storycode:copy id="f-e0a4f3a47871-automatic_candidate_selection_checks_scoped_projects_and_unexplained_outputs">
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

</storycode:copy>

## Test the behavior: `automatic_index_check_and_hook_list_ignore_dirty_project_working_bytes`

This is the maintained test fragment from `crates/hickory-cli/tests/project_revision.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-e0a4f3a47871-automatic_index_check_and_hook_list_ignore_dirty_project_working_bytes"></a>
### `automatic_index_check_and_hook_list_ignore_dirty_project_working_bytes`

<storycode:copy id="f-e0a4f3a47871-automatic_index_check_and_hook_list_ignore_dirty_project_working_bytes">
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
</storycode:copy>

## Implementation review

### Implementation

- [crates/hickory-cli/src/init_hook.sh](../../../mechanisms/crates/hickory-cli/src/init_hook.sh.md#output-cd84f6b6cd98)
- [crates/hickory-cli/src/project_command.rs](../../../mechanisms/crates/hickory-cli/src/project_command.rs.md#output-3d5623416448)
- [crates/hickory-cli/src/project_index.rs](../../../mechanisms/crates/hickory-cli/src/project_index.rs.md#output-01e9e63e945c)
- [crates/hickory-cli/src/project_revision.rs](../../../mechanisms/crates/hickory-cli/src/project_revision.rs.md#output-fb969c2bf635)

### Tests and fixtures

- [crates/hickory-cli/tests/project_revision.rs](../../../mechanisms/crates/hickory-cli/tests/project_revision.rs.md#output-e0a4f3a47871)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
