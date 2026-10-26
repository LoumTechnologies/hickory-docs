<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-exec/src/node.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ba14154add06"></a>
## `crates/hick-exec/src/node.rs`

<a id="f-ba14154add06-tests"></a>
### `tests`

Node trait and core node types — re-exported from `hick-flow`.

<storycode:copy id="f-ba14154add06-tests">
//! Node trait and core node types — re-exported from `hick-flow`.

pub use hick_flow::{
    BinaryData, BinaryNode, BoxStream, Context, FileContent, InsertionPoint, Node, NodeTrace,
    NodeValue, ProvenanceMap, ProvenanceSpan, ProvenanceTransformNode, SeparatorNode, SourceOrigin,
    SpanNode, StringNode, TransformNode, TransformSegment, TransformSegmentOrigin, converge,
    converge_to_string, converge_with_provenance,
};

// Keep Operation accessible within this crate (hick-flow re-exports it as pub(crate)).
// InsertionPoint::get_stream is the only consumer, and it's in hick-flow now.

// ---------------------------------------------------------------------------
// Tests (validate the re-export surface)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn string_node_emits_value() {
        let node: std::sync::Arc<dyn Node> = std::sync::Arc::new(StringNode::new("hello"));
        let result = converge_to_string(node, Context::default()).await;
        assert_eq!(result.unwrap(), "hello");
    }

    #[tokio::test]
    async fn insertion_point_concatenates_children() {
        let ip = std::sync::Arc::new(InsertionPoint::new());
        ip.add(std::sync::Arc::new(StringNode::new("a")));
        ip.add(std::sync::Arc::new(StringNode::new("b")));
        ip.add(std::sync::Arc::new(StringNode::new("c")));
        ip.close();

        let result = converge_to_string(ip, Context::default()).await;
        assert_eq!(result.unwrap(), "abc");
    }

    #[tokio::test]
    async fn insertion_point_with_separator() {
        let sep: std::sync::Arc<dyn Node> = std::sync::Arc::new(StringNode::new(", "));
        let ip = std::sync::Arc::new(InsertionPoint::with_separator(sep));
        ip.add(std::sync::Arc::new(StringNode::new("x")));
        ip.add(std::sync::Arc::new(StringNode::new("y")));
        ip.add(std::sync::Arc::new(StringNode::new("z")));
        ip.close();

        let result = converge_to_string(ip, Context::default()).await;
        assert_eq!(result.unwrap(), "x, y, z");
    }

    #[tokio::test]
    async fn nested_insertion_points() {
        let outer = std::sync::Arc::new(InsertionPoint::new());
        let inner = std::sync::Arc::new(InsertionPoint::new());
        inner.add(std::sync::Arc::new(StringNode::new("inner")));
        inner.close();
        outer.add(std::sync::Arc::new(StringNode::new("before-")));
        outer.add(inner);
        outer.add(std::sync::Arc::new(StringNode::new("-after")));
        outer.close();

        let result = converge_to_string(outer, Context::default()).await;
        assert_eq!(result.unwrap(), "before-inner-after");
    }
}
</storycode:copy>

### Assemble `crates/hick-exec/src/node.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-exec/src/node.rs"><storycode:paste select="#f-ba14154add06-tests"/></storycode:file>

</storycode:doc>
