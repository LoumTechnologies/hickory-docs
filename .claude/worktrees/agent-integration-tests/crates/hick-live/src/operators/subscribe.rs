//! Subscribe operator for incremental collections.
//!
//! For each Add event, validates and dispatches data to an API sink backend.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use futures::StreamExt;
use hick_flow::Context;
use hick_flow::node::{BoxStream, Node, NodeTrace, NodeValue};
use hick_sink::{SinkBackend, SinkDef, SinkValidator};
use hick_token::ContainerCapabilities;

use crate::changes::ChangeType;
use crate::node::NodeExt;
use crate::schema::LiveSchema;

/// Binding from a sink slot to a column in the source data.
#[derive(Debug, Clone)]
pub struct SlotBinding {
    pub slot_name: String,
    pub column: String,
}

/// Dispatches source data to an API sink backend.
///
/// For each Add event: extract slot values from columns, validate against
/// token capabilities, and call the sink backend. Remove events are passed
/// through.
pub struct SubscribeNode {
    pub source: Arc<dyn Node>,
    pub sink_type: String,
    pub sink_def: SinkDef,
    pub slot_bindings: Vec<SlotBinding>,
    pub output_schema: LiveSchema,
    call_count: Arc<AtomicU64>,
}

impl SubscribeNode {
    pub fn new(
        source: Arc<dyn Node>,
        sink_type: String,
        sink_def: SinkDef,
        slot_bindings: Vec<SlotBinding>,
        input_schema: &LiveSchema,
    ) -> Self {
        Self {
            source,
            sink_type,
            sink_def,
            slot_bindings,
            output_schema: input_schema.clone(),
            call_count: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl Node for SubscribeNode {
    fn get_stream(self: Arc<Self>, context: Context) -> BoxStream<Vec<NodeTrace>> {
        let source_stream = self.source.clone().get_stream(context.clone());
        let sink_type = self.sink_type.clone();
        let sink_def = self.sink_def.clone();
        let slot_bindings = self.slot_bindings.clone();
        let output_schema = self.output_schema.clone();
        let call_count = self.call_count.clone();
        let me: Arc<dyn Node> = self;

        // Get backend and capabilities from context
        let backend = context.get_extension::<Arc<dyn SinkBackend>>();
        let capabilities = context.get_extension::<ContainerCapabilities>();

        Box::pin(source_stream.then(move |traces| {
            let backend = backend.clone();
            let capabilities = capabilities.clone();
            let sink_type = sink_type.clone();
            let sink_def = sink_def.clone();
            let slot_bindings = slot_bindings.clone();
            let output_schema = output_schema.clone();
            let call_count = call_count.clone();
            let me = me.clone();

            async move {
                let mut result_traces = Vec::new();

                for trace in traces {
                    if let Some(change) = trace.final_result().as_record_change() {
                        if change.change_type == ChangeType::Add {
                            if let (Some(backend), Some(caps)) = (&backend, &capabilities) {
                                let batch = &change.records;
                                let schema = batch.schema();

                                for row in 0..batch.num_rows() {
                                    let mut slot_values = HashMap::new();
                                    let mut slot_cls = HashMap::new();

                                    for binding in &slot_bindings {
                                        if let Some((col_idx, _)) =
                                            schema.column_with_name(&binding.column)
                                        {
                                            let col = batch.column(col_idx);
                                            let val =
                                                crate::operators::mask::value_to_string(col, row);
                                            slot_values.insert(
                                                binding.slot_name.clone(),
                                                serde_json::Value::String(val),
                                            );
                                        }

                                        if let Some(cls) =
                                            output_schema.classifications.get(&binding.column)
                                        {
                                            slot_cls.insert(binding.slot_name.clone(), cls.clone());
                                        }
                                    }

                                    let current = call_count.load(Ordering::SeqCst);

                                    let recipient: Option<&str> =
                                        slot_values.get("to").and_then(|v| v.as_str());

                                    if let Err(e) = SinkValidator::validate_call(
                                        caps,
                                        &sink_def,
                                        &slot_cls,
                                        current,
                                        "2026-02-17T00:00:00Z",
                                        recipient,
                                    ) {
                                        log::warn!(
                                            "Subscribe validation failed for {}: {}",
                                            sink_type,
                                            e
                                        );
                                        continue;
                                    }

                                    match backend.execute(&sink_type, &slot_values).await {
                                        Ok(_) => {
                                            call_count.fetch_add(1, Ordering::SeqCst);
                                        }
                                        Err(e) => {
                                            log::warn!(
                                                "Subscribe backend error for {}: {}",
                                                sink_type,
                                                e
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        // Pass through the trace regardless
                        result_traces.push(trace.add_to_trace(me.clone()));
                    } else {
                        result_traces.push(trace.add_to_trace(me.clone()));
                    }
                }
                result_traces
            }
        }))
    }

    fn node_value(&self) -> NodeValue {
        NodeValue::None
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StreamingStructuredDataNode;
    use crate::sources::memory::MemorySource;
    use arrow::array::{Int32Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use futures::StreamExt;
    use hick_classify::{Classification, ColumnClassifications};
    use hick_sink::{MockSinkBackend, SinkSlot};
    use serde_json::json;

    fn test_schema() -> (Arc<Schema>, LiveSchema) {
        let arrow = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("recipient", DataType::Utf8, false),
            Field::new("body", DataType::Utf8, false),
        ]));
        let mut cls = ColumnClassifications::new();
        cls.set(
            "body",
            Classification::new(vec![json!({"sensitivity": "low"})]),
        );
        cls.set(
            "recipient",
            Classification::new(vec![json!({"trust": "internal"})]),
        );
        let live =
            LiveSchema::new_strict(arrow.clone(), vec!["id".into()]).with_classifications(cls);
        (arrow, live)
    }

    fn test_batch() -> arrow::record_batch::RecordBatch {
        let (schema, _) = test_schema();
        arrow::record_batch::RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int32Array::from(vec![1])),
                Arc::new(StringArray::from(vec!["alice@co.com"])),
                Arc::new(StringArray::from(vec!["Hello"])),
            ],
        )
        .unwrap()
    }

    fn test_sink_def() -> SinkDef {
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

    #[tokio::test]
    async fn subscribe_records_mock_call() {
        let (_, live) = test_schema();

        let source = Arc::new(MemorySource::new(live.clone(), test_batch()));
        let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

        let bindings = vec![
            SlotBinding {
                slot_name: "to".to_string(),
                column: "recipient".to_string(),
            },
            SlotBinding {
                slot_name: "body".to_string(),
                column: "body".to_string(),
            },
        ];

        let subscribe = Arc::new(SubscribeNode::new(
            data_node,
            "email".to_string(),
            test_sink_def(),
            bindings,
            &live,
        ));

        let mock = Arc::new(MockSinkBackend::new());
        let caps = ContainerCapabilities::new()
            .allow_api_sink("email")
            .with_expires("2027-01-01T00:00:00Z");

        let mut ctx = Context::default();
        ctx.set_extension::<Arc<dyn SinkBackend>>(mock.clone() as Arc<dyn SinkBackend>);
        ctx.set_extension::<ContainerCapabilities>(caps);

        let events: Vec<_> = subscribe.get_stream(ctx).collect().await;
        assert!(!events.is_empty());
        assert_eq!(mock.call_count("email"), 1);
    }

    #[tokio::test]
    async fn subscribe_rejects_without_api_sink_permission() {
        let (_, live) = test_schema();

        let source = Arc::new(MemorySource::new(live.clone(), test_batch()));
        let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

        let bindings = vec![SlotBinding {
            slot_name: "to".to_string(),
            column: "recipient".to_string(),
        }];

        let subscribe = Arc::new(SubscribeNode::new(
            data_node,
            "email".to_string(),
            test_sink_def(),
            bindings,
            &live,
        ));

        let mock = Arc::new(MockSinkBackend::new());
        // No api-sink permission
        let caps = ContainerCapabilities::new().with_expires("2027-01-01T00:00:00Z");

        let mut ctx = Context::default();
        ctx.set_extension::<Arc<dyn SinkBackend>>(mock.clone() as Arc<dyn SinkBackend>);
        ctx.set_extension::<ContainerCapabilities>(caps);

        let _: Vec<_> = subscribe.get_stream(ctx).collect().await;
        // Mock should have 0 calls (validation failed)
        assert_eq!(mock.call_count("email"), 0);
    }
}
