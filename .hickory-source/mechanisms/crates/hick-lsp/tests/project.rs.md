<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-lsp/tests/project.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d2279aae5a8a"></a>
## `crates/hick-lsp/tests/project.rs`

Related contracts: [Debug the latest atomic project that still contains the displayed bytes](../../../../guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code/contract.md), [Keep language tools and debug launches tied to their captured project](../../../../guarantees/literate-projects/tools-select-complete-isolated-endpoints/contract.md).

<a id="f-d2279aae5a8a-fixture"></a>
### `fixture`

<storycode:copy id="f-d2279aae5a8a-fixture">
use hick_lang::*;
use hick_lsp::{ProjectEndpoint, endpoint, materialize, to_endpoint, to_source, utf16_position};
use std::collections::BTreeMap;
fn fixture() -> (BTreeMap<String, String>, EvaluatedProject) {
    let sources = BTreeMap::from([
        (
            "project.md".into(),
            include_str!("../../hick-lang/tests/fixtures/literate-project/project.md").into(),
        ),
        (
            "library.md".into(),
            include_str!("../../hick-lang/tests/fixtures/literate-project/library.md").into(),
        ),
    ]);
    let project = evaluate_project("project.md", &sources, &ProjectTree::default()).unwrap();
    (sources, project)
}
</storycode:copy>
[Read the test `services_use_completed_tree_and_separate_native_identities` beside its requirement](../../../../guarantees/literate-projects/tools-select-complete-isolated-endpoints/contract.md#f-d2279aae5a8a-services_use_completed_tree_and_separate_native_identities).
[Read the test `removed_patch_lines_cannot_bind_to_unrelated_final_code` beside its requirement](../../../../guarantees/literate-projects/tools-select-complete-isolated-endpoints/contract.md#f-d2279aae5a8a-removed_patch_lines_cannot_bind_to_unrelated_final_code).
[Read the test `native_writer_does_not_follow_existing_symlinks` beside its requirement](../../../../guarantees/literate-projects/tools-select-complete-isolated-endpoints/contract.md#f-d2279aae5a8a-native_writer_does_not_follow_existing_symlinks).
[Read the test `complete_project_positions_keep_foreign_documents_and_utf16_boundaries` beside its requirement](../../../../guarantees/literate-projects/tools-select-complete-isolated-endpoints/contract.md#f-d2279aae5a8a-complete_project_positions_keep_foreign_documents_and_utf16_boundaries).
[Read the test `captured_native_contexts_are_lazy_isolated_and_never_borrow_working_files` beside its requirement](../../../../guarantees/literate-projects/tools-select-complete-isolated-endpoints/contract.md#f-d2279aae5a8a-captured_native_contexts_are_lazy_isolated_and_never_borrow_working_files).
[Read the test `clicked_file_uses_exact_old_bytes_and_original_document_breakpoint_lines` beside its requirement](../../../../guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code/contract.md#f-d2279aae5a8a-clicked_file_uses_exact_old_bytes_and_original_document_breakpoint_lines).
[Read the test `generated_code_selection_uses_displayed_bytes_instead_of_the_previous_file_context` beside its requirement](../../../../guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code/contract.md#f-d2279aae5a8a-generated_code_selection_uses_displayed_bytes_instead_of_the_previous_file_context).

### Assemble `crates/hick-lsp/tests/project.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lsp/tests/project.rs"><storycode:paste select="#f-d2279aae5a8a-fixture"/><storycode:paste select="#f-d2279aae5a8a-services_use_completed_tree_and_separate_native_identities"/><storycode:paste select="#f-d2279aae5a8a-removed_patch_lines_cannot_bind_to_unrelated_final_code"/><storycode:paste select="#f-d2279aae5a8a-native_writer_does_not_follow_existing_symlinks"/><storycode:paste select="#f-d2279aae5a8a-complete_project_positions_keep_foreign_documents_and_utf16_boundaries"/><storycode:paste select="#f-d2279aae5a8a-captured_native_contexts_are_lazy_isolated_and_never_borrow_working_files"/><storycode:paste select="#f-d2279aae5a8a-clicked_file_uses_exact_old_bytes_and_original_document_breakpoint_lines"/><storycode:paste select="#f-d2279aae5a8a-generated_code_selection_uses_displayed_bytes_instead_of_the_previous_file_context"/></storycode:file>

</storycode:doc>
