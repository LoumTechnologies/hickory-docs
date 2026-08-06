//! Utility for consuming a node's stream and returning the fully-converged output.

use std::sync::Arc;

use futures::StreamExt;

use crate::context::Context;
use crate::node::{BinaryData, FileContent, Node, NodeValue, SourceOrigin};
use crate::provenance::{ProvenanceMap, ProvenanceSpan};

/// Consume a node's stream and return the fully-converged text.
pub async fn converge_to_string(node: Arc<dyn Node>, ctx: Context) -> Option<String> {
    let mut stream = node.get_stream(ctx);
    let mut last: Option<String> = None;

    while let Some(batch) = stream.next().await {
        let mut s = String::new();
        for trace in &batch {
            let final_node = trace.final_result();
            if let Some(v) = final_node.as_string_value() {
                s.push_str(v);
            }
        }
        last = Some(s);
    }

    last
}

/// Consume a node's stream and return the converged output as [`FileContent`].
///
/// - If all leaf values are text → `FileContent::Text` (concatenated).
/// - If any leaf value is binary → `FileContent::Binary` (concatenated;
///   whitespace-only text traces are silently skipped as XML formatting artifacts).
/// - `NodeValue::None` values are always skipped.
pub async fn converge(node: Arc<dyn Node>, ctx: Context) -> Option<FileContent> {
    let mut stream = node.get_stream(ctx);
    let mut last: Option<FileContent> = None;

    while let Some(batch) = stream.next().await {
        let mut texts: Vec<String> = Vec::new();
        let mut binaries: Vec<Vec<u8>> = Vec::new();
        let mut has_binary = false;

        for trace in &batch {
            let final_node = trace.final_result();
            match final_node.node_value() {
                NodeValue::Text(s) => texts.push(s),
                NodeValue::Binary(data) => {
                    has_binary = true;
                    if let Ok(bytes) = data.to_bytes() {
                        binaries.push(bytes);
                    }
                }
                NodeValue::None => {}
            }
        }

        last = Some(if has_binary {
            // Mixed batch: concatenate all binary, skip whitespace-only text
            let mut combined = Vec::new();
            for t in &texts {
                if !t.trim().is_empty() {
                    combined.extend_from_slice(t.as_bytes());
                }
            }
            for b in binaries {
                combined.extend_from_slice(&b);
            }
            FileContent::Binary(BinaryData::Inline(combined))
        } else {
            // All text
            let mut s = String::new();
            for t in texts {
                s.push_str(&t);
            }
            FileContent::Text(s)
        });
    }

    last
}

/// Consume a node's stream and return converged text plus a [`ProvenanceMap`].
///
/// For each `NodeTrace` in the final batch, records the output byte range
/// and the node's `source_origin()`. If the final result has no origin,
/// walks the trace in reverse looking for the closest node with one.
pub async fn converge_with_provenance(
    node: Arc<dyn Node>,
    ctx: Context,
) -> Option<(String, ProvenanceMap)> {
    let mut stream = node.get_stream(ctx);
    let mut last_batch: Option<Vec<crate::node::NodeTrace>> = None;

    while let Some(batch) = stream.next().await {
        last_batch = Some(batch);
    }

    let batch = last_batch?;
    let mut output = String::new();
    let mut map = ProvenanceMap::new();

    for trace in &batch {
        let final_node = trace.final_result();
        if let Some(text) = final_node.as_string_value() {
            let start = output.len();
            output.push_str(text);
            let end = output.len();

            if start < end {
                // Try to get origin from the final result first
                let origin = final_node
                    .source_origin()
                    .cloned()
                    .or_else(|| {
                        // Walk trace in reverse for closest origin
                        trace
                            .trace()
                            .iter()
                            .rev()
                            .find_map(|n| n.source_origin().cloned())
                    })
                    .unwrap_or(SourceOrigin::Synthetic);

                map.push(ProvenanceSpan {
                    output_start: start,
                    output_end: end,
                    origin,
                });
            }
        }
    }

    Some((output, map))
}
