<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/tests/project.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d5caa71df0d7"></a>
## `crates/hick-literate/tests/project.rs`

Related contracts: [Check every source file, including assets and permissions](../../../../guarantees/literate-projects/complete-inventory-is-byte-exact/contract.md), [Rebuild the program from its Markdown source](../../../../guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md).

[Read the test `binary_comparison_does_not_equal_same_length_placeholders` beside its requirement](../../../../guarantees/literate-projects/complete-inventory-is-byte-exact/contract.md#f-d5caa71df0d7-binary_comparison_does_not_equal_same_length_placeholders).
[Read the test `pipeline_uses_the_same_pure_complete_evaluator` beside its requirement](../../../../guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md#f-d5caa71df0d7-pipeline_uses_the_same_pure_complete_evaluator).
[Read the test `kept_generator_replay_uses_shared_fingerprint_formatting_expectations_and_origins` beside its requirement](../../../../guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md#f-d5caa71df0d7-kept_generator_replay_uses_shared_fingerprint_formatting_expectations_and_origins).
[Read the test `owned_scaffold_accepts_patches_without_generator_execution` beside its requirement](../../../../guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md#f-d5caa71df0d7-owned_ingest_is_editable_source_without_claiming_generator_execution).
[Read the test `a_refactor_baseline_cannot_follow_mutating_binary_temp_files` beside its requirement](../../../../guarantees/literate-projects/complete-inventory-is-byte-exact/contract.md#f-d5caa71df0d7-a_refactor_baseline_cannot_follow_mutating_binary_temp_files).

### Assemble `crates/hick-literate/tests/project.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

## immutable scaffold literal replay test

The ordinary pipeline also reconstructs standalone documents without a project declaration. Its global variable and substitution mechanisms must not rewrite captured paths or bodies. Preserve BOM, CRLF, Unicode and a literal placeholder, then refuse a changed base before any result is returned.

<storycode:copy id="immutable-scaffold-literal-replay-test">
#[tokio::test]
async fn standalone_scaffold_replay_keeps_literal_paths_and_bytes() {
    let path = "{{name}}.txt";
    let content = "\u{feff}FavoriteApp λ\r\n";
    let hash = hick_lang::scaffold_fingerprint(&std::collections::BTreeMap::from([(
        path.into(),
        content.into(),
    )]));
    let source = format!(
        r#"<hick:var name="name">changed</hick:var>
<hick:substitute name="app" pattern="FavoriteApp" value="Modified" />
<hick:container name="sdk" />
<hick:exec container="sdk">
<hick:copy>echo retained</hick:copy>
<hick:ingested sha256="{hash}">
<hick:file path="{path}">
{content}</hick:file>
</hick:ingested>
</hick:exec>
"#
    );
    let result = hick_literate::run_pipeline_weave(&[("standalone.md", &source)], &[], None)
        .await
        .unwrap();
    assert_eq!(result.files.len(), 1);
    assert_eq!(result.files[path].as_text(), Some(content));
    let tampered = source.replace("FavoriteApp", "WrongBase");
    let error = hick_literate::run_pipeline_weave(&[("standalone.md", &tampered)], &[], None)
        .await
        .err()
        .unwrap();
    assert!(error.to_string().contains("hash mismatch"));
}
</storycode:copy>

<storycode:file path="crates/hick-literate/tests/project.rs"><storycode:paste select="#f-d5caa71df0d7-binary_comparison_does_not_equal_same_length_placeholders"/><storycode:paste select="#f-d5caa71df0d7-pipeline_uses_the_same_pure_complete_evaluator"/><storycode:paste select="#f-d5caa71df0d7-kept_generator_replay_uses_shared_fingerprint_formatting_expectations_and_origins"/><storycode:paste select="#f-d5caa71df0d7-owned_ingest_is_editable_source_without_claiming_generator_execution"/><storycode:paste select="#f-d5caa71df0d7-a_refactor_baseline_cannot_follow_mutating_binary_temp_files"/><storycode:paste select="#immutable-scaffold-literal-replay-test"/></storycode:file>

</storycode:doc>
