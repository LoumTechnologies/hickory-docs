//! Integration tests for the capability-based data flow control system.
//!
//! These tests exercise the full pipeline: classified data sources → masking →
//! subscribe → mock API backend, all governed by macaroon-based capability tokens.

use std::sync::Arc;

use arrow::array::{AsArray, Int32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use futures::StreamExt;
use hick_classify::{Classification, ColumnClassifications, MaskSpec};
use hick_flow::Context;
use hick_flow::node::Node;
use hick_live::NodeExt;
use hick_live::operators::mask::MaskNode;
use hick_live::operators::subscribe::{SlotBinding, SubscribeNode};
use hick_live::sources::memory::MemorySource;
use hick_live::{LiveSchema, StreamingStructuredDataNode};
use hick_sink::{MockSinkBackend, SinkBackend, SinkDef, SinkSlot};
use hick_token::ContainerCapabilities;
use serde_json::json;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn classified_schema() -> (Arc<Schema>, LiveSchema) {
    let arrow = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("ssn", DataType::Utf8, false),
        Field::new("email", DataType::Utf8, false),
    ]));
    let mut cls = ColumnClassifications::new();
    cls.set(
        "ssn",
        Classification::new(vec![json!({"sensitivity": "secret", "type": "ssn"})]),
    );
    cls.set(
        "name",
        Classification::new(vec![json!({"sensitivity": "pii"})]),
    );
    cls.set(
        "email",
        Classification::new(vec![json!({"trust": "internal"})]),
    );
    let live = LiveSchema::new_strict(arrow.clone(), vec!["id".into()]).with_classifications(cls);
    (arrow, live)
}

fn test_batch() -> RecordBatch {
    let (schema, _) = classified_schema();
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int32Array::from(vec![1, 2])),
            Arc::new(StringArray::from(vec!["Alice", "Bob"])),
            Arc::new(StringArray::from(vec!["123-45-6789", "987-65-4321"])),
            Arc::new(StringArray::from(vec!["alice@co.com", "bob@co.com"])),
        ],
    )
    .unwrap()
}

fn email_sink_def() -> SinkDef {
    SinkDef {
        sink_type: "email".to_string(),
        slots: vec![
            SinkSlot {
                name: "to".to_string(),
                accepted_classifications: vec![Classification::new(vec![
                    json!({"trust": "internal"}),
                ])],
                rejected_classifications: vec![],
            },
            SinkSlot {
                name: "body".to_string(),
                accepted_classifications: vec![],
                rejected_classifications: vec![Classification::new(vec![
                    json!({"sensitivity": "secret"}),
                ])],
            },
        ],
        required_capabilities: vec![],
    }
}

fn build_ctx(mock: &Arc<MockSinkBackend>, caps: ContainerCapabilities) -> Context {
    let mut ctx = Context::default();
    ctx.set_extension::<Arc<dyn SinkBackend>>(mock.clone() as Arc<dyn SinkBackend>);
    ctx.set_extension::<ContainerCapabilities>(caps);
    ctx
}

// ---------------------------------------------------------------------------
// Test 1: Mask SSN → Select → verify classifications propagated
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mask_then_select_preserves_classifications() {
    let (_, live) = classified_schema();

    let source = Arc::new(MemorySource::new(live.clone(), test_batch()));
    let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

    // Mask SSN with hash
    let target_cls = Classification::new(vec![json!({"sensitivity": "pseudonymized"})]);
    let specs = vec![
        MaskSpec::hash("ssn")
            .with_salt("test")
            .with_target_classification(target_cls.clone()),
    ];
    let mask_node = Arc::new(MaskNode::new(data_node, specs, &live));

    // Verify classification was downgraded
    assert_eq!(
        mask_node.output_schema.classifications.get("ssn").unwrap(),
        &target_cls
    );
    // Name classification unchanged
    assert_eq!(
        mask_node.output_schema.classifications.get("name").unwrap(),
        &Classification::new(vec![json!({"sensitivity": "pii"})]),
    );

    // Execute and verify data flows
    let mut stream = mask_node.get_stream(Context::default());
    let traces = stream.next().await.unwrap();
    let node = traces[0].final_result();
    let change = node.as_record_change().unwrap();

    // SSN is hashed (not original values)
    let ssn_col = change.records.column(2).as_string::<i32>();
    assert_ne!(ssn_col.value(0), "123-45-6789");
    assert_eq!(ssn_col.value(0).len(), 64); // BLAKE3 hex string
}

// ---------------------------------------------------------------------------
// Test 2: Subscribe to email sink with MockSinkBackend
// ---------------------------------------------------------------------------

#[tokio::test]
async fn subscribe_email_records_correct_call() {
    let (_, live) = classified_schema();

    let source = Arc::new(MemorySource::new(live.clone(), test_batch()));
    let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

    let bindings = vec![
        SlotBinding {
            slot_name: "to".to_string(),
            column: "email".to_string(),
        },
        SlotBinding {
            slot_name: "body".to_string(),
            column: "name".to_string(),
        },
    ];

    let subscribe = Arc::new(SubscribeNode::new(
        data_node,
        "email".to_string(),
        email_sink_def(),
        bindings,
        &live,
    ));

    let mock = Arc::new(MockSinkBackend::new());
    let caps = ContainerCapabilities::new()
        .allow_api_sink("email")
        .with_expires("2027-01-01T00:00:00Z");

    let ctx = build_ctx(&mock, caps);
    let _: Vec<_> = subscribe.get_stream(ctx).collect().await;

    // Both rows should dispatch
    assert_eq!(mock.call_count("email"), 2);

    // Verify call content
    let calls = mock.calls();
    assert_eq!(
        calls[0].slot_values.get("to").unwrap(),
        &json!("alice@co.com")
    );
}

// ---------------------------------------------------------------------------
// Test 3: Token without api-sink permission → subscribe → rejection
// ---------------------------------------------------------------------------

#[tokio::test]
async fn subscribe_rejects_without_api_sink_permission() {
    let (_, live) = classified_schema();

    let source = Arc::new(MemorySource::new(live.clone(), test_batch()));
    let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

    let bindings = vec![SlotBinding {
        slot_name: "to".to_string(),
        column: "email".to_string(),
    }];

    let subscribe = Arc::new(SubscribeNode::new(
        data_node,
        "email".to_string(),
        email_sink_def(),
        bindings,
        &live,
    ));

    let mock = Arc::new(MockSinkBackend::new());
    // No api-sink permission
    let caps = ContainerCapabilities::new().with_expires("2027-01-01T00:00:00Z");

    let ctx = build_ctx(&mock, caps);
    let _: Vec<_> = subscribe.get_stream(ctx).collect().await;

    // Zero calls — validation rejected everything
    assert_eq!(mock.call_count("email"), 0);
}

// ---------------------------------------------------------------------------
// Test 4: Token with recipient restriction → wrong recipient → rejection
// ---------------------------------------------------------------------------

#[tokio::test]
async fn subscribe_rejects_wrong_recipient() {
    let (_, live) = classified_schema();

    let source = Arc::new(MemorySource::new(live.clone(), test_batch()));
    let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

    let bindings = vec![
        SlotBinding {
            slot_name: "to".to_string(),
            column: "email".to_string(),
        },
        SlotBinding {
            slot_name: "body".to_string(),
            column: "name".to_string(),
        },
    ];

    let subscribe = Arc::new(SubscribeNode::new(
        data_node,
        "email".to_string(),
        email_sink_def(),
        bindings,
        &live,
    ));

    let mock = Arc::new(MockSinkBackend::new());
    let caps = ContainerCapabilities::new()
        .allow_api_sink("email")
        .with_recipient("email", "alice@co.com")
        .with_expires("2027-01-01T00:00:00Z");

    let ctx = build_ctx(&mock, caps);
    let _: Vec<_> = subscribe.get_stream(ctx).collect().await;

    // Only alice@co.com allowed — bob@co.com rejected
    assert_eq!(mock.call_count("email"), 1);
    assert_eq!(
        mock.calls()[0].slot_values.get("to").unwrap(),
        &json!("alice@co.com")
    );
}

// ---------------------------------------------------------------------------
// Test 5: Token with max-calls=1 → 2 rows → second call rejected
// ---------------------------------------------------------------------------

#[tokio::test]
async fn subscribe_respects_max_calls() {
    let (_, live) = classified_schema();

    let source = Arc::new(MemorySource::new(live.clone(), test_batch()));
    let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

    let bindings = vec![SlotBinding {
        slot_name: "to".to_string(),
        column: "email".to_string(),
    }];

    let subscribe = Arc::new(SubscribeNode::new(
        data_node,
        "email".to_string(),
        email_sink_def(),
        bindings,
        &live,
    ));

    let mock = Arc::new(MockSinkBackend::new());
    let caps = ContainerCapabilities::new()
        .allow_api_sink("email")
        .with_max_calls("email", 1)
        .with_expires("2027-01-01T00:00:00Z");

    let ctx = build_ctx(&mock, caps);
    let _: Vec<_> = subscribe.get_stream(ctx).collect().await;

    // Only 1 call allowed
    assert_eq!(mock.call_count("email"), 1);
}

// ---------------------------------------------------------------------------
// Test 6: Full pipeline: source → mask → subscribe with classification
// ---------------------------------------------------------------------------

#[tokio::test]
async fn full_pipeline_mask_then_subscribe() {
    let (_, live) = classified_schema();

    let source = Arc::new(MemorySource::new(live.clone(), test_batch()));
    let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

    // Mask SSN so it's no longer "secret"
    let pseudonymized = Classification::new(vec![json!({"sensitivity": "pseudonymized"})]);
    let specs = vec![
        MaskSpec::hash("ssn")
            .with_salt("pipeline-salt")
            .with_target_classification(pseudonymized),
    ];
    let mask_node = Arc::new(MaskNode::new(data_node, specs, &live));

    // Subscribe — body is now name (pii, not secret) so sink should accept
    let bindings = vec![
        SlotBinding {
            slot_name: "to".to_string(),
            column: "email".to_string(),
        },
        SlotBinding {
            slot_name: "body".to_string(),
            column: "name".to_string(),
        },
    ];

    let masked_schema = mask_node.output_schema.clone();
    let subscribe = Arc::new(SubscribeNode::new(
        mask_node,
        "email".to_string(),
        email_sink_def(),
        bindings,
        &masked_schema,
    ));

    let mock = Arc::new(MockSinkBackend::new());
    let caps = ContainerCapabilities::new()
        .allow_api_sink("email")
        .with_expires("2027-01-01T00:00:00Z");

    let ctx = build_ctx(&mock, caps);
    let _: Vec<_> = subscribe.get_stream(ctx).collect().await;

    // Both rows dispatched through mask → subscribe
    assert_eq!(mock.call_count("email"), 2);
}

// ---------------------------------------------------------------------------
// Test 7: from_caveats roundtrip for ContainerCapabilities
// ---------------------------------------------------------------------------

#[test]
fn capabilities_from_caveats_roundtrip() {
    let original = ContainerCapabilities::new()
        .allow_api_sink("email")
        .with_max_calls("email", 5)
        .with_recipient("email", "alice@co.com")
        .with_expires("2027-01-01T00:00:00Z");

    let caveats = original.to_caveats();
    let restored = ContainerCapabilities::from_caveats(&caveats);

    assert!(restored.check_api_sink("email"));
    assert!(restored.check_max_calls("email", 4));
    assert!(!restored.check_max_calls("email", 5));
    assert!(restored.check_recipient("email", "alice@co.com"));
    assert!(!restored.check_recipient("email", "bob@co.com"));
    assert!(restored.check_expires("2026-06-01T00:00:00Z"));
}
