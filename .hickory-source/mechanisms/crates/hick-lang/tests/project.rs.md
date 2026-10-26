<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-lang/tests/project.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-005c1e9344c7"></a>
## `crates/hick-lang/tests/project.rs`

Turn declared document constructions into immutable project trees, retaining byte origins so generated edits can return to their source.

Related contracts: [Literal files use native bytes](../../../../guarantees/embedding/literal-files-use-native-bytes/contract.md), [Comparison edit permissions do not establish replay evidence](../../../../guarantees/literate-projects/comparison-permissions-and-evidence-are-independent/contract.md), [Check every source file, including assets and permissions](../../../../guarantees/literate-projects/complete-inventory-is-byte-exact/contract.md), [Check the agent's transition separately from maintained-source reconstruction](../../../../guarantees/literate-projects/finalization-and-incorporation-check-distinct-equalities/contract.md), [Rebuild the program from its Markdown source](../../../../guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md), [Keep a patch edit as a draft until the whole construction still works](../../../../guarantees/literate-projects/patch-edits-revalidate-downstream-construction/contract.md), [Reject a patch that does not fit its exact input](../../../../guarantees/literate-projects/replay-is-exact-and-atomic/contract.md), [Keep language tools and debug launches tied to their captured project](../../../../guarantees/literate-projects/tools-select-complete-isolated-endpoints/contract.md).

<a id="f-005c1e9344c7-tree"></a>
### `tree`

<storycode:copy id="f-005c1e9344c7-tree">
use hick_lang::*;
use std::collections::BTreeMap;
fn tree(files: &[(&str, &[u8])]) -> ProjectTree {
    let mut tree = ProjectTree::default();
    for (path, bytes) in files {
        tree.insert(path, ProjectEntry::file(bytes.to_vec()))
            .unwrap();
    }
    tree
}
</storycode:copy>
<a id="f-005c1e9344c7-generate"></a>
### `generate`

<storycode:copy id="f-005c1e9344c7-generate">
fn generate(source: &str) -> Result<EvaluatedProject, String> {
    evaluate_project(
        "project.md",
        &BTreeMap::from([("project.md".into(), source.into())]),
        &ProjectTree::default(),
    )
}
</storycode:copy>
[Read the test `patch_preserves_bom_crlf_unicode_and_untouched_origins` beside its requirement](../../../../guarantees/literate-projects/replay-is-exact-and-atomic/contract.md#f-005c1e9344c7-patch_preserves_bom_crlf_unicode_and_untouched_origins).
[Read the test `newline_markers_add_delete_rename_and_failure_are_atomic` beside its requirement](../../../../guarantees/literate-projects/replay-is-exact-and-atomic/contract.md#f-005c1e9344c7-newline_markers_add_delete_rename_and_failure_are_atomic).
[Read the test `closure_constructs_atomic_endpoints_and_can_be_reorganized` beside its requirement](../../../../guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md#f-005c1e9344c7-closure_constructs_atomic_endpoints_and_can_be_reorganized).
[Read the test `refuses_unordered_transforms_hidden_inputs_and_processes` beside its requirement](../../../../guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md#f-005c1e9344c7-refuses_unordered_transforms_hidden_inputs_and_processes).
[Read the test `inventory_checks_binary_modes_symlinks_and_portable_paths` beside its requirement](../../../../guarantees/literate-projects/complete-inventory-is-byte-exact/contract.md#f-005c1e9344c7-inventory_checks_binary_modes_symlinks_and_portable_paths).
[Read the test `permissions_replay_and_destination_identity` beside its requirement](../../../../guarantees/literate-projects/comparison-permissions-and-evidence-are-independent/contract.md#f-005c1e9344c7-permissions_replay_and_destination_identity).
[Read the test `raw_patch_grammar_and_ordinary_diff_fence` beside its requirement](../../../../guarantees/literate-projects/replay-is-exact-and-atomic/contract.md#f-005c1e9344c7-raw_patch_grammar_and_ordinary_diff_fence).
[Read the test `final_response_uses_dirty_start_and_incorporation_rechecks_destinations` beside its requirement](../../../../guarantees/literate-projects/finalization-and-incorporation-check-distinct-equalities/contract.md#f-005c1e9344c7-final_response_uses_dirty_start_and_incorporation_rechecks_destinations).
[Read the test `patch_regeneration_preserves_order_and_reports_invalid_later_context` beside its requirement](../../../../guarantees/literate-projects/patch-edits-revalidate-downstream-construction/contract.md#f-005c1e9344c7-patch_regeneration_preserves_order_and_reports_invalid_later_context).
[Read the test `editing_a_proposal_does_not_replace_its_destination_guard` beside its requirement](../../../../guarantees/literate-projects/comparison-permissions-and-evidence-are-independent/contract.md#f-005c1e9344c7-editing_a_proposal_does_not_replace_its_destination_guard).
[Read the test `contributors_can_route_to_the_completed_project_from_markdown_alone` beside its requirement](../../../../guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md#f-005c1e9344c7-contributors_can_route_to_the_completed_project_from_markdown_alone).
<a id="f-005c1e9344c7-the_legacy_literal_subset_cannot_return_a_half_patched_program"></a>
### `the_legacy_literal_subset_cannot_return_a_half_patched_program`

- docs/guarantees/embedding/literal-files-use-native-bytes.md
- docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md

<storycode:copy id="f-005c1e9344c7-the_legacy_literal_subset_cannot_return_a_half_patched_program">
// Guarantees:
// - docs/guarantees/embedding/literal-files-use-native-bytes.md
// - docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[test]
fn the_legacy_literal_subset_cannot_return_a_half_patched_program() {
    let source =
        "<hick:project/><hick:file path=\"x\">old</hick:file><hick:patch>diff</hick:patch>";
    assert!(
        literal_files(source)
            .unwrap_err()
            .contains("complete project evaluation")
    );
}
</storycode:copy>
[Read the test `generated_paths_cannot_shadow_the_markdown_closure_on_another_platform` beside its requirement](../../../../guarantees/literate-projects/complete-inventory-is-byte-exact/contract.md#f-005c1e9344c7-generated_paths_cannot_shadow_the_markdown_closure_on_another_platform).
<a id="f-005c1e9344c7-immutable_identity_cannot_be_mislabeled_as_editable_and_invalid_origins_refuse"></a>
### `immutable_identity_cannot_be_mislabeled_as_editable_and_invalid_origins_refuse`

- docs/guarantees/literate-projects/comparison-permissions-and-evidence-are-independent.md
- docs/guarantees/literate-projects/complete-inventory-is-byte-exact.md

<storycode:copy id="f-005c1e9344c7-immutable_identity_cannot_be_mislabeled_as_editable_and_invalid_origins_refuse">
// Guarantees:
// - docs/guarantees/literate-projects/comparison-permissions-and-evidence-are-independent.md
// - docs/guarantees/literate-projects/complete-inventory-is-byte-exact.md
#[test]
fn immutable_identity_cannot_be_mislabeled_as_editable_and_invalid_origins_refuse() {
    let immutable = ComparisonSource {
        identity: RevisionIdentity::Git {
            revision: "abc".into(),
        },
        tree: ProjectTree::default(),
        writable_target: Some("fork.md".into()),
    };
    assert!(
        ProjectComparison::new(vec![immutable.clone(), immutable], 0, 1, EditPolicy::Both).is_err()
    );
    let mut t = ProjectTree::default();
    let mut entry = ProjectEntry::file(b"x".to_vec());
    entry.origins.push(ByteOrigin {
        output: (0, 2),
        document: "source.md".into(),
        source: (0, 2),
    });
    assert!(t.insert("x", entry).unwrap_err().contains("provenance"));
}
</storycode:copy>
[Read the test `local_predecessor_state_does_not_depend_on_independent_act_names` beside its requirement](../../../../guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md#f-005c1e9344c7-local_predecessor_state_does_not_depend_on_independent_act_names).
[Read the test `explicit_nested_scopes_finish_all_their_acts_without_ordering_independent_writes` beside its requirement](../../../../guarantees/literate-projects/tools-select-complete-isolated-endpoints/contract.md#f-005c1e9344c7-explicit_nested_scopes_finish_all_their_acts_without_ordering_independent_writes).
[Read the test `reverse_inserted_code_regenerates_its_patch_and_old_literal_edits_check_downstream` beside its requirement](../../../../guarantees/literate-projects/patch-edits-revalidate-downstream-construction/contract.md#f-005c1e9344c7-reverse_inserted_code_regenerates_its_patch_and_old_literal_edits_check_downstream).
[Read the test `explicit_fragment_closure_is_ordered_private_and_upstream_is_read_only` beside its requirement](../../../../guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md#f-005c1e9344c7-explicit_fragment_closure_is_ordered_private_and_upstream_is_read_only).
[Read the test `local_bytes_map_to_renamed_endpoint_by_origin_and_removed_bytes_have_no_endpoint` beside its requirement](../../../../guarantees/literate-projects/tools-select-complete-isolated-endpoints/contract.md#f-005c1e9344c7-local_bytes_map_to_renamed_endpoint_by_origin_and_removed_bytes_have_no_endpoint).

### Assemble `crates/hick-lang/tests/project.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lang/tests/project.rs"><storycode:paste select="#f-005c1e9344c7-tree"/><storycode:paste select="#f-005c1e9344c7-generate"/><storycode:paste select="#f-005c1e9344c7-patch_preserves_bom_crlf_unicode_and_untouched_origins"/><storycode:paste select="#f-005c1e9344c7-newline_markers_add_delete_rename_and_failure_are_atomic"/><storycode:paste select="#f-005c1e9344c7-closure_constructs_atomic_endpoints_and_can_be_reorganized"/><storycode:paste select="#f-005c1e9344c7-refuses_unordered_transforms_hidden_inputs_and_processes"/><storycode:paste select="#f-005c1e9344c7-inventory_checks_binary_modes_symlinks_and_portable_paths"/><storycode:paste select="#f-005c1e9344c7-permissions_replay_and_destination_identity"/><storycode:paste select="#f-005c1e9344c7-raw_patch_grammar_and_ordinary_diff_fence"/><storycode:paste select="#f-005c1e9344c7-final_response_uses_dirty_start_and_incorporation_rechecks_destinations"/><storycode:paste select="#f-005c1e9344c7-patch_regeneration_preserves_order_and_reports_invalid_later_context"/><storycode:paste select="#f-005c1e9344c7-editing_a_proposal_does_not_replace_its_destination_guard"/><storycode:paste select="#f-005c1e9344c7-contributors_can_route_to_the_completed_project_from_markdown_alone"/><storycode:paste select="#f-005c1e9344c7-the_legacy_literal_subset_cannot_return_a_half_patched_program"/><storycode:paste select="#f-005c1e9344c7-generated_paths_cannot_shadow_the_markdown_closure_on_another_platform"/><storycode:paste select="#f-005c1e9344c7-immutable_identity_cannot_be_mislabeled_as_editable_and_invalid_origins_refuse"/><storycode:paste select="#f-005c1e9344c7-local_predecessor_state_does_not_depend_on_independent_act_names"/><storycode:paste select="#f-005c1e9344c7-explicit_nested_scopes_finish_all_their_acts_without_ordering_independent_writes"/><storycode:paste select="#f-005c1e9344c7-reverse_inserted_code_regenerates_its_patch_and_old_literal_edits_check_downstream"/><storycode:paste select="#f-005c1e9344c7-explicit_fragment_closure_is_ordered_private_and_upstream_is_read_only"/><storycode:paste select="#f-005c1e9344c7-local_bytes_map_to_renamed_endpoint_by_origin_and_removed_bytes_have_no_endpoint"/></storycode:file>

</storycode:doc>
