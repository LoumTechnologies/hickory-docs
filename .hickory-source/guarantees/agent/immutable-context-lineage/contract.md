<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Host context observations are immutable and changes are source-backed

For: Authors using an agent to change documents while retaining review control.

Given a retained host context session,
when Rust captures a saved file, an explicitly supplied editor snapshot or an
existing exact read receipt, or validates a derivation,
then it retains the exact returned UTF-8 bytes, excerpt digest, source identity,
whole-source revision hash and half-open byte range. Later observations never
replace those nodes. Capture records do not assert model delivery or exposure.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/immutable-context-lineage.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `external_updates_overlapping_reads_unicode_and_duplicates`

This is the maintained test fragment from `crates/hick-context-store/tests/lineage.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-335caf8febec-external_updates_overlapping_reads_unicode_and_duplicates"></a>
### `external_updates_overlapping_reads_unicode_and_duplicates`

<storycode:copy id="f-335caf8febec-external_updates_overlapping_reads_unicode_and_duplicates">
// Guarantee: docs/guarantees/agent/immutable-context-lineage.md
#[test]
fn external_updates_overlapping_reads_unicode_and_duplicates() {
    let (root, mut service) = service();
    let text = "first\r\n📝 target\r\nlast\n";
    fs::write(root.path().join("note"), text).unwrap();
    let source = service.saved("note").unwrap();
    let a = service
        .read_receipt(source.clone(), text, 2, 2, "📝 target\r\n", "read-1")
        .unwrap();
    let overlap = service
        .read_receipt(source, text, 2, 3, "📝 target\r\nlast\n", "read-2")
        .unwrap();
    let before = a.clone();
    let compare = |service: &Service, current: &Node, expected| {
        assert_eq!(
            service
                .store
                .compare(&CompareRequest {
                    old: a.id.clone(),
                    current: current.id.clone()
                })
                .unwrap()
                .state,
            expected
        );
    };
    let outside = saved(
        &mut service,
        root.path(),
        "first\r\n📝 target\r\nelse\n",
        "outside",
    );
    compare(&service, &outside, ComparisonState::Unchanged);
    let moved = saved(
        &mut service,
        root.path(),
        "inserted\nfirst\r\n📝 target\r\nlast\n",
        "moved",
    );
    compare(&service, &moved, ComparisonState::Moved);
    assert_eq!(
        service
            .store
            .compare(&CompareRequest {
                old: overlap.id,
                current: moved.id
            })
            .unwrap()
            .state,
        ComparisonState::Moved
    );
    let duplicate = saved(
        &mut service,
        root.path(),
        "📝 target\r\n📝 target\r\n",
        "duplicate",
    );
    compare(&service, &duplicate, ComparisonState::Ambiguous);
    let changed = saved(
        &mut service,
        root.path(),
        "first\r\n📝 changed\r\nlast\n",
        "changed",
    );
    compare(&service, &changed, ComparisonState::Changed);
    assert_eq!(service.store.node(&a.id).unwrap(), &before);
    assert_eq!(a.source_span, Some(span(7, 20)));
    let invalid = CaptureRequest {
        idempotency_key: "split-unicode".into(),
        source: changed.source.unwrap(),
        span: Some(span(8, 9)),
        max_bytes: 10,
    };
    assert!(
        service
            .store
            .capture(&invalid, "first\r\n📝 changed\r\nlast\n")
            .is_err()
    );
}
</storycode:copy>

## Test the behavior: `editor_saved_rename_deleted_unavailable_and_instruction_scope`

This is the maintained test fragment from `crates/hick-context-store/tests/lineage.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-335caf8febec-editor_saved_rename_deleted_unavailable_and_instruction_scope"></a>
### `editor_saved_rename_deleted_unavailable_and_instruction_scope`

<storycode:copy id="f-335caf8febec-editor_saved_rename_deleted_unavailable_and_instruction_scope">
// Guarantee: docs/guarantees/agent/immutable-context-lineage.md
#[test]
fn editor_saved_rename_deleted_unavailable_and_instruction_scope() {
    let (root, mut service) = service();
    let disk = saved(&mut service, root.path(), "saved", "disk");
    let editor = service.editor("buffer-1", "unsaved 📝").unwrap();
    let buffer = capture(&mut service, editor, "unsaved 📝", None, "buffer");
    assert_ne!(
        disk.source.as_ref().unwrap().id,
        buffer.source.as_ref().unwrap().id
    );
    service.poll().unwrap();
    assert_eq!(
        service.store.currentness(&buffer.id).unwrap(),
        ComparisonState::Unchanged
    );
    assert_eq!(
        fs::read_to_string(root.path().join("note")).unwrap(),
        "saved"
    );
    service.editors.0.insert(
        buffer.source.as_ref().unwrap().id.clone(),
        "new unsaved bytes".into(),
    );
    service.poll().unwrap();
    assert_eq!(
        service.store.currentness(&buffer.id).unwrap(),
        ComparisonState::Changed
    );
    assert_eq!(
        service.store.currentness(&disk.id).unwrap(),
        ComparisonState::Unchanged
    );
    assert_eq!(
        service.store.node(&buffer.id).unwrap().text.as_deref(),
        Some("unsaved 📝")
    );
    let source_id = disk.source.as_ref().unwrap().id.clone();
    fs::rename(root.path().join("note"), root.path().join("renamed")).unwrap();
    service.store.relocate(&source_id, "renamed").unwrap();
    let renamed = service.saved("renamed").unwrap();
    assert_eq!(renamed.id, source_id);
    let renamed = capture(&mut service, renamed, "saved", None, "rename");
    assert_eq!(
        service
            .store
            .compare(&CompareRequest {
                old: disk.id.clone(),
                current: renamed.id
            })
            .unwrap()
            .state,
        ComparisonState::Moved
    );
    fs::remove_file(root.path().join("renamed")).unwrap();
    service.editors.0.clear();
    service.poll().unwrap();
    assert_eq!(
        service.store.currentness(&disk.id).unwrap(),
        ComparisonState::Unavailable
    );
    assert_eq!(
        service.store.currentness(&buffer.id).unwrap(),
        ComparisonState::CannotCheck
    );
    service
        .store
        .instruction(Instruction {
            node: buffer.id.clone(),
            role: "system".into(),
            order: 2,
            directory: "app".into(),
        })
        .unwrap();
    service
        .store
        .instruction(Instruction {
            node: disk.id,
            role: "developer".into(),
            order: 1,
            directory: "".into(),
        })
        .unwrap();
    assert_eq!(
        service
            .store
            .applicable(Path::new("app/task.md"))
            .unwrap()
            .iter()
            .map(|i| i.order)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(
        service
            .store
            .applicable(Path::new("other/task.md"))
            .unwrap()
            .len(),
        1
    );
    assert!(service.store.applicable(Path::new("../escape")).is_err());
    // Capture receipt is deliberately not an assertion of model delivery.
    assert!(
        service
            .store
            .captures()
            .iter()
            .all(|r| r.state == "captured")
    );
}
</storycode:copy>

## Test the behavior: `restart_expired_cursor_recovery_pagination_replay_and_quota`

This is the maintained test fragment from `crates/hick-context-store/tests/lineage.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-335caf8febec-restart_expired_cursor_recovery_pagination_replay_and_quota"></a>
### `restart_expired_cursor_recovery_pagination_replay_and_quota`

<storycode:copy id="f-335caf8febec-restart_expired_cursor_recovery_pagination_replay_and_quota">
// Guarantee: docs/guarantees/agent/immutable-context-lineage.md
#[test]
fn restart_expired_cursor_recovery_pagination_replay_and_quota() {
    let (root, mut service) = service();
    let node = saved(&mut service, root.path(), "old", "one");
    service.poll().unwrap();
    let cursor = service.store.cursor("branch");
    fs::write(root.path().join("note"), "pull-like replacement").unwrap();
    service.poll().unwrap();
    let request = ChangesRequest {
        cursor: cursor.clone(),
        references: vec![node.id.clone()],
        limit: 1,
    };
    let page = service.store.changes(&request, "branch").unwrap();
    assert_eq!(page.events.len(), 1);
    validate_page(
        &serde_json::to_value(&page).unwrap(),
        &cursor.epoch,
        "branch",
    )
    .unwrap();
    assert!(service.store.changes(&request, "wrong-branch").is_err());
    let original = node.clone();
    let path = root.path().join("store.json");
    drop(service);
    let mut service = Service::open(root.path(), &path, "branch").unwrap();
    assert!(service.store.changes(&request, "branch").is_err());
    service.recovery().unwrap();
    assert_eq!(
        service.store.currentness(&node.id).unwrap(),
        ComparisonState::Changed
    );
    assert_eq!(service.store.node(&node.id).unwrap(), &original);
    let capture_request = CaptureRequest {
        idempotency_key: "one".into(),
        source: node.source.unwrap(),
        span: None,
        max_bytes: 200_000,
    };
    // Restart replay returns the original result, even though the source has changed.
    service.call("context.hello", &Service::offer()).unwrap();
    let replay = service
        .call(
            "context.capture",
            &serde_json::to_value(&capture_request).unwrap(),
        )
        .unwrap();
    assert_eq!(replay["node"]["text"], "old");
    let conflicting = CaptureRequest {
        max_bytes: 10,
        ..capture_request
    };
    assert!(service.store.capture(&conflicting, "old").is_err());
    drop(service);
    let len = fs::metadata(&path).unwrap().len() as usize;
    let mut store = Store::open(&path, len + 20, 1).unwrap();
    assert!(
        store
            .register("saved_file", "another-long-locator")
            .is_err()
    );
    assert_eq!(store.nodes().len(), 1);
}
</storycode:copy>

## Test the behavior: `derivation_validates_bytes_and_invalidates_transitively`

This is the maintained test fragment from `crates/hick-context-store/tests/lineage.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-335caf8febec-derivation_validates_bytes_and_invalidates_transitively"></a>
### `derivation_validates_bytes_and_invalidates_transitively`

<storycode:copy id="f-335caf8febec-derivation_validates_bytes_and_invalidates_transitively">
// Guarantee: docs/guarantees/agent/immutable-context-lineage.md
#[test]
fn derivation_validates_bytes_and_invalidates_transitively() {
    let (root, mut service) = service();
    let root_node = saved(&mut service, root.path(), "hello 📝", "root");
    service.poll().unwrap();
    let node = Node {
        id: "derived".into(),
        digest: digest(b"hello"),
        text: Some("hello".into()),
        artifact: None,
        source: None,
        source_span: None,
        parents: vec![root_node.id.clone()],
        derivation: DerivationKind::Selection,
        coverage: Coverage::Complete,
        mappings: vec![Mapping {
            parent: root_node.id.clone(),
            input: span(0, 5),
            output: span(0, 5),
        }],
        receipt: None,
    };
    let request = DeriveRequest {
        idempotency_key: "derive".into(),
        node: node.clone(),
    };
    service.store.derive(&request).unwrap();
    assert_eq!(
        service.store.currentness(&node.id).unwrap(),
        ComparisonState::Unchanged
    );
    let mut opaque = node.clone();
    opaque.id = "opaque-dependency".into();
    opaque.derivation = DerivationKind::Opaque;
    opaque.coverage = Coverage::Opaque;
    opaque.mappings.clear();
    service
        .store
        .derive(&DeriveRequest {
            idempotency_key: "opaque-derive".into(),
            node: opaque.clone(),
        })
        .unwrap();
    assert_eq!(
        service.store.currentness(&opaque.id).unwrap(),
        ComparisonState::CannotCheck
    );
    let mut forged = node.clone();
    forged.id = "forged".into();
    forged.text = Some("wrong".into());
    forged.digest = digest(b"wrong");
    assert!(
        service
            .store
            .derive(&DeriveRequest {
                idempotency_key: "bad".into(),
                node: forged
            })
            .is_err()
    );
    let mut replacement = request.clone();
    replacement.idempotency_key = "replace".into();
    assert!(service.store.derive(&replacement).is_err());
    fs::write(root.path().join("note"), "new requirement").unwrap();
    service.poll().unwrap();
    assert_eq!(
        service.store.currentness(&node.id).unwrap(),
        ComparisonState::Invalidated
    );
    assert_eq!(
        service.store.currentness(&opaque.id).unwrap(),
        ComparisonState::Invalidated
    );
    assert_eq!(service.store.node(&node.id).unwrap(), &node);
}
</storycode:copy>

## Test the behavior: `real_capability_path_and_confinement`

This is the maintained test fragment from `crates/hick-context-store/tests/lineage.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-335caf8febec-real_capability_path_and_confinement"></a>
### `real_capability_path_and_confinement`

<storycode:copy id="f-335caf8febec-real_capability_path_and_confinement">
// Guarantee: docs/guarantees/agent/immutable-context-lineage.md
#[test]
fn real_capability_path_and_confinement() {
    let (root, mut service) = service();
    let node = saved(&mut service, root.path(), "one\ntwo 📝\n", "initial");
    let params = json!({"idempotency_key":"rpc","source":node.source,"span":{"start":4,"end":13,"unit":"utf8_bytes"},"max_bytes":100});
    assert!(service.call("context.capture", &params).is_err());
    let hello = service.call("context.hello", &Service::offer()).unwrap();
    let result = service.call("context.capture", &params).unwrap();
    assert_eq!(result["node"]["text"], "two 📝\n");
    let cursor: Cursor = serde_json::from_value(hello["cursor"].clone()).unwrap();
    fs::write(root.path().join("note"), "external update").unwrap();
    let changes = service
        .call(
            "context.changes",
            &json!({"cursor":cursor,"references":[node.id],"limit":1}),
        )
        .unwrap();
    assert_eq!(changes["events"].as_array().unwrap().len(), 1);
    assert!(service.saved("../outside").is_err());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("/etc/passwd", root.path().join("escape")).unwrap();
        assert!(service.saved("escape").is_err());
    }
}

</storycode:copy>

## Test the behavior: `bounded_events_chunks_and_ambiguous_overlapping_text`

This is the maintained test fragment from `crates/hick-context-store/tests/lineage.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-335caf8febec-bounded_events_chunks_and_ambiguous_overlapping_text"></a>
### `bounded_events_chunks_and_ambiguous_overlapping_text`

<storycode:copy id="f-335caf8febec-bounded_events_chunks_and_ambiguous_overlapping_text">
// Guarantee: docs/guarantees/agent/immutable-context-lineage.md
#[test]
fn bounded_events_chunks_and_ambiguous_overlapping_text() {
    struct Memory(String);
    impl SourceAdapter for Memory {
        fn head(&mut self, source: &Source) -> anyhow::Result<Head> {
            Ok(Head::Present {
                source: Source {
                    revision: digest(self.0.as_bytes()),
                    ..source.clone()
                },
                text: self.0.clone(),
            })
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(&dir.path().join("bounded.json"), 2_000_000, 1).unwrap();
    let mut source = store.register("external", "memory").unwrap();
    source.revision = digest(b"aa");
    let old = store
        .import_read(source.clone(), span(0, 2), "aa", "old")
        .unwrap();
    let mut adapter = Memory("aa".into());
    store.reconcile(&mut adapter).unwrap();
    let cursor = store.cursor("branch");
    for text in ["other", "aaa"] {
        adapter.0 = text.into();
        store.reconcile(&mut adapter).unwrap();
    }
    assert!(
        store
            .changes(
                &ChangesRequest {
                    cursor,
                    references: vec![old.id.clone()],
                    limit: 1
                },
                "branch"
            )
            .is_err()
    );
    source.revision = digest(b"aaa");
    let current = store
        .import_read(source.clone(), span(0, 3), "aaa", "current")
        .unwrap();
    assert_eq!(
        store
            .compare(&CompareRequest {
                old: old.id.clone(),
                current: current.id
            })
            .unwrap()
            .state,
        ComparisonState::Ambiguous
    );
    let partial = store
        .import_read(source.clone(), span(1, 3), "aa", "partial")
        .unwrap();
    assert_eq!(
        store
            .compare(&CompareRequest {
                old: old.id,
                current: partial.id
            })
            .unwrap()
            .state,
        ComparisonState::CannotCheck
    );
    let large = "📝\r\n".repeat(50_000);
    source.revision = digest(large.as_bytes());
    let chunks = store
        .import_chunks(source, span(0, large.len()), &large, "chunks")
        .unwrap();
    assert!(chunks.len() > 1);
    assert_eq!(
        chunks
            .iter()
            .map(|n| n.text.as_deref().unwrap())
            .collect::<String>(),
        large
    );
    assert_eq!(
        chunks.first().unwrap().source_span.as_ref().unwrap().start,
        0
    );
    assert_eq!(
        chunks.last().unwrap().source_span.as_ref().unwrap().end,
        large.len() as u64
    );
    for pair in chunks.windows(2) {
        assert_eq!(
            pair[0].source_span.as_ref().unwrap().end,
            pair[1].source_span.as_ref().unwrap().start
        );
    }
    let second = store.register("external", "different").unwrap();
    assert_ne!(second.id, chunks[0].source.as_ref().unwrap().id);
}

</storycode:copy>

## Test the behavior: `negotiation_frames_and_empty_filtered_pages_fail_honestly`

This is the maintained test fragment from `crates/hick-context-store/tests/lineage.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-335caf8febec-negotiation_frames_and_empty_filtered_pages_fail_honestly"></a>
### `negotiation_frames_and_empty_filtered_pages_fail_honestly`

<storycode:copy id="f-335caf8febec-negotiation_frames_and_empty_filtered_pages_fail_honestly">
// Guarantee: docs/guarantees/agent/immutable-context-lineage.md
#[test]
fn negotiation_frames_and_empty_filtered_pages_fail_honestly() {
    let (root, mut service) = service();
    let observed = saved(&mut service, root.path(), "observed", "observed");
    let hello = service.call("context.hello", &Service::offer()).unwrap();
    let cursor: Cursor = serde_json::from_value(hello["cursor"].clone()).unwrap();
    fs::write(root.path().join("other"), "other").unwrap();
    service.saved("other").unwrap();
    service.poll().unwrap();
    let page = service
        .store
        .changes(
            &ChangesRequest {
                cursor: cursor.clone(),
                references: vec![observed.id],
                limit: 1,
            },
            "branch",
        )
        .unwrap();
    assert!(page.events.is_empty());
    validate_page(
        &serde_json::to_value(page).unwrap(),
        &cursor.epoch,
        "branch",
    )
    .unwrap();
    let mut tiny = Service::offer();
    tiny["max_frame_bytes"] = json!(128);
    assert!(service.call("context.hello", &tiny).is_err());
    assert!(service.call("context.resolve", &json!({})).is_err());
}

</storycode:copy>

## Test the behavior: `dependency_epochs_and_preserved_normalization`

This is the maintained test fragment from `crates/hick-context-store/tests/lineage.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-335caf8febec-dependency_epochs_and_preserved_normalization"></a>
### `dependency_epochs_and_preserved_normalization`

<storycode:copy id="f-335caf8febec-dependency_epochs_and_preserved_normalization">
// Guarantee: docs/guarantees/agent/immutable-context-lineage.md
#[test]
fn dependency_epochs_and_preserved_normalization() {
    let (root, mut service) = service();
    let original = saved(&mut service, root.path(), "e\u{301}\r\n", "normal-source");
    let normalized = Node {
        id: "normalized".into(),
        digest: digest("é\n".as_bytes()),
        text: Some("é\n".into()),
        artifact: None,
        source: None,
        source_span: None,
        parents: vec![original.id.clone()],
        derivation: DerivationKind::Normalization,
        coverage: Coverage::Complete,
        mappings: vec![Mapping {
            parent: original.id.clone(),
            input: span(0, 5),
            output: span(0, 3),
        }],
        receipt: None,
    };
    service
        .store
        .derive(&DeriveRequest {
            idempotency_key: "normalize".into(),
            node: normalized.clone(),
        })
        .unwrap();
    service.poll().unwrap();
    let path = root.path().join("store.json");
    drop(service);
    let mut service = Service::open(root.path(), &path, "branch").unwrap();
    assert_eq!(
        service.store.currentness(&original.id).unwrap(),
        ComparisonState::CannotCheck
    );
    service.recovery().unwrap();
    assert_eq!(
        service.store.currentness(&normalized.id).unwrap(),
        ComparisonState::Unchanged
    );
    assert_eq!(
        service.store.node(&original.id).unwrap().text.as_deref(),
        Some("e\u{301}\r\n")
    );
    fs::write(root.path().join("note"), "changed").unwrap();
    service.poll().unwrap();
    assert_eq!(
        service.store.currentness(&normalized.id).unwrap(),
        ComparisonState::Invalidated
    );
}

</storycode:copy>

## Test the behavior: `retained_history_is_integrity_checked_and_single_writer`

This is the maintained test fragment from `crates/hick-context-store/tests/lineage.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-335caf8febec-retained_history_is_integrity_checked_and_single_writer"></a>
### `retained_history_is_integrity_checked_and_single_writer`

<storycode:copy id="f-335caf8febec-retained_history_is_integrity_checked_and_single_writer">
// Guarantee: docs/guarantees/agent/immutable-context-lineage.md
#[test]
fn retained_history_is_integrity_checked_and_single_writer() {
    let (root, mut service) = service();
    let original = saved(&mut service, root.path(), "original", "original");
    let path = root.path().join("store.json");
    assert!(Store::open(&path, 64 * 1024 * 1024, 1024).is_err());
    drop(service);
    let mut data: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    data["nodes"][0]["text"] = json!("tampered");
    fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
    assert!(Store::open(&path, 64 * 1024 * 1024, 1024).is_err());
    assert_eq!(original.text.as_deref(), Some("original"));
}

</storycode:copy>

## Test the behavior: `a_unique_current_match_cannot_resolve_original_duplicates`

This is the maintained test fragment from `crates/hick-context-store/tests/lineage.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-335caf8febec-a_unique_current_match_cannot_resolve_original_duplicates"></a>
### `a_unique_current_match_cannot_resolve_original_duplicates`

<storycode:copy id="f-335caf8febec-a_unique_current_match_cannot_resolve_original_duplicates">
// Guarantee: docs/guarantees/agent/immutable-context-lineage.md
#[test]
fn a_unique_current_match_cannot_resolve_original_duplicates() {
    let (root, mut service) = service();
    let old_text = "same\nmiddle\nsame\n";
    fs::write(root.path().join("note"), old_text).unwrap();
    let source = service.saved("note").unwrap();
    let observed = service
        .read_receipt(source, old_text, 1, 1, "same\n", "duplicate-original")
        .unwrap();
    let current = saved(
        &mut service,
        root.path(),
        "edited\nmiddle\nsame\n",
        "only-other-copy-survives",
    );
    let comparison = service
        .store
        .compare(&CompareRequest {
            old: observed.id,
            current: current.id,
        })
        .unwrap();
    assert_eq!(comparison.state, ComparisonState::Ambiguous);
    assert!(comparison.mappings.is_empty());
    assert!(!unique_passage("aaa", "aa"));
}

</storycode:copy>

## Test the behavior: `retained_artifacts_reopen_exact_bytes_and_refuse_forged_descriptors_and_tampering`

This is the maintained test fragment from `crates/hick-context-store/tests/lineage.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-335caf8febec-retained_artifacts_reopen_exact_bytes_and_refuse_forged_descriptors_and_tampering"></a>
### `retained_artifacts_reopen_exact_bytes_and_refuse_forged_descriptors_and_tampering`

<storycode:copy id="f-335caf8febec-retained_artifacts_reopen_exact_bytes_and_refuse_forged_descriptors_and_tampering">
// Guarantee: docs/guarantees/agent/immutable-context-lineage.md
#[test]
fn retained_artifacts_reopen_exact_bytes_and_refuse_forged_descriptors_and_tampering() {
    let (root, mut service) = service();
    let bytes = b"immutable image fixture bytes";
    let node = service
        .store
        .import_artifact("ui_image", "capture", "image/png", bytes, "image-read")
        .unwrap();
    assert_eq!(
        service
            .store
            .import_artifact("ui_image", "capture", "image/png", bytes, "image-read")
            .unwrap(),
        node
    );
    assert!(
        service
            .store
            .import_artifact("ui_image", "capture", "image/png", b"changed", "image-read")
            .is_err()
    );
    let mut fabricated = node.clone();
    fabricated.id = fresh_id();
    fabricated.source = None;
    fabricated.receipt = None;
    assert!(
        service
            .store
            .derive(&DeriveRequest {
                idempotency_key: "worker-image".into(),
                node: fabricated
            })
            .is_err()
    );
    let artifact = node.artifact.unwrap();
    assert_eq!(service.store.artifact_bytes(&artifact).unwrap(), bytes);
    let mut forged = artifact.clone();
    forged.locator = "/etc/passwd".into();
    assert!(service.store.artifact_bytes(&forged).is_err());
    drop(service);
    let reopened = Service::open(root.path(), &root.path().join("store.json"), "branch").unwrap();
    assert_eq!(reopened.store.artifact_bytes(&artifact).unwrap(), bytes);
    drop(reopened);
    let path = root.path().join("store.json");
    let mut data: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    data["artifacts"][&artifact.digest][0] = json!(0);
    fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
    assert!(Service::open(root.path(), &path, "branch").is_err());
}

</storycode:copy>

<a id="output-9f857d07870a"></a>
## `crates/hickory-agent/src/tools/receipts.rs`

Related contracts: [Host context observations are immutable and changes are source-backed](contract.md).

<a id="f-9f857d07870a-ContextRead"></a>
### `ContextRead`

<storycode:copy id="f-9f857d07870a-ContextRead">
use super::*;

/// One thing a tool showed the model: a file, at a content hash and (when
/// the file is in a git repository) a commit, over a 1-based inclusive line
/// range. Derived by the tool, never declared by the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextRead {
    /// The path as the session knows it — the document's name for the
    /// primary, the upstream's path, an output's path, or the path given to
    /// `read_file` resolved relative to the primary document.
    pub path: String,
    /// Exact host-resolved source path when available. `path` above remains the
    /// historical citation locator and may be relative to a different tool root.
    pub source_path: Option<std::path::PathBuf>,
    /// `git rev-parse HEAD` of the repository containing the file, if any.
    /// The working tree may differ from HEAD; `sha256` is the truth about
    /// the bytes, `commit` is where to look for them later.
    pub commit: Option<String>,
    /// SHA-256 of the whole file's bytes as read.
    pub sha256: String,
    /// First line shown, 1-based.
    pub first_line: usize,
    /// Last line shown, 1-based, inclusive.
    pub last_line: usize,
    /// Exact returned source bytes, retained for historical inspection.
    pub content: String,
    /// Half-open UTF-8 byte range in the exact snapshot hashed above.
    pub byte_span: (usize, usize),
    /// Uniqueness witnessed in the original full snapshot, including overlapping matches.
    pub unique: bool,
}

</storycode:copy>
<a id="f-9f857d07870a-Wrote"></a>
### `Wrote`

The lines an edit left in a file, 1-based inclusive, with their content
hashes — the anchors by which a later reader finds them again after the
file has moved on.

<storycode:copy id="f-9f857d07870a-Wrote">
/// The lines an edit left in a file, 1-based inclusive, with their content
/// hashes — the anchors by which a later reader finds them again after the
/// file has moved on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wrote {
    pub file: String,
    pub first_line: usize,
    pub last_line: usize,
    /// SHA-256 hashline hashes of those lines, in order.
    pub hashes: Vec<String>,
    /// SHA-256 per written line; short anchors are never attribution evidence.
    pub fingerprints: Vec<String>,
}

</storycode:copy>
<a id="f-9f857d07870a-sha256_hex"></a>
### `sha256_hex`

<storycode:copy id="f-9f857d07870a-sha256_hex">
/// SHA-256 of `bytes`, lowercase hex.
pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest as _;
    let mut h = sha2::Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

</storycode:copy>
<a id="f-9f857d07870a-head_commit_for"></a>
### `head_commit_for`

`git rev-parse HEAD` for the repository containing `path`, if there is
one and git is installed. Anything else is `None`: outside a repository
there is no commit to name, and the read is still recorded by its hash.

<storycode:copy id="f-9f857d07870a-head_commit_for">
/// `git rev-parse HEAD` for the repository containing `path`, if there is
/// one and git is installed. Anything else is `None`: outside a repository
/// there is no commit to name, and the read is still recorded by its hash.
pub(super) fn head_commit_for(path: &Path) -> Option<String> {
    let dir = if path.is_dir() {
        path
    } else {
        match path.parent() {
            Some(p) if !p.as_os_str().is_empty() => p,
            _ => Path::new("."),
        }
    };
    let out = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!s.is_empty()).then_some(s)
}

</storycode:copy>
<a id="f-9f857d07870a-changed_lines"></a>
### `changed_lines`

The lines of `new` that differ from `old`: the first and last (1-based,
inclusive) line indices into `new` outside the common prefix and suffix,
with their hashes. `None` when nothing changed or the change was a pure
deletion (no lines of `new` were written).

<storycode:copy id="f-9f857d07870a-changed_lines">
/// The lines of `new` that differ from `old`: the first and last (1-based,
/// inclusive) line indices into `new` outside the common prefix and suffix,
/// with their hashes. `None` when nothing changed or the change was a pure
/// deletion (no lines of `new` were written).
pub(super) fn changed_lines(old: &str, new: &str, file: &str) -> Option<Wrote> {
    let a: Vec<&str> = old.lines().collect();
    let b: Vec<&str> = new.lines().collect();
    let mut prefix = 0;
    while prefix < a.len() && prefix < b.len() && a[prefix] == b[prefix] {
        prefix += 1;
    }
    let mut suffix = 0;
    while suffix < a.len() - prefix
        && suffix < b.len() - prefix
        && a[a.len() - 1 - suffix] == b[b.len() - 1 - suffix]
    {
        suffix += 1;
    }
    let last = b.len().checked_sub(suffix)?;
    if last <= prefix {
        return None;
    }
    Some(Wrote {
        file: file.to_string(),
        first_line: prefix + 1,
        last_line: last,
        fingerprints: b[prefix..last]
            .iter()
            .map(|line| hashline::line_fingerprint(line))
            .collect(),
        hashes: b[prefix..last]
            .iter()
            .map(|l| hashline::line_hash(l))
            .collect(),
    })
}

</storycode:copy>
<a id="f-9f857d07870a-impl-super-EditSession"></a>
### `impl-super::EditSession`

<storycode:copy id="f-9f857d07870a-impl-super-EditSession">
impl super::EditSession {
    /// A context record for showing the whole of `source`, known as `path`.
    pub(super) fn read_of(&self, path: &std::path::Path, name: &str, source: &str) -> ContextRead {
        let lines = source.lines().count().max(1);
        ContextRead {
            path: name.to_string(),
            source_path: Some(path.to_path_buf()),
            commit: super::head_commit_for(path),
            sha256: super::sha256_hex(source.as_bytes()),
            first_line: 1,
            last_line: lines,
            content: source.to_string(),
            byte_span: (0, source.len()),
            unique: !source.is_empty(),
        }
    }
}
</storycode:copy>

### Assemble `crates/hickory-agent/src/tools/receipts.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-agent/src/tools/receipts.rs"><storycode:paste select="#f-9f857d07870a-ContextRead"/><storycode:paste select="#f-9f857d07870a-Wrote"/><storycode:paste select="#f-9f857d07870a-sha256_hex"/><storycode:paste select="#f-9f857d07870a-head_commit_for"/><storycode:paste select="#f-9f857d07870a-changed_lines"/><storycode:paste select="#f-9f857d07870a-impl-super-EditSession"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-context-store/src/host.rs](../../../mechanisms/crates/hick-context-store/src/host.rs.md#output-c4e1ee6d700e)
- [crates/hick-context-store/src/lib.rs](../../../mechanisms/crates/hick-context-store/src/lib.rs.md#output-86b54fda68eb)
- [crates/hickory-agent/src/tools/read_file.rs](../../../mechanisms/crates/hickory-agent/src/tools/read_file.rs.md#output-7845fffcfd2e)
- [crates/hickory-agent/src/tools/receipts.rs](contract.md#output-9f857d07870a)
- [crates/hickory-cli/src/python_agent.rs](../../../mechanisms/crates/hickory-cli/src/python_agent.rs.md#output-609d98dd1fad)
- [crates/hickory-cli/src/serve/acp/read_receipt.rs](../../../mechanisms/crates/hickory-cli/src/serve/acp/read_receipt.rs.md#output-bc08d0c441ad)
- [crates/hickory-cli/src/serve/python_agent.rs](../../../mechanisms/crates/hickory-cli/src/serve/python_agent.rs.md#output-89d47ddc390d)

### Tests and fixtures

- [crates/hick-context-store/tests/lineage.rs](../../../mechanisms/crates/hick-context-store/tests/lineage.rs.md#output-335caf8febec)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
