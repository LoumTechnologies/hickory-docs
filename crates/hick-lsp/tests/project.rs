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
// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[test]
fn services_use_completed_tree_and_separate_native_identities() {
    let (_, project) = fixture();
    assert!(
        materialize(&project, ProjectEndpoint::Old)
            .unwrap_err()
            .contains("empty")
    );
    let a = materialize(&project, ProjectEndpoint::New).unwrap();
    let b = materialize(&project, ProjectEndpoint::New).unwrap();
    assert_ne!(a.root(), b.root());
    assert_eq!(
        std::fs::read(a.root().join("library.ts")).unwrap(),
        b"export function welcome(name: string) { return 'Hello ' + name; }\n"
    );
    let local = materialize(&project, ProjectEndpoint::LocalAfter("caller".into())).unwrap();
    assert!(
        std::fs::read_to_string(local.root().join("library.ts"))
            .unwrap()
            .contains("greet")
    );
    assert!(
        endpoint(&project, &ProjectEndpoint::New)
            .unwrap()
            .get("app.ts")
            .is_some()
    );
}
// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[test]
fn removed_patch_lines_cannot_bind_to_unrelated_final_code() {
    let (sources, project) = fixture();
    let source = &sources["project.md"];
    let removed = source.find("-import { greet }").unwrap() + 1;
    let inserted = source.find("+import { welcome }").unwrap() + 1;
    assert_eq!(
        to_endpoint(&project.new, "project.md", removed).unwrap(),
        None
    );
    assert_eq!(
        to_endpoint(&project.new, "project.md", inserted).unwrap(),
        Some(("app.ts".into(), 0))
    );
    assert_eq!(
        to_source(&project.new, "app.ts", 0).unwrap(),
        Some(("project.md".into(), inserted))
    );
    let positions = BTreeMap::from([("unicode.md".into(), "🦀é\nabc".into())]);
    assert_eq!(utf16_position(&positions, "unicode.md", 6).unwrap(), (0, 3));
    assert!(utf16_position(&positions, "unicode.md", 1).is_err());
    let state =
        hick_lsp::document::HickDocumentState::from_project("project.md", &sources).unwrap();
    let file = state
        .virtual_files
        .iter()
        .find(|f| f.path == "app.ts")
        .unwrap();
    assert_eq!(
        file.written_content().as_bytes(),
        project.new.get("app.ts").unwrap().bytes.as_ref()
    );
    let map = hick_lsp::position_map::PositionMap::build(&file.segments).without_first_line();
    let source_line = source[..inserted].bytes().filter(|b| *b == b'\n').count() as u32;
    assert_eq!(map.to_virtual(source_line, 1), Some((0, 0)));
    let removed_line = source[..removed].bytes().filter(|b| *b == b'\n').count() as u32;
    assert_eq!(map.to_virtual(removed_line, 1), None);
}
// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[test]
fn native_writer_does_not_follow_existing_symlinks() {
    #[cfg(unix)]
    {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), dir.path().join("escape")).unwrap();
        let mut tree = ProjectTree::default();
        tree.insert("escape/write", ProjectEntry::file(b"no".to_vec()))
            .unwrap();
        assert!(hick_lsp::write_tree(dir.path(), &tree).is_err());
        assert!(!outside.path().join("write").exists());
    }
}
