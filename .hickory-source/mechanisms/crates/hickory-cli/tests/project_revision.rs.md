<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/project_revision.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-e0a4f3a47871"></a>
## `crates/hickory-cli/tests/project_revision.rs`

Related contracts: [Candidate revision checks never borrow working bytes](../../../../guarantees/literate-projects/candidate-revisions-never-borrow-working-bytes/contract.md), [Rebuild the program from its Markdown source](../../../../guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md).

<a id="f-e0a4f3a47871-git"></a>
### `git`

<storycode:copy id="f-e0a4f3a47871-git">
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
</storycode:copy>
<a id="f-e0a4f3a47871-checked"></a>
### `checked`

<storycode:copy id="f-e0a4f3a47871-checked">
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
</storycode:copy>
[Read the test `index_partial_changes_cannot_borrow_unstaged_source_or_outputs` beside its requirement](../../../../guarantees/literate-projects/candidate-revisions-never-borrow-working-bytes/contract.md#f-e0a4f3a47871-index_partial_changes_cannot_borrow_unstaged_source_or_outputs).
[Read the test `fresh_git_free_folder_runs_the_markdown_closure` beside its requirement](../../../../guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md#f-e0a4f3a47871-fresh_git_free_folder_runs_the_markdown_closure).
[Read the test `persisted_crdt_snapshots_are_frozen_not_live_rooms` beside its requirement](../../../../guarantees/literate-projects/candidate-revisions-never-borrow-working-bytes/contract.md#f-e0a4f3a47871-persisted_crdt_snapshots_are_frozen_not_live_rooms).
[Read the test `mercurial_revision_uses_the_shared_tree_and_exact_committed_bytes` beside its requirement](../../../../guarantees/literate-projects/candidate-revisions-never-borrow-working-bytes/contract.md#f-e0a4f3a47871-mercurial_revision_uses_the_shared_tree_and_exact_committed_bytes).
[Read the test `candidate_git_scope_is_the_chosen_project_directory` beside its requirement](../../../../guarantees/literate-projects/candidate-revisions-never-borrow-working-bytes/contract.md#f-e0a4f3a47871-candidate_git_scope_is_the_chosen_project_directory).
[Read the test `automatic_candidate_selection_checks_scoped_projects_and_unexplained_outputs` beside its requirement](../../../../guarantees/literate-projects/candidate-revisions-never-borrow-working-bytes/contract.md#f-e0a4f3a47871-automatic_candidate_selection_checks_scoped_projects_and_unexplained_outputs).
[Read the test `automatic_index_check_and_hook_list_ignore_dirty_project_working_bytes` beside its requirement](../../../../guarantees/literate-projects/candidate-revisions-never-borrow-working-bytes/contract.md#f-e0a4f3a47871-automatic_index_check_and_hook_list_ignore_dirty_project_working_bytes).

### Assemble `crates/hickory-cli/tests/project_revision.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/project_revision.rs"><storycode:paste select="#f-e0a4f3a47871-git"/><storycode:paste select="#f-e0a4f3a47871-checked"/><storycode:paste select="#f-e0a4f3a47871-index_partial_changes_cannot_borrow_unstaged_source_or_outputs"/><storycode:paste select="#f-e0a4f3a47871-fresh_git_free_folder_runs_the_markdown_closure"/><storycode:paste select="#f-e0a4f3a47871-persisted_crdt_snapshots_are_frozen_not_live_rooms"/><storycode:paste select="#f-e0a4f3a47871-mercurial_revision_uses_the_shared_tree_and_exact_committed_bytes"/><storycode:paste select="#f-e0a4f3a47871-candidate_git_scope_is_the_chosen_project_directory"/><storycode:paste select="#f-e0a4f3a47871-automatic_candidate_selection_checks_scoped_projects_and_unexplained_outputs"/><storycode:paste select="#f-e0a4f3a47871-automatic_index_check_and_hook_list_ignore_dirty_project_working_bytes"/><storycode:paste select="#output-test-cli-concept-incorporation"/></storycode:file>

</storycode:doc>
