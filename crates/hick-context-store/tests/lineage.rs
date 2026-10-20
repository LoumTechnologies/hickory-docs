use hick_context_protocol::*;
use hick_context_store::{host::Service, *};
use serde_json::json;
use std::{fs, path::Path};

fn service() -> (tempfile::TempDir, Service) {
    let root = tempfile::tempdir().unwrap();
    let service = Service::open(root.path(), &root.path().join("store.json"), "branch").unwrap();
    (root, service)
}
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
fn saved(service: &mut Service, root: &Path, text: &str, key: &str) -> Node {
    fs::write(root.join("note"), text).unwrap();
    let source = service.saved("note").unwrap();
    capture(service, source, text, None, key)
}
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

// Guarantee: docs/guarantees/agent/python-requests-have-validated-context.md
#[test]
fn model_response_retains_production_genealogy_after_restart() {
    let (root, mut service) = service();
    let input = saved(&mut service, root.path(), "Requirement 📝", "input");
    assert!(
        service
            .store
            .model_response("bad", vec!["missing".into()], "Invented")
            .is_err()
    );
    let output = service
        .store
        .model_response("request-1", vec![input.id.clone()], "Summary")
        .unwrap();
    assert_eq!(output.derivation, DerivationKind::ModelOutput);
    assert_eq!(output.coverage, Coverage::Partial);
    assert_eq!(output.parents, vec![input.id]);
    let receipt = service.store.productions()[0].clone();
    assert_eq!(receipt.output, output.id);
    assert_eq!(
        service
            .store
            .node(&receipt.response)
            .unwrap()
            .text
            .as_deref(),
        Some("Summary")
    );
    drop(service);
    let reopened = Service::open(root.path(), &root.path().join("store.json"), "child").unwrap();
    assert_eq!(reopened.store.node(&output.id).unwrap(), &output);
    assert_eq!(reopened.store.productions()[0].request, "request-1");
}

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
