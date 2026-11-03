use super::*;
fn setup() -> (tempfile::TempDir, BTreeMap<String, String>, ProjectTree) {
    let root = tempfile::tempdir().unwrap();
    let source = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project/><h:file id=\"base\" path=\"app.py\">print('hello')\n</h:file></h:doc>";
    fs::write(root.path().join("project.md"), source).unwrap();
    let sources = sources(root.path(), "project.md").unwrap();
    let tree = evaluate_project("project.md", &sources, &ProjectTree::default())
        .unwrap()
        .new;
    materialize(root.path(), "project.md", &sources, &tree).unwrap();
    (root, sources, tree)
}

#[test]
fn sync_initializes_only_complete_checkouts() {
    let (_fixture, source, _) = setup();
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("project.md"), &source["project.md"]).unwrap();
    assert!(
        sync(root.path(), "project.md")
            .unwrap_err()
            .contains("app.py")
    );
    fs::write(root.path().join("app.py"), "print('hello')\n").unwrap();
    sync(root.path(), "project.md").unwrap();
    sync(root.path(), "project.md").unwrap();
    assert!(status(root.path(), "project.md").unwrap().is_empty());
}

#[test]
fn sync_adopts_a_pulled_tree_including_new_and_removed_outputs() {
    let (root, original, tree) = setup();
    let pulled = original["project.md"].replace("hello", "pulled").replace(
        "</h:doc>",
        "<h:file path=\"new.txt\">new\n</h:file></h:doc>",
    );
    fs::write(root.path().join("project.md"), &pulled).unwrap();
    fs::write(root.path().join("app.py"), "print('pulled')\n").unwrap();
    fs::write(root.path().join("new.txt"), "new\n").unwrap();
    // Reproduce journals left by the older, baseline-only capture command.
    let observation = capture(root.path(), "project.md", &[]).unwrap().unwrap();
    assert!(!observation.paths.contains(&"new.txt".into()));
    sync(root.path(), "project.md").unwrap();
    assert!(status(root.path(), "project.md").unwrap().is_empty());
    let store = Store::open(root.path(), "project.md").unwrap();
    assert!(
        store
            .folder
            .join(format!(
                "accepted/{}/{}.json",
                store
                    .read::<serde_json::Value>("incorporated.json")
                    .unwrap()["capture"]
                    .as_str()
                    .unwrap(),
                observation.id
            ))
            .exists()
    );
    let baseline: Snapshot = store.read("baseline.json").unwrap();
    assert!(baseline.entries.contains_key("new.txt"));
    drop(store);
    sync(root.path(), "project.md").unwrap();
    let source = sources(root.path(), "project.md").unwrap();
    let output = evaluate_project("project.md", &source, &ProjectTree::default())
        .unwrap()
        .new;
    materialize(root.path(), "project.md", &source, &output).unwrap();
    // A later pull removes an output as well as its construction.
    fs::write(root.path().join("project.md"), &original["project.md"]).unwrap();
    fs::write(root.path().join("app.py"), "print('hello')\n").unwrap();
    assert!(
        sync(root.path(), "project.md")
            .unwrap_err()
            .contains("new.txt")
    );
    assert_eq!(
        fs::read_to_string(root.path().join("new.txt")).unwrap(),
        "new\n"
    );
    fs::remove_file(root.path().join("new.txt")).unwrap();
    sync(root.path(), "project.md").unwrap();
    materialize(root.path(), "project.md", &original, &tree).unwrap();
}

#[test]
fn sync_refuses_local_edits_and_missing_new_outputs_without_acknowledging_them() {
    let (root, original, _) = setup();
    fs::write(root.path().join("app.py"), "local work\n").unwrap();
    let capture = capture(root.path(), "project.md", &[]).unwrap().unwrap();
    assert!(
        sync(root.path(), "project.md")
            .unwrap_err()
            .contains("app.py")
    );
    assert_eq!(status(root.path(), "project.md").unwrap()[0].id, capture.id);
    assert_eq!(
        fs::read_to_string(root.path().join("app.py")).unwrap(),
        "local work\n"
    );
    fs::write(root.path().join("app.py"), "print('hello')\n").unwrap();
    fs::write(
        root.path().join("project.md"),
        original["project.md"].replace(
            "</h:doc>",
            "<h:file path=\"new.txt\">new\n</h:file></h:doc>",
        ),
    )
    .unwrap();
    assert!(
        sync(root.path(), "project.md")
            .unwrap_err()
            .contains("new.txt")
    );
    assert_eq!(status(root.path(), "project.md").unwrap()[0].id, capture.id);
}

#[test]
fn sync_preserves_captured_new_files_outside_current_construction() {
    let (root, _, _) = setup();
    fs::write(root.path().join("local.txt"), "new local work\n").unwrap();
    let capture = capture(root.path(), "project.md", &["local.txt".into()])
        .unwrap()
        .unwrap();
    assert!(
        sync(root.path(), "project.md")
            .unwrap_err()
            .contains("local.txt")
    );
    assert_eq!(status(root.path(), "project.md").unwrap()[0].id, capture.id);
    assert_eq!(
        fs::read_to_string(root.path().join("local.txt")).unwrap(),
        "new local work\n"
    );
}

#[test]
fn sync_refuses_unmaterialized_mounted_edits() {
    let (root, source, tree) = setup();
    let mut edited = tree.clone();
    edited
        .insert(
            "virtual.txt",
            ProjectEntry::file(b"unsaved work\n".to_vec()),
        )
        .unwrap();
    let capture = capture_virtual(root.path(), "project.md", &source, &tree, &edited).unwrap();
    assert!(
        sync(root.path(), "project.md")
            .unwrap_err()
            .contains("mounted")
    );
    assert_eq!(status(root.path(), "project.md").unwrap()[0].id, capture.id);
}

#[cfg(unix)]
#[test]
fn sync_refuses_output_mode_and_symlink_changes() {
    use std::os::unix::{fs::PermissionsExt, fs::symlink};
    let (root, _, _) = setup();
    let path = root.path().join("app.py");
    let permissions = fs::metadata(&path).unwrap().permissions();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(
        sync(root.path(), "project.md")
            .unwrap_err()
            .contains("app.py")
    );
    fs::set_permissions(&path, permissions).unwrap();
    fs::rename(&path, root.path().join("local.py")).unwrap();
    symlink("local.py", &path).unwrap();
    assert!(
        sync(root.path(), "project.md")
            .unwrap_err()
            .contains("app.py")
    );
}
#[test]
fn external_edits_are_durable_pending_and_block_generation_until_explicit_incorporation() {
    let (root, original, tree) = setup();
    fs::write(root.path().join("app.py"), "print('redacted')\n").unwrap();
    let error = materialize(root.path(), "project.md", &original, &tree).unwrap_err();
    assert!(error.contains("pending output"));
    assert_eq!(sources(root.path(), "project.md").unwrap(), original);
    assert_eq!(
        fs::read_to_string(root.path().join("app.py")).unwrap(),
        "print('redacted')\n"
    );
    let pending = status(root.path(), "project.md").unwrap();
    assert_eq!(pending.len(), 1);
    let id = &pending[0].id;
    assert!(
        pending[0]
            .patch
            .as_ref()
            .unwrap()
            .contains("+print('redacted')")
    );
    assert_eq!(
        capture(root.path(), "project.md", &[]).unwrap().unwrap().id,
        *id
    );
    assert!(
        accept(root.path(), "project.md", id)
            .unwrap_err()
            .contains("does not reproduce")
    );
    new_patch(
        root.path(),
        "project.md",
        id,
        "privacy.md",
        "redact",
        "Avoid disclosing the original greeting.",
    )
    .unwrap();
    assert!(status(root.path(), "project.md").unwrap().len() == 1);
    assert!(
        fs::read_to_string(root.path().join("project.md"))
            .unwrap()
            .contains("print('hello')")
    );
    let concept = fs::read_to_string(root.path().join("privacy.md")).unwrap();
    assert!(concept.contains("after=\"base\""));
    accept(root.path(), "project.md", id).unwrap();
    assert!(status(root.path(), "project.md").unwrap().is_empty());
    let authored = sources(root.path(), "project.md").unwrap();
    let output = evaluate_project("project.md", &authored, &ProjectTree::default())
        .unwrap()
        .new;
    materialize(root.path(), "project.md", &authored, &output).unwrap();
}
#[test]
fn moved_source_and_later_output_edits_cannot_be_implicitly_acknowledged() {
    let (root, original, tree) = setup();
    fs::write(root.path().join("app.py"), "print('first')\n").unwrap();
    let first = capture(root.path(), "project.md", &[]).unwrap().unwrap();
    fs::write(
        root.path().join("project.md"),
        original["project.md"].replace("hello", "first"),
    )
    .unwrap();
    assert!(
        new_patch(
            root.path(),
            "project.md",
            &first.id,
            "concept.md",
            "change",
            "Reason"
        )
        .unwrap_err()
        .contains("source changed")
    );
    let edited_sources = sources(root.path(), "project.md").unwrap();
    let edited_tree = evaluate_project("project.md", &edited_sources, &ProjectTree::default())
        .unwrap()
        .new;
    assert!(materialize(root.path(), "project.md", &edited_sources, &edited_tree).is_err());
    fs::write(root.path().join("app.py"), "print('second')\n").unwrap();
    assert!(
        accept(root.path(), "project.md", &first.id)
            .unwrap_err()
            .contains("output changed")
    );
    assert_eq!(status(root.path(), "project.md").unwrap().len(), 1);
    assert!(
        tree.get("app.py")
            .unwrap()
            .bytes
            .starts_with(b"print('hello')")
    );
}
#[test]
fn an_amended_or_split_arrangement_is_accepted_only_for_the_complete_capture() {
    let (root, original, _) = setup();
    fs::write(root.path().join("app.py"), "print('new')\n").unwrap();
    fs::write(root.path().join("helper.py"), "value = 1\n").unwrap();
    let edit = capture(root.path(), "project.md", &["helper.py".into()])
        .unwrap()
        .unwrap();
    let amended = original["project.md"].replace("hello", "new");
    fs::write(root.path().join("project.md"), &amended).unwrap();
    assert!(accept(root.path(), "project.md", &edit.id).is_err());
    let split = amended.replace("</h:doc>", "<h:include file=\"helper.md\"/></h:doc>");
    fs::write(root.path().join("project.md"), split).unwrap();
    fs::write(root.path().join("helper.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"helper.py\">value = 1\n</h:file></h:doc>").unwrap();
    accept(root.path(), "project.md", &edit.id).unwrap();
}
#[test]
fn binary_and_deleted_outputs_are_preserved_without_inventing_text_patches() {
    let (root, _, _) = setup();
    fs::write(root.path().join("app.py"), [0xff, 0x00]).unwrap();
    let edit = capture(root.path(), "project.md", &[]).unwrap().unwrap();
    assert!(edit.patch.is_none());
    assert!(edit.patch_error.unwrap().contains("binary"));
    assert_eq!(status(root.path(), "project.md").unwrap().len(), 1);
    fs::remove_file(root.path().join("app.py")).unwrap();
    let deletion = capture(root.path(), "project.md", &[]).unwrap().unwrap();
    assert!(deletion.patch.unwrap().contains("+++ /dev/null"));
    assert!(accept(root.path(), "project.md", &deletion.id).is_err());
    assert!(!root.path().join("app.py").exists());
}
#[test]
fn virtual_captures_preserve_unmaterialized_products_without_claiming_disk_writes() {
    let (root, source, tree) = setup();
    let mut before = tree.clone();
    before
        .insert("virtual.txt", ProjectEntry::file(b"not on disk\n".to_vec()))
        .unwrap();
    let mut after = before.clone();
    after
        .insert("app.py", ProjectEntry::file(b"print('mounted')\n".to_vec()))
        .unwrap();
    let capture = capture_virtual(root.path(), "project.md", &source, &before, &after).unwrap();
    assert!(!root.path().join("virtual.txt").exists());
    fs::write(root.path().join("app.py"), "print('mounted')\n").unwrap();
    let overlay = overlay(root.path(), "project.md").unwrap();
    assert_eq!(
        overlay["app.py"].as_deref(),
        Some(b"print('mounted')\n".as_slice())
    );
    // A supplied weave must still match the whole virtual result.
    assert!(accept_evaluated(root.path(), "project.md", &capture.id, &source, &tree).is_err());
    accept_evaluated(root.path(), "project.md", &capture.id, &source, &after).unwrap();
    assert!(status(root.path(), "project.md").unwrap().is_empty());
    assert!(!root.path().join("virtual.txt").exists());
}
#[test]
fn reverting_disk_still_requires_explicit_acceptance_and_preserves_history() {
    let (root, source, tree) = setup();
    fs::write(root.path().join("app.py"), "changed\n").unwrap();
    let changed = capture(root.path(), "project.md", &[]).unwrap().unwrap();
    fs::write(root.path().join("app.py"), "print('hello')\n").unwrap();
    let reverted = capture(root.path(), "project.md", &[]).unwrap().unwrap();
    assert_ne!(changed.id, reverted.id);
    assert!(reverted.paths.is_empty());
    assert!(materialize(root.path(), "project.md", &source, &tree).is_err());
    accept(root.path(), "project.md", &reverted.id).unwrap();
    assert!(status(root.path(), "project.md").unwrap().is_empty());
}
#[cfg(unix)]
#[test]
fn capture_and_publication_refuse_symlink_ancestors_and_unknown_preexisting_outputs() {
    let (root, sources, tree) = setup();
    let outside = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    fs::write(other.path().join("app.py"), "unrelated work").unwrap();
    assert!(
        materialize(other.path(), "project.md", &sources, &tree)
            .unwrap_err()
            .contains("no recorded baseline")
    );
    assert_eq!(
        fs::read_to_string(other.path().join("app.py")).unwrap(),
        "unrelated work"
    );
    std::os::unix::fs::symlink(outside.path(), root.path().join("linked")).unwrap();
    assert!(capture(root.path(), "project.md", &["linked/secret".into()]).is_err());
    fs::rename(
        root.path().join(".hick-journal"),
        root.path().join("journal-backup"),
    )
    .unwrap();
    std::os::unix::fs::symlink(outside.path(), root.path().join(".hick-journal")).unwrap();
    assert!(status(root.path(), "project.md").is_err());
    assert!(!outside.path().join("output-edits").exists());
}
