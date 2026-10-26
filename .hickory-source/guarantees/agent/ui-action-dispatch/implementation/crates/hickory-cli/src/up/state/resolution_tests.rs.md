<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/up/state/resolution_tests.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-3b41d39d831f"></a>
## `crates/hickory-cli/src/up/state/resolution_tests.rs`

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../../contract.md), [Keep divergent output until an explicit decision names the write](../../../../../../../../authoring/an-output-that-cannot-be-carried-back-is-held/contract.md).

[Read the test `held` beside its requirement](../../../../../../contract.md#f-3b41d39d831f-held).
<a id="f-3b41d39d831f-output_resolution_checks_both_observed_sides_and_preserves_holds_on_refusal"></a>
### `output_resolution_checks_both_observed_sides_and_preserves_holds_on_refusal`

<storycode:copy id="f-3b41d39d831f-output_resolution_checks_both_observed_sides_and_preserves_holds_on_refusal">
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
    state.outputs.get_mut(&request.path).unwrap().content = "new produced".into();
    state.resolve_output(&request).unwrap();
    assert_eq!(std::fs::read_to_string(&request.path).unwrap(), "merged");
    assert!(!state.held.contains_key(&request.path));
    assert!(state.resolve_output(&request).is_err());
}

</storycode:copy>
<a id="f-3b41d39d831f-output_resolution_refuses_a_symlink_retarget_outside_the_folder"></a>
### `output_resolution_refuses_a_symlink_retarget_outside_the_folder`

<storycode:copy id="f-3b41d39d831f-output_resolution_refuses_a_symlink_retarget_outside_the_folder">
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

</storycode:copy>
<a id="f-3b41d39d831f-output_regeneration_checks_observed_bytes_and_restores_read_only_generated_output"></a>
### `output_regeneration_checks_observed_bytes_and_restores_read_only_generated_output`

<storycode:copy id="f-3b41d39d831f-output_regeneration_checks_observed_bytes_and_restores_read_only_generated_output">
#[test]
fn output_regeneration_checks_observed_bytes_and_restores_read_only_generated_output() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let (mut state, mut request) = held(&root);
    request.mode = ResolutionMode::Regenerate;
    assert!(
        state
            .resolve_output(&request)
            .unwrap_err()
            .to_string()
            .contains("must use the observed")
    );
    assert_eq!(std::fs::read_to_string(&request.path).unwrap(), "disk side");
    request.content = request.expected_produced.clone();
    state.resolve_output(&request).unwrap();
    assert_eq!(
        std::fs::read_to_string(&request.path).unwrap(),
        "produced side"
    );
    assert!(!state.held.contains_key(&request.path));
    assert!(
        std::fs::metadata(&request.path)
            .unwrap()
            .permissions()
            .readonly()
    );
    set_read_only(&request.path, false).unwrap();
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/up/state/resolution_tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/up/state/resolution_tests.rs"><storycode:paste select="#f-3b41d39d831f-held"/><storycode:paste select="#f-3b41d39d831f-output_resolution_checks_both_observed_sides_and_preserves_holds_on_refusal"/><storycode:paste select="#f-3b41d39d831f-output_resolution_refuses_a_symlink_retarget_outside_the_folder"/><storycode:paste select="#f-3b41d39d831f-output_regeneration_checks_observed_bytes_and_restores_read_only_generated_output"/></storycode:file>

</storycode:doc>
