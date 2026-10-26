<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-lang/tests/project_debug.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-3d2aa0516f40"></a>
## `crates/hick-lang/tests/project_debug.rs`

Turn declared document constructions into immutable project trees, retaining byte origins so generated edits can return to their source.

Related contracts: [Debug the latest atomic project that still contains the displayed bytes](../../../../guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code/contract.md), [Launch profiles choose targets within the viewed project](../../../../guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project/contract.md).

<a id="f-3d2aa0516f40-patch"></a>
### `patch`

<storycode:copy id="f-3d2aa0516f40-patch">
use hick_lang::{ProjectTree, evaluate_project, latest_debug_project, unified_transition};
use std::collections::BTreeMap;

fn patch(id: &str, after: &str, before: &str, next: &str) -> String {
    let tree = |text: &str| {
        let mut tree = ProjectTree::default();
        tree.insert("app.py", hick_lang::ProjectEntry::file(text.as_bytes()))
            .unwrap();
        tree
    };
    format!(
        "<hick:patch id=\"{id}\" after=\"{after}\">\n{}</hick:patch>",
        unified_transition(&tree(before), &tree(next)).unwrap()
    )
}
</storycode:copy>
<a id="f-3d2aa0516f40-fixture"></a>
### `fixture`

<storycode:copy id="f-3d2aa0516f40-fixture">
fn fixture() -> hick_lang::EvaluatedProject {
    // Dependency order deliberately differs from alphabetical identity order.
    let source = format!(
        "<hick:project/><hick:file id=\"zbase\" path=\"app.py\">print(1)\n</hick:file><hick:file id=\"alibrary\" after=\"zbase\" path=\"lib.py\">value = 2\n</hick:file>{}<hick:file id=\"config\" after=\"replacement\" path=\"config.json\">{{}}\n</hick:file>",
        patch("replacement", "alibrary", "print(1)\n", "print(2)\n")
    );
    evaluate_project(
        "project.md",
        &BTreeMap::from([("project.md".into(), source)]),
        &ProjectTree::default(),
    )
    .unwrap()
}
</storycode:copy>
[Read the test `latest_compatible_version_preserves_later_other_files_without_replacing_displayed_code` beside its requirement](../../../../guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code/contract.md#f-3d2aa0516f40-latest_compatible_version_preserves_later_other_files_without_replacing_displayed_code).
[Read the test `unsaved_or_absent_code_is_refused_instead_of_debugging_different_bytes` beside its requirement](../../../../guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code/contract.md#f-3d2aa0516f40-unsaved_or_absent_code_is_refused_instead_of_debugging_different_bytes).
[Read the test `byte_identical_revert_uses_latest_state_even_when_provenance_changed` beside its requirement](../../../../guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code/contract.md#f-3d2aa0516f40-byte_identical_revert_uses_latest_state_even_when_provenance_changed).
[Read the test `unchanged_rename_follows_provenance_but_equal_unrelated_files_do_not` beside its requirement](../../../../guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code/contract.md#f-3d2aa0516f40-unchanged_rename_follows_provenance_but_equal_unrelated_files_do_not).
[Read the test `a_mode_change_does_not_make_identical_code_older` beside its requirement](../../../../guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code/contract.md#f-3d2aa0516f40-a_mode_change_does_not_make_identical_code_older).
[Read the test `equal_text_in_an_unrelated_file_is_not_used_as_a_rename` beside its requirement](../../../../guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code/contract.md#f-3d2aa0516f40-equal_text_in_an_unrelated_file_is_not_used_as_a_rename).
[Read the test `atomic_scope_never_exposes_a_new_caller_with_the_old_library` beside its requirement](../../../../guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project/contract.md#f-3d2aa0516f40-atomic_scope_never_exposes_a_new_caller_with_the_old_library).

### Assemble `crates/hick-lang/tests/project_debug.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lang/tests/project_debug.rs"><storycode:paste select="#f-3d2aa0516f40-patch"/><storycode:paste select="#f-3d2aa0516f40-fixture"/><storycode:paste select="#f-3d2aa0516f40-latest_compatible_version_preserves_later_other_files_without_replacing_displayed_code"/><storycode:paste select="#f-3d2aa0516f40-unsaved_or_absent_code_is_refused_instead_of_debugging_different_bytes"/><storycode:paste select="#f-3d2aa0516f40-byte_identical_revert_uses_latest_state_even_when_provenance_changed"/><storycode:paste select="#f-3d2aa0516f40-unchanged_rename_follows_provenance_but_equal_unrelated_files_do_not"/><storycode:paste select="#f-3d2aa0516f40-a_mode_change_does_not_make_identical_code_older"/><storycode:paste select="#f-3d2aa0516f40-equal_text_in_an_unrelated_file_is_not_used_as_a_rename"/><storycode:paste select="#f-3d2aa0516f40-atomic_scope_never_exposes_a_new_caller_with_the_old_library"/></storycode:file>

</storycode:doc>
