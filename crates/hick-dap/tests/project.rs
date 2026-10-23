use hick_lang::ProjectTree;
use std::{collections::BTreeMap, path::Path};
// Guarantee: docs/guarantees/debugging/a-frame-stays-in-document-coordinates.md
#[cfg(unix)]
#[test]
fn adapter_paths_through_a_symlink_keep_the_document_coordinates() {
    let dir = tempfile::tempdir().unwrap();
    let actual = dir.path().join("actual");
    std::fs::create_dir(&actual).unwrap();
    let alias = dir.path().join("alias");
    std::os::unix::fs::symlink(&actual, &alias).unwrap();
    let source = "# Explanation\n\n<hick:file path=\"program.py\">\nprint('hello')\n</hick:file>\n";
    let mapping = hick_dap::Mapping::for_document(Path::new("note.md"), source, &actual).unwrap();
    hick_dap::weave_into(source, &actual).unwrap();
    let path = alias.join("program.py");
    assert_eq!(mapping.to_document(&path, 0), Some(3));
    assert_eq!(
        mapping.source_location(&path, 0),
        Some((Path::new("note.md").to_path_buf(), 3))
    );
}
// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[test]
fn dap_line_breakpoints_ignore_diff_prefix_columns_and_refuse_deleted_lines() {
    let source = "<hick:project/>\n<hick:file id=\"base\" path=\"program.py\">\nprint('before')\n</hick:file>\n<hick:patch after=\"base\">\n--- a/program.py\n+++ b/program.py\n@@ -1 +1 @@\n-print('before')\n+print('after')\n</hick:patch>";
    let sources = BTreeMap::from([("p.md".into(), source.into())]);
    let state = hick_lsp::document::HickDocumentState::from_project("p.md", &sources).unwrap();
    let mapping =
        hick_dap::Mapping::for_state(Path::new("p.md"), &state, Path::new("scratch")).unwrap();
    assert_eq!(
        mapping.to_generated(9),
        Some((Path::new("scratch/program.py").to_path_buf(), 0))
    );
    assert_eq!(mapping.to_generated(8), None);
    assert!(hick_lang::evaluate_project("p.md", &sources, &ProjectTree::default()).is_ok());
}

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
