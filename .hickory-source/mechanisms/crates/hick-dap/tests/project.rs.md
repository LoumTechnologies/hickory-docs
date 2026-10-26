<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-dap/tests/project.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b360a6d46b8d"></a>
## `crates/hick-dap/tests/project.rs`

Related contracts: [A frame stays in document coordinates](../../../../guarantees/debugging/a-frame-stays-in-document-coordinates/contract.md), [Keep language tools and debug launches tied to their captured project](../../../../guarantees/literate-projects/tools-select-complete-isolated-endpoints/contract.md).

[Read the test `adapter_paths_through_a_symlink_keep_the_document_coordinates` beside its requirement](../../../../guarantees/debugging/a-frame-stays-in-document-coordinates/contract.md#f-b360a6d46b8d-adapter_paths_through_a_symlink_keep_the_document_coordinates).
[Read the test `dap_line_breakpoints_ignore_diff_prefix_columns_and_refuse_deleted_lines` beside its requirement](../../../../guarantees/literate-projects/tools-select-complete-isolated-endpoints/contract.md#f-b360a6d46b8d-dap_line_breakpoints_ignore_diff_prefix_columns_and_refuse_deleted_lines).
<a id="f-b360a6d46b8d-captured_source_breakpoints_require_this_context_and_a_present_file"></a>
### `captured_source_breakpoints_require_this_context_and_a_present_file`

<storycode:copy id="f-b360a6d46b8d-captured_source_breakpoints_require_this_context_and_a_present_file">
#[test]
fn captured_source_breakpoints_require_this_context_and_a_present_file() {
    let source = "<hick:project/>\n<hick:file path=\"main.py\">\nprint(1)\n</hick:file>\n<hick:file path=\"helpers.py\">\ndef helper():\n    return 2\n</hick:file>";
    let sources = BTreeMap::from([("p.md".into(), source.into())]);
    let project = hick_lang::evaluate_project("p.md", &sources, &ProjectTree::default()).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    hick_lsp::write_tree(scratch.path(), &project.new).unwrap();
    let mapping =
        hick_dap::Mapping::captured(&project.new, scratch.path(), "current", "main.py").unwrap();
    assert_eq!(
        mapping.to_generated_in(Path::new("hick-project:///current/helpers.py"), 1),
        Some((scratch.path().canonicalize().unwrap().join("helpers.py"), 1))
    );
    assert_eq!(
        mapping.to_generated_in(Path::new("hick-project:///other/helpers.py"), 1),
        None
    );
    assert_eq!(
        mapping.to_generated_in(Path::new("hick-project:///current/missing.py"), 1),
        None
    );
    assert_eq!(
        mapping.to_generated_in(Path::new("hick-project:///current/helpers.py"), 99),
        None
    );
}

</storycode:copy>
<a id="f-b360a6d46b8d-missing_generated_files_resolve_through_an_existing_parent_alias"></a>
### `missing_generated_files_resolve_through_an_existing_parent_alias`

Private capture validates source coordinates before materializing outputs.

<storycode:copy id="f-b360a6d46b8d-missing_generated_files_resolve_through_an_existing_parent_alias">
// Private capture validates source coordinates before materializing outputs.
#[cfg(unix)]
#[test]
fn missing_generated_files_resolve_through_an_existing_parent_alias() {
    let dir = tempfile::tempdir().unwrap();
    let actual = dir.path().join("actual");
    std::fs::create_dir(&actual).unwrap();
    let alias = dir.path().join("alias");
    std::os::unix::fs::symlink(&actual, &alias).unwrap();
    let source = "# Explanation\n<hick:file path=\"src/program.py\">\nvalue = 42\n</hick:file>\n";
    let mapping = hick_dap::Mapping::for_document(Path::new("note.md"), source, &alias).unwrap();
    let generated = alias.join("src/program.py");
    assert!(!generated.exists());
    assert_eq!(mapping.to_document(&generated, 0), Some(2));
    assert_eq!(mapping.to_document(&alias.join("src/other.py"), 0), None);
    assert_eq!(
        mapping.to_document(&dir.path().join("src/program.py"), 0),
        None
    );
    // Materializing the same file must preserve its identity and coordinates.
    hick_dap::weave_into(source, &actual).unwrap();
    assert_eq!(mapping.to_document(&generated, 0), Some(2));
}

</storycode:copy>
<a id="f-b360a6d46b8d-plain_file_breakpoints_bind_canonical_paths_and_keep_the_editor_alias"></a>
### `plain_file_breakpoints_bind_canonical_paths_and_keep_the_editor_alias`

<storycode:copy id="f-b360a6d46b8d-plain_file_breakpoints_bind_canonical_paths_and_keep_the_editor_alias">
#[cfg(unix)]
#[test]
fn plain_file_breakpoints_bind_canonical_paths_and_keep_the_editor_alias() {
    let dir = tempfile::tempdir().unwrap();
    let actual = dir.path().join("actual");
    std::fs::create_dir(&actual).unwrap();
    let file = actual.join("Program.cs");
    std::fs::write(&file, "int total = 7;\nSystem.Console.WriteLine(total);\n").unwrap();
    let alias = dir.path().join("alias");
    std::os::unix::fs::symlink(&actual, &alias).unwrap();
    let viewed = alias.join("Program.cs");
    let canonical = file.canonicalize().unwrap();
    let mapping = hick_dap::Mapping::identity(&viewed);
    assert_eq!(mapping.to_generated(1), Some((canonical.clone(), 1)));
    assert_eq!(
        mapping.to_generated_in(&viewed, 1),
        Some((canonical.clone(), 1))
    );
    assert_eq!(mapping.document(), viewed);
    assert_eq!(
        mapping.source_location(&canonical, 1),
        Some((viewed.clone(), 1))
    );
    assert_eq!(mapping.to_document(&canonical, 1), Some(1));
    assert_eq!(mapping.source_location(&actual.join("Other.cs"), 1), None);
}
</storycode:copy>

### Assemble `crates/hick-dap/tests/project.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-dap/tests/project.rs"><storycode:paste select="#f-b360a6d46b8d-adapter_paths_through_a_symlink_keep_the_document_coordinates"/><storycode:paste select="#f-b360a6d46b8d-dap_line_breakpoints_ignore_diff_prefix_columns_and_refuse_deleted_lines"/><storycode:paste select="#f-b360a6d46b8d-captured_source_breakpoints_require_this_context_and_a_present_file"/><storycode:paste select="#f-b360a6d46b8d-missing_generated_files_resolve_through_an_existing_parent_alias"/><storycode:paste select="#f-b360a6d46b8d-plain_file_breakpoints_bind_canonical_paths_and_keep_the_editor_alias"/></storycode:file>

</storycode:doc>
