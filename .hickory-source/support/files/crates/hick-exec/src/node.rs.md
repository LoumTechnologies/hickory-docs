<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-exec/src/node.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `092f51b18a83e7f6cf9398f26a2e4174b9278543b15d1c6c09a06d91e3600efd`.

<book:entry path="crates/hick-exec/src/node.rs">
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
</book:entry>

</book:doc>
