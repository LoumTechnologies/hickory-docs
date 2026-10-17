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
// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
#[test]
fn latest_compatible_version_preserves_later_other_files_without_replacing_displayed_code() {
    let project = fixture();
    let displayed = &project.local_states["zbase"].1;
    let (index, path) = latest_debug_project(&project.versions, displayed, "app.py").unwrap();
    assert_eq!(path, "app.py");
    let selected = &project.versions[index];
    assert_eq!(
        selected.get("app.py").unwrap().bytes.as_ref(),
        b"print(1)\n"
    );
    assert_eq!(
        selected.get("lib.py").unwrap().bytes.as_ref(),
        b"value = 2\n"
    );
    assert!(selected.get("config.json").is_none()); // No synthetic mixed-version tree.
    let (index, _) = latest_debug_project(
        &project.versions,
        &project.local_states["replacement"].1,
        "app.py",
    )
    .unwrap();
    assert_eq!(index, project.versions.len() - 1);
    assert!(project.versions[index].get("config.json").is_some());
}
// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
#[test]
fn unsaved_or_absent_code_is_refused_instead_of_debugging_different_bytes() {
    let project = fixture();
    let mut draft = project.new.clone();
    draft
        .insert(
            "app.py",
            hick_lang::ProjectEntry::file(b"unsaved".as_slice()),
        )
        .unwrap();
    assert!(
        latest_debug_project(&project.versions, &draft, "app.py")
            .unwrap_err()
            .contains("exact displayed code")
    );
    assert!(latest_debug_project(&project.versions, &ProjectTree::default(), "app.py").is_err());
}
// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
#[test]
fn byte_identical_revert_uses_latest_state_even_when_provenance_changed() {
    let source = format!(
        "<hick:project/><hick:file id=\"base\" path=\"app.py\">print(1)\n</hick:file>{}{}",
        patch("change", "base", "print(1)\n", "print(2)\n"),
        patch("revert", "change", "print(2)\n", "print(1)\n")
    );
    let project = evaluate_project(
        "p.md",
        &BTreeMap::from([("p.md".into(), source)]),
        &ProjectTree::default(),
    )
    .unwrap();
    let (index, _) =
        latest_debug_project(&project.versions, &project.local_states["base"].1, "app.py").unwrap();
    assert_eq!(index, project.versions.len() - 1);
}
// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
#[test]
fn unchanged_rename_follows_provenance_but_equal_unrelated_files_do_not() {
    let source = "<hick:project/><hick:file id=\"base\" path=\"app.py\">print(1)\n</hick:file><hick:patch id=\"rename\" after=\"base\">\n--- a/app.py\n+++ b/main.py\n@@ -1 +1 @@\n print(1)\n</hick:patch>";
    let project = evaluate_project(
        "p.md",
        &BTreeMap::from([("p.md".into(), source.into())]),
        &ProjectTree::default(),
    )
    .unwrap();
    let (index, path) =
        latest_debug_project(&project.versions, &project.local_states["base"].1, "app.py").unwrap();
    assert_eq!(index, project.versions.len() - 1);
    assert_eq!(path, "main.py");
}

// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
#[test]
fn a_mode_change_does_not_make_identical_code_older() {
    let mut old = ProjectTree::default();
    old.insert(
        "app.py",
        hick_lang::ProjectEntry::file(b"print(1)\n".as_slice()),
    )
    .unwrap();
    let mut executable = old.get("app.py").unwrap().as_ref().clone();
    executable.executable = true;
    let mut new = old.clone();
    new.insert("app.py", executable).unwrap();
    assert_eq!(
        latest_debug_project(&[old.clone(), new], &old, "app.py")
            .unwrap()
            .0,
        1
    );
}

// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
#[test]
fn equal_text_in_an_unrelated_file_is_not_used_as_a_rename() {
    let mut old = ProjectTree::default();
    old.insert(
        "app.py",
        hick_lang::ProjectEntry::file(b"print(1)\n".as_slice()),
    )
    .unwrap();
    let mut unrelated = ProjectTree::default();
    unrelated
        .insert(
            "other.py",
            hick_lang::ProjectEntry::file(b"print(1)\n".as_slice()),
        )
        .unwrap();
    assert_eq!(
        latest_debug_project(&[old.clone(), unrelated], &old, "app.py").unwrap(),
        (0, "app.py".into())
    );
}

// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
#[test]
fn atomic_scope_never_exposes_a_new_caller_with_the_old_library() {
    let source = "<hick:project/><hick:file id=\"app\" path=\"app.py\">from library import greet\nprint(greet())\n</hick:file><hick:file id=\"library\" path=\"library.py\">def greet():\n    return 'Hello'\n</hick:file><hick:scope id=\"greeting\" after=\"app library\"><hick:patch id=\"caller\">\n--- a/app.py\n+++ b/app.py\n@@ -1,2 +1,2 @@\n-from library import greet\n-print(greet())\n+from library import welcome\n+print(welcome('Hickory'))\n</hick:patch><hick:patch id=\"complete\" after=\"caller\">\n--- a/library.py\n+++ b/library.py\n@@ -1,2 +1,2 @@\n-def greet():\n-    return 'Hello'\n+def welcome(name):\n+    return 'Hello ' + name\n</hick:patch></hick:scope>";
    let project = evaluate_project(
        "p.md",
        &BTreeMap::from([("p.md".into(), source.replace("><hick:", ">\n<hick:"))]),
        &ProjectTree::default(),
    )
    .unwrap();
    let old = latest_debug_project(
        &project.versions,
        &project.local_states["complete"].0,
        "library.py",
    )
    .unwrap()
    .0;
    assert_eq!(
        project.versions[old].get("app.py").unwrap().bytes.as_ref(),
        b"from library import greet\nprint(greet())\n"
    );
    let old_caller = latest_debug_project(
        &project.versions,
        &project.local_states["caller"].0,
        "app.py",
    )
    .unwrap()
    .0;
    assert_eq!(old, old_caller);
    let new = latest_debug_project(
        &project.versions,
        &project.local_states["caller"].1,
        "app.py",
    )
    .unwrap()
    .0;
    assert_eq!(
        project.versions[new]
            .get("library.py")
            .unwrap()
            .bytes
            .as_ref(),
        b"def welcome(name):\n    return 'Hello ' + name\n"
    );
    assert_eq!(new, project.versions.len() - 1);
}
