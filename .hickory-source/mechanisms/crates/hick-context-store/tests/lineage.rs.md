<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-context-store/tests/lineage.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-335caf8febec"></a>
## `crates/hick-context-store/tests/lineage.rs`

Related contracts: [Host context observations are immutable and changes are source-backed](../../../../guarantees/agent/immutable-context-lineage/contract.md), [Python model requests carry validated context manifests](../../../../guarantees/agent/python-requests-have-validated-context/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../guarantees/verification/predictions-precede-managed-observation/contract.md).

<a id="f-335caf8febec-service"></a>
### `service`

<storycode:copy id="f-335caf8febec-service">
use hick_context_protocol::*;
use hick_context_store::{host::Service, *};
use serde_json::json;
use std::{fs, path::Path};

fn service() -> (tempfile::TempDir, Service) {
    let root = tempfile::tempdir().unwrap();
    let service = Service::open(root.path(), &root.path().join("store.json"), "branch").unwrap();
    (root, service)
}
</storycode:copy>
<a id="f-335caf8febec-capture"></a>
### `capture`

<storycode:copy id="f-335caf8febec-capture">
fn capture(
    service: &mut Service,
    source: Source,
    text: &str,
    range: Option<Span>,
    key: &str,
) -> Node {
    service
        .store
        .capture(
            &CaptureRequest {
                idempotency_key: key.into(),
                source,
                span: range,
                max_bytes: 200_000,
            },
            text,
        )
        .unwrap()
        .0
}
</storycode:copy>
<a id="f-335caf8febec-saved"></a>
### `saved`

<storycode:copy id="f-335caf8febec-saved">
fn saved(service: &mut Service, root: &Path, text: &str, key: &str) -> Node {
    fs::write(root.join("note"), text).unwrap();
    let source = service.saved("note").unwrap();
    capture(service, source, text, None, key)
}
</storycode:copy>
[Read the test `external_updates_overlapping_reads_unicode_and_duplicates` beside its requirement](../../../../guarantees/agent/immutable-context-lineage/contract.md#f-335caf8febec-external_updates_overlapping_reads_unicode_and_duplicates).
[Read the test `editor_saved_rename_deleted_unavailable_and_instruction_scope` beside its requirement](../../../../guarantees/agent/immutable-context-lineage/contract.md#f-335caf8febec-editor_saved_rename_deleted_unavailable_and_instruction_scope).
[Read the test `restart_expired_cursor_recovery_pagination_replay_and_quota` beside its requirement](../../../../guarantees/agent/immutable-context-lineage/contract.md#f-335caf8febec-restart_expired_cursor_recovery_pagination_replay_and_quota).
[Read the test `derivation_validates_bytes_and_invalidates_transitively` beside its requirement](../../../../guarantees/agent/immutable-context-lineage/contract.md#f-335caf8febec-derivation_validates_bytes_and_invalidates_transitively).
[Read the test `real_capability_path_and_confinement` beside its requirement](../../../../guarantees/agent/immutable-context-lineage/contract.md#f-335caf8febec-real_capability_path_and_confinement).
[Read the test `bounded_events_chunks_and_ambiguous_overlapping_text` beside its requirement](../../../../guarantees/agent/immutable-context-lineage/contract.md#f-335caf8febec-bounded_events_chunks_and_ambiguous_overlapping_text).
[Read the test `negotiation_frames_and_empty_filtered_pages_fail_honestly` beside its requirement](../../../../guarantees/agent/immutable-context-lineage/contract.md#f-335caf8febec-negotiation_frames_and_empty_filtered_pages_fail_honestly).
[Read the test `dependency_epochs_and_preserved_normalization` beside its requirement](../../../../guarantees/agent/immutable-context-lineage/contract.md#f-335caf8febec-dependency_epochs_and_preserved_normalization).
[Read the test `retained_history_is_integrity_checked_and_single_writer` beside its requirement](../../../../guarantees/agent/immutable-context-lineage/contract.md#f-335caf8febec-retained_history_is_integrity_checked_and_single_writer).
[Read the test `a_unique_current_match_cannot_resolve_original_duplicates` beside its requirement](../../../../guarantees/agent/immutable-context-lineage/contract.md#f-335caf8febec-a_unique_current_match_cannot_resolve_original_duplicates).
[Read the test `model_response_retains_production_genealogy_after_restart` beside its requirement](../../../../guarantees/agent/python-requests-have-validated-context/contract.md#f-335caf8febec-model_response_retains_production_genealogy_after_restart).
[Read the test `retained_artifacts_reopen_exact_bytes_and_refuse_forged_descriptors_and_tampering` beside its requirement](../../../../guarantees/agent/immutable-context-lineage/contract.md#f-335caf8febec-retained_artifacts_reopen_exact_bytes_and_refuse_forged_descriptors_and_tampering).
<a id="f-335caf8febec-private_snapshot_context_never_refreshes_from_files_or_editors"></a>
### `private_snapshot_context_never_refreshes_from_files_or_editors`

<storycode:copy id="f-335caf8febec-private_snapshot_context_never_refreshes_from_files_or_editors">
// Guarantees: docs/guarantees/agent/immutable-context-lineage.md;
// docs/guarantees/verification/predictions-precede-managed-observation.md
#[test]
fn private_snapshot_context_never_refreshes_from_files_or_editors() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("private.json");
    fs::write(root.path().join("note"), "held-out answer").unwrap();
    let mut service = Service::open_snapshots(root.path(), &path, "private").unwrap();
    assert!(service.saved("note").is_err());
    assert!(service.editor("buffer", "held-out answer").is_err());
    let node = service
        .snapshots("saved_file", "note", "approved source", "approved")
        .unwrap()
        .remove(0);
    let hello = service.call("context.hello", &Service::offer()).unwrap();
    assert!(!hello.to_string().contains("held-out answer"));
    let source = service.store.sources().remove(0);
    let request = json!({"idempotency_key":"fresh-live-read","source":source,"max_bytes":100});
    assert!(
        service
            .call("context.capture", &request)
            .unwrap_err()
            .to_string()
            .contains("live capture withheld")
    );
    service.poll().unwrap();
    assert_eq!(
        service.store.node(&node.id).unwrap().text.as_deref(),
        Some("approved source")
    );
    let recovered = service.recovery_references(&[node.id]).unwrap();
    assert!(!recovered.to_string().contains("held-out answer"));
    drop(service);
    assert!(Service::open_snapshots(root.path(), &path, "private").is_err());
}
</storycode:copy>

### Assemble `crates/hick-context-store/tests/lineage.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-context-store/tests/lineage.rs"><storycode:paste select="#f-335caf8febec-service"/><storycode:paste select="#f-335caf8febec-capture"/><storycode:paste select="#f-335caf8febec-saved"/><storycode:paste select="#f-335caf8febec-external_updates_overlapping_reads_unicode_and_duplicates"/><storycode:paste select="#f-335caf8febec-editor_saved_rename_deleted_unavailable_and_instruction_scope"/><storycode:paste select="#f-335caf8febec-restart_expired_cursor_recovery_pagination_replay_and_quota"/><storycode:paste select="#f-335caf8febec-derivation_validates_bytes_and_invalidates_transitively"/><storycode:paste select="#f-335caf8febec-real_capability_path_and_confinement"/><storycode:paste select="#f-335caf8febec-bounded_events_chunks_and_ambiguous_overlapping_text"/><storycode:paste select="#f-335caf8febec-negotiation_frames_and_empty_filtered_pages_fail_honestly"/><storycode:paste select="#f-335caf8febec-dependency_epochs_and_preserved_normalization"/><storycode:paste select="#f-335caf8febec-retained_history_is_integrity_checked_and_single_writer"/><storycode:paste select="#f-335caf8febec-a_unique_current_match_cannot_resolve_original_duplicates"/><storycode:paste select="#f-335caf8febec-model_response_retains_production_genealogy_after_restart"/><storycode:paste select="#f-335caf8febec-retained_artifacts_reopen_exact_bytes_and_refuse_forged_descriptors_and_tampering"/><storycode:paste select="#f-335caf8febec-private_snapshot_context_never_refreshes_from_files_or_editors"/></storycode:file>

</storycode:doc>
