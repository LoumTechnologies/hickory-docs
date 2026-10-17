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
// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[test]
fn complete_project_positions_keep_foreign_documents_and_utf16_boundaries() {
    let sources = std::collections::BTreeMap::from([
        ("project.md".into(),"<hick:project sources=\"member.md\"/><hick:file id=\"a\" path=\"a.ts\">🦀x\n</hick:file>".into()),
        ("member.md".into(),"<hick:project entry=\"project.md\"/><hick:file id=\"b\" path=\"b.ts\">éy\n</hick:file>".into()),
    ]);
    let state =
        hick_lsp::document::HickDocumentState::from_project("project.md", &sources).unwrap();
    let foreign = &state.project_maps["b.ts"];
    let (document, line, column) = foreign.source_document(0, 1).unwrap();
    assert_eq!(document.as_deref(), Some("member.md"));
    assert_eq!(line, 0);
    assert!(column > 1);
    assert_eq!(foreign.to_source(0, 1), None);
    let local = &state.project_maps["a.ts"];
    assert!(
        local.to_source(0, 1).is_none(),
        "half a surrogate pair has no byte coordinate"
    );
    assert!(local.to_source(0, 2).is_some());
    let old = hick_lsp::document::HickDocumentState::from_project_at(
        "project.md",
        &sources,
        &hick_lang::ProjectTree::default(),
        &hick_lsp::ProjectEndpoint::Old,
    )
    .unwrap();
    assert!(
        old.virtual_files.is_empty(),
        "Old is genuinely empty, never borrowed working code"
    );
}

// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[test]
fn captured_native_contexts_are_lazy_isolated_and_never_borrow_working_files() {
    let contexts = hick_lsp::ProjectContexts::default();
    let mut old = ProjectTree::default();
    old.insert("app.ts", ProjectEntry::file(b"old".to_vec()))
        .unwrap();
    let mut new = old.clone();
    new.insert("app.ts", ProjectEntry::file(b"new".to_vec()))
        .unwrap();
    contexts.register("1111111111111111", old).unwrap();
    contexts.register("2222222222222222", new).unwrap();
    let (_, _, a) = contexts
        .get(&"hick-project:///1111111111111111/app.ts".parse().unwrap())
        .unwrap();
    let (_, _, b) = contexts
        .get(&"hick-project:///2222222222222222/app.ts".parse().unwrap())
        .unwrap();
    assert!(!a.is_materialized());
    assert!(!b.is_materialized());
    let first = a.materialize().unwrap();
    let second = b.materialize().unwrap();
    assert_ne!(first, second);
    assert_eq!(std::fs::read(first.join("app.ts")).unwrap(), b"old");
    assert_eq!(std::fs::read(second.join("app.ts")).unwrap(), b"new");
    assert!(
        contexts
            .get(
                &"hick-project:///1111111111111111/missing.ts"
                    .parse()
                    .unwrap()
            )
            .is_err()
    );
    contexts.remove("1111111111111111");
    assert!(
        contexts
            .get(&"hick-project:///1111111111111111/app.ts".parse().unwrap())
            .is_err()
    );
    assert!(
        contexts
            .register("3333333333333333", ProjectTree::default())
            .is_err()
    );
}

// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
#[test]
fn clicked_file_uses_exact_old_bytes_and_original_document_breakpoint_lines() {
    let source = "<hick:project/>\n<hick:file id=\"base\" path=\"app.py\">\nprint(1)\n</hick:file>\n<hick:file id=\"lib\" after=\"base\" path=\"lib.py\">value = 2\n</hick:file>\n<hick:patch id=\"replace\" after=\"lib\">\n--- a/app.py\n+++ b/app.py\n@@ -1 +1 @@\n-print(1)\n+print(2)\n</hick:patch>";
    let sources = BTreeMap::from([("p.md".into(), source.into())]);
    let selection = ProjectEndpoint::LatestFile {
        id: "base".into(),
        source: source.into(),
    };
    let state = hick_lsp::document::HickDocumentState::from_project_at(
        "p.md",
        &sources,
        &ProjectTree::default(),
        &selection,
    )
    .unwrap();
    let tree = state.selected_project.as_ref().unwrap();
    assert_eq!(tree.get("app.py").unwrap().bytes.as_ref(), b"print(1)\n");
    assert!(tree.get("lib.py").is_some());
    let map = &state.project_maps["app.py"];
    assert_eq!(map.to_virtual(2, 0), Some((0, 0)));
    let changed = BTreeMap::from([("p.md".into(), source.replace("print(1)", "print(3)"))]);
    let error = hick_lsp::document::HickDocumentState::from_project_at(
        "p.md",
        &changed,
        &ProjectTree::default(),
        &selection,
    )
    .unwrap_err();
    assert!(error.to_string().contains("document changed"));
}

// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
#[test]
fn generated_code_selection_uses_displayed_bytes_instead_of_the_previous_file_context() {
    let source = "<hick:project/><hick:file id=\"base\" path=\"app.py\">print(1)\n</hick:file><hick:patch id=\"change\" after=\"base\">\n--- a/app.py\n+++ b/app.py\n@@ -1 +1 @@\n-print(1)\n+print(2)\n</hick:patch>";
    let sources = BTreeMap::from([("p.md".into(), source.into())]);
    for code in ["print(1)\n", "print(2)\n"] {
        let value = format!(
            "latest-code:{}",
            serde_json::json!(["app.py", code, source])
        );
        let selection = ProjectEndpoint::parse(&value).unwrap();
        let state = hick_lsp::document::HickDocumentState::from_project_at(
            "p.md",
            &sources,
            &ProjectTree::default(),
            &selection,
        )
        .unwrap();
        assert_eq!(
            state
                .selected_project
                .unwrap()
                .get("app.py")
                .unwrap()
                .bytes
                .as_ref(),
            code.as_bytes()
        );
    }
    let selection = ProjectEndpoint::LatestCode {
        path: "app.py".into(),
        code: "not evaluated".into(),
        source: source.into(),
    };
    assert!(
        hick_lsp::document::HickDocumentState::from_project_at(
            "p.md",
            &sources,
            &ProjectTree::default(),
            &selection
        )
        .unwrap_err()
        .to_string()
        .contains("exact displayed code")
    );
}
