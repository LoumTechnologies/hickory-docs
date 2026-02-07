//! Utility for consuming a node's stream and returning the fully-converged text.

use std::sync::Arc;

use futures::StreamExt;

use crate::context::Context;
use crate::node::Node;

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
