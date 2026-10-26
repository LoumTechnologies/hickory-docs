<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/api/tests/tree_tests.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c98e0438f96c"></a>
## `crates/hickory-cli/src/serve/api/tests/tree_tests.rs`

<a id="f-c98e0438f96c-a_documents_weave_target_is_one_of_its_outputs"></a>
### `a_documents_weave_target_is_one_of_its_outputs`

<storycode:copy id="f-c98e0438f96c-a_documents_weave_target_is_one_of_its_outputs">
use super::*;

#[test]
fn a_documents_weave_target_is_one_of_its_outputs() {
    // An explicitly declared rendering is a generated output.
    let outputs = declared_outputs(
        "cards-source.md",
        r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="cards.md">"#,
    );
    assert_eq!(outputs, vec!["cards.md".to_string()]);
}

</storycode:copy>
<a id="f-c98e0438f96c-every_generated_file_block_is_an_output"></a>
### `every_generated_file_block_is_an_output`

<storycode:copy id="f-c98e0438f96c-every_generated_file_block_is_an_output">
#[test]
fn every_generated_file_block_is_an_output() {
    let outputs = declared_outputs(
        "app.md",
        r#"
            <hick:file path="src/main.rs">fn main() {}</hick:file>
            <hick:file path="Cargo.toml">[package]</hick:file>
            "#,
    );
    assert_eq!(
        outputs,
        vec!["src/main.rs".to_string(), "Cargo.toml".to_string()]
    );
}

</storycode:copy>
<a id="f-c98e0438f96c-outputs_are_relative_to_the_documents_own_directory"></a>
### `outputs_are_relative_to_the_documents_own_directory`

<storycode:copy id="f-c98e0438f96c-outputs_are_relative_to_the_documents_own_directory">
#[test]
fn outputs_are_relative_to_the_documents_own_directory() {
    // A tree key is root-relative, so a document three folders down that
    // says `path="main.rs"` must not claim the root's `main.rs`.
    let outputs = declared_outputs(
        "notes/deep/app.md",
        r#"<hick:doc weave="README.md"><hick:file path="src/main.rs"/>"#,
    );
    assert_eq!(
        outputs,
        vec![
            "notes/deep/README.md".to_string(),
            "notes/deep/src/main.rs".to_string(),
        ]
    );
}

</storycode:copy>
<a id="f-c98e0438f96c-a_dot_dot_in_a_path_is_resolved_rather_than_left_in_the_key"></a>
### `a_dot_dot_in_a_path_is_resolved_rather_than_left_in_the_key`

<storycode:copy id="f-c98e0438f96c-a_dot_dot_in_a_path_is_resolved_rather_than_left_in_the_key">
#[test]
fn a_dot_dot_in_a_path_is_resolved_rather_than_left_in_the_key() {
    // `notes/deep/../out.rs` would never match the tree's `notes/out.rs`.
    let outputs = declared_outputs("notes/deep/app.md", r#"<hick:file path="../out.rs"/>"#);
    assert_eq!(outputs, vec!["notes/out.rs".to_string()]);
}

</storycode:copy>
<a id="f-c98e0438f96c-a_path_built_from_a_variable_is_not_claimed"></a>
### `a_path_built_from_a_variable_is_not_claimed`

<storycode:copy id="f-c98e0438f96c-a_path_built_from_a_variable_is_not_claimed">
#[test]
fn a_path_built_from_a_variable_is_not_claimed() {
    // The honest boundary of reading declarations instead of weaving: an
    // interpolated path is left alone rather than recorded as the literal
    // `{{name}}.rs`, which would mark a file nobody has.
    // Markdown source is not a generated output; only explicit outputs
    // are claimed.
    assert_eq!(
        declared_outputs("app.md", r#"<hick:file path="{{name}}.rs"/>"#),
        Vec::<String>::new()
    );
}

</storycode:copy>
<a id="f-c98e0438f96c-a_volume_output_directory_claims_the_files_under_it"></a>
### `a_volume_output_directory_claims_the_files_under_it`

A volume's `output=` directory claims everything under it.
This is how a document says "a program wrote these and I do not know
their names". Before it was read, the app showed a generated API
layer as ordinary source you could edit — and the next run replaced
it — while marking the hand-written domain beside it as generated.

<storycode:copy id="f-c98e0438f96c-a_volume_output_directory_claims_the_files_under_it">
/// A volume's `output=` directory claims everything under it.
///
/// This is how a document says "a program wrote these and I do not know
/// their names". Before it was read, the app showed a generated API
/// layer as ordinary source you could edit — and the next run replaced
/// it — while marking the hand-written domain beside it as generated.
#[test]
fn a_volume_output_directory_claims_the_files_under_it() {
    let declared = declared_outputs(
        "30-api.md",
        r#"<hick:volume name="apiout" output="src/Api/Generated" />"#,
    );
    assert_eq!(declared, vec!["src/Api/Generated/".to_string()]);

    let generated = HashMap::from([
        ("src/Api/Generated/".to_string(), "d1".to_string()),
        ("src/Api/".to_string(), "d0".to_string()),
    ]);
    // Under the directory: generated, and by the NARROWEST volume that
    // claims it.
    assert_eq!(
        generated_by(&generated, "src/Api/Generated/Endpoints.g.cs").as_deref(),
        Some("d1")
    );
    assert_eq!(
        generated_by(&generated, "src/Api/Other.cs").as_deref(),
        Some("d0")
    );
    // A prefix match is on a path BOUNDARY, never on spelling.
    // `src/ApiOther/` is not under `src/Api/`, and
    // `src/Api/GeneratedThing.cs` is under `src/Api/` but NOT under
    // `src/Api/Generated/` — so the wider volume claims it and the
    // narrower one does not.
    assert_eq!(generated_by(&generated, "src/ApiOther/x.cs"), None);
    assert_eq!(
        generated_by(&generated, "src/Api/GeneratedThing.cs").as_deref(),
        Some("d0")
    );
}

</storycode:copy>
<a id="f-c98e0438f96c-markdown_source_and_weave_none_claim_no_rendered_output"></a>
### `markdown_source_and_weave_none_claim_no_rendered_output`

Markdown source generates nothing unless it declares an output.
`weave="none"` also claims no output: the keyword is not a path.

<storycode:copy id="f-c98e0438f96c-markdown_source_and_weave_none_claim_no_rendered_output">
/// Markdown source generates nothing unless it declares an output.
/// `weave="none"` also claims no output: the keyword is not a path.
#[test]
fn markdown_source_and_weave_none_claim_no_rendered_output() {
    assert_eq!(
        declared_outputs("notes.md", "# Just prose\n"),
        Vec::<String>::new()
    );
    assert!(
        declared_outputs(
            "gen.md",
            r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">"#
        )
        .is_empty()
    );
}

</storycode:copy>
<a id="f-c98e0438f96c-marking_skips_documents_and_directories"></a>
### `marking_skips_documents_and_directories`

<storycode:copy id="f-c98e0438f96c-marking_skips_documents_and_directories">
#[test]
fn marking_skips_documents_and_directories() {
    // A `.md` file is a document, never somebody else's output, and a
    // directory is not a file at all.
    let generated = HashMap::from([
        ("cards.md".to_string(), "d1".to_string()),
        ("src".to_string(), "d1".to_string()),
        ("other.md".to_string(), "d1".to_string()),
    ]);
    let mut tree = vec![
        TreeNode {
            name: "cards.md".into(),
            path: "cards.md".into(),
            dir: false,
            doc_id: None,
            generated_by: None,
            diverged: None,
            children: None,
        },
        TreeNode {
            name: "other.md".into(),
            path: "other.md".into(),
            dir: false,
            doc_id: Some("d2".into()),
            generated_by: None,
            diverged: None,
            children: None,
        },
        TreeNode {
            name: "src".into(),
            path: "src".into(),
            dir: true,
            doc_id: None,
            generated_by: None,
            diverged: None,
            children: Some(vec![TreeNode {
                name: "main.rs".into(),
                path: "src/main.rs".into(),
                dir: false,
                doc_id: None,
                generated_by: None,
                diverged: None,
                children: None,
            }]),
        },
    ];
    mark_generated(&mut tree, &generated);
    assert_eq!(tree[0].generated_by.as_deref(), Some("d1"));
    assert_eq!(tree[1].generated_by, None, "a document is not an output");
    assert_eq!(tree[2].generated_by, None, "a directory is not a file");
    let children = tree[2].children.as_ref().unwrap();
    assert_eq!(children[0].generated_by, None, "nothing claims src/main.rs");
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/api/tests/tree_tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/api/tests/tree_tests.rs"><storycode:paste select="#f-c98e0438f96c-a_documents_weave_target_is_one_of_its_outputs"/><storycode:paste select="#f-c98e0438f96c-every_generated_file_block_is_an_output"/><storycode:paste select="#f-c98e0438f96c-outputs_are_relative_to_the_documents_own_directory"/><storycode:paste select="#f-c98e0438f96c-a_dot_dot_in_a_path_is_resolved_rather_than_left_in_the_key"/><storycode:paste select="#f-c98e0438f96c-a_path_built_from_a_variable_is_not_claimed"/><storycode:paste select="#f-c98e0438f96c-a_volume_output_directory_claims_the_files_under_it"/><storycode:paste select="#f-c98e0438f96c-markdown_source_and_weave_none_claim_no_rendered_output"/><storycode:paste select="#f-c98e0438f96c-marking_skips_documents_and_directories"/></storycode:file>

</storycode:doc>
