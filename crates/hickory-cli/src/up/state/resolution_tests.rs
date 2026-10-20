use super::*;

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
fn held(root: &Path) -> (WovenState, OutputResolution) {
    let path = root.join("out.txt");
    std::fs::write(&path, "disk side").unwrap();
    let mut state = WovenState::default();
    state.outputs.insert(
        path.clone(),
        OutputState {
            doc: root.join("doc.md"),
            rel_path: "out.txt".into(),
            content: "produced side".into(),
            provenance: vec![],
        },
    );
    state.hold_output(&path, "Cannot carry back".into());
    let request = OutputResolution {
        root: root.to_path_buf(),
        path,
        content: "merged".into(),
        expected_current: "disk side".into(),
        expected_produced: "produced side".into(),
    };
    (state, request)
}

#[test]
fn output_resolution_checks_both_observed_sides_and_preserves_holds_on_refusal() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let (mut state, mut request) = held(&root);
    std::fs::write(&request.path, "new disk").unwrap();
    assert!(
        state
            .resolve_output(&request)
            .unwrap_err()
            .to_string()
            .contains("changed on disk")
    );
    assert!(state.held.contains_key(&request.path));
    assert_eq!(std::fs::read_to_string(&request.path).unwrap(), "new disk");
    request.expected_current = "new disk".into();
    state.held.get_mut(&request.path).unwrap().theirs = "new produced".into();
    assert!(
        state
            .resolve_output(&request)
            .unwrap_err()
            .to_string()
            .contains("Produced output changed")
    );
    assert!(state.held.contains_key(&request.path));
    request.expected_produced = "new produced".into();
    state.resolve_output(&request).unwrap();
    assert_eq!(std::fs::read_to_string(&request.path).unwrap(), "merged");
    assert!(!state.held.contains_key(&request.path));
    assert!(state.resolve_output(&request).is_err());
}

#[cfg(unix)]
#[test]
fn output_resolution_refuses_a_symlink_retarget_outside_the_folder() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let target = outside.path().join("out.txt");
    std::fs::write(&target, "disk side").unwrap();
    let (mut state, request) = held(&root);
    std::fs::remove_file(&request.path).unwrap();
    std::os::unix::fs::symlink(&target, &request.path).unwrap();
    assert!(
        state
            .resolve_output(&request)
            .unwrap_err()
            .to_string()
            .contains("leaves the open folder")
    );
    assert_eq!(std::fs::read_to_string(target).unwrap(), "disk side");
    assert!(state.held.contains_key(&request.path));
}
