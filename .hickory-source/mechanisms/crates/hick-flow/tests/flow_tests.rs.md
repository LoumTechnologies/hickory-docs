<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-flow/tests/flow_tests.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c959154fc401"></a>
## `crates/hick-flow/tests/flow_tests.rs`

<a id="f-c959154fc401-node_trace_new_has_empty_trace"></a>
### `node_trace_new_has_empty_trace`

<storycode:copy id="f-c959154fc401-node_trace_new_has_empty_trace">
//! Comprehensive tests for hick-flow.

use std::sync::Arc;

use futures::StreamExt;
use hick_flow::{
    Context, DynamicCombineLatest, InsertionPoint, Node, NodeTrace, SeparatorNode, StringNode,
    TransformNode, converge_to_string,
};

// ===========================================================================
// NodeTrace tests
// ===========================================================================

#[test]
fn node_trace_new_has_empty_trace() {
    let node: Arc<dyn Node> = Arc::new(StringNode::new("x"));
    let trace = NodeTrace::new(node.clone());
    assert!(trace.trace().is_empty());
    assert!(trace.final_result().as_string_value() == Some("x"));
}

</storycode:copy>
<a id="f-c959154fc401-node_trace_add_to_trace_preserves_final_result"></a>
### `node_trace_add_to_trace_preserves_final_result`

<storycode:copy id="f-c959154fc401-node_trace_add_to_trace_preserves_final_result">
#[test]
fn node_trace_add_to_trace_preserves_final_result() {
    let leaf: Arc<dyn Node> = Arc::new(StringNode::new("leaf"));
    let wrapper: Arc<dyn Node> = Arc::new(StringNode::new("wrapper"));
    let trace = NodeTrace::new(leaf);
    let trace2 = trace.add_to_trace(wrapper);
    assert_eq!(trace2.final_result().as_string_value(), Some("leaf"));
    assert_eq!(trace2.trace().len(), 1);
}

</storycode:copy>
<a id="f-c959154fc401-node_trace_transform_changes_final_result"></a>
### `node_trace_transform_changes_final_result`

<storycode:copy id="f-c959154fc401-node_trace_transform_changes_final_result">
#[test]
fn node_trace_transform_changes_final_result() {
    let original: Arc<dyn Node> = Arc::new(StringNode::new("original"));
    let transformed: Arc<dyn Node> = Arc::new(StringNode::new("TRANSFORMED"));
    let transformer: Arc<dyn Node> = Arc::new(StringNode::new("xform"));
    let trace = NodeTrace::new(original);
    let trace2 = trace.transform(transformed, transformer);
    assert_eq!(trace2.final_result().as_string_value(), Some("TRANSFORMED"));
    assert_eq!(trace2.trace().len(), 1);
}

</storycode:copy>
<a id="f-c959154fc401-node_trace_accumulates_multiple_entries"></a>
### `node_trace_accumulates_multiple_entries`

<storycode:copy id="f-c959154fc401-node_trace_accumulates_multiple_entries">
#[test]
fn node_trace_accumulates_multiple_entries() {
    let leaf: Arc<dyn Node> = Arc::new(StringNode::new("leaf"));
    let n1: Arc<dyn Node> = Arc::new(StringNode::new("n1"));
    let n2: Arc<dyn Node> = Arc::new(StringNode::new("n2"));
    let n3: Arc<dyn Node> = Arc::new(StringNode::new("n3"));
    let trace = NodeTrace::new(leaf)
        .add_to_trace(n1)
        .add_to_trace(n2)
        .add_to_trace(n3);
    assert_eq!(trace.trace().len(), 3);
    assert_eq!(trace.final_result().as_string_value(), Some("leaf"));
}

</storycode:copy>
<a id="f-c959154fc401-node_trace_clone_is_independent"></a>
### `node_trace_clone_is_independent`

<storycode:copy id="f-c959154fc401-node_trace_clone_is_independent">
#[test]
fn node_trace_clone_is_independent() {
    let leaf: Arc<dyn Node> = Arc::new(StringNode::new("leaf"));
    let n1: Arc<dyn Node> = Arc::new(StringNode::new("n1"));
    let trace = NodeTrace::new(leaf);
    let trace2 = trace.add_to_trace(n1);
    // Original still has empty trace
    assert!(trace.trace().is_empty());
    assert_eq!(trace2.trace().len(), 1);
}

</storycode:copy>
<a id="f-c959154fc401-node_trace_transform_then_add_to_trace"></a>
### `node_trace_transform_then_add_to_trace`

<storycode:copy id="f-c959154fc401-node_trace_transform_then_add_to_trace">
#[test]
fn node_trace_transform_then_add_to_trace() {
    let leaf: Arc<dyn Node> = Arc::new(StringNode::new("leaf"));
    let transformed: Arc<dyn Node> = Arc::new(StringNode::new("LEAF"));
    let xform: Arc<dyn Node> = Arc::new(StringNode::new("upper"));
    let wrapper: Arc<dyn Node> = Arc::new(StringNode::new("wrapper"));

    let trace = NodeTrace::new(leaf)
        .transform(transformed, xform)
        .add_to_trace(wrapper);
    assert_eq!(trace.final_result().as_string_value(), Some("LEAF"));
    assert_eq!(trace.trace().len(), 2);
}

</storycode:copy>
<a id="f-c959154fc401-node_trace_new_final_result_is_correct_arc"></a>
### `node_trace_new_final_result_is_correct_arc`

<storycode:copy id="f-c959154fc401-node_trace_new_final_result_is_correct_arc">
#[test]
fn node_trace_new_final_result_is_correct_arc() {
    let node = Arc::new(StringNode::new("hello"));
    let trace = NodeTrace::new(node.clone() as Arc<dyn Node>);
    // Verify we get the same value back
    assert_eq!(trace.final_result().as_string_value(), Some("hello"));
}

</storycode:copy>
<a id="f-c959154fc401-string_node_emits_value"></a>
### `string_node_emits_value`

<storycode:copy id="f-c959154fc401-string_node_emits_value">
// ===========================================================================
// StringNode tests
// ===========================================================================

#[tokio::test]
async fn string_node_emits_value() {
    let node: Arc<dyn Node> = Arc::new(StringNode::new("hello"));
    let result = converge_to_string(node, Context::default()).await;
    assert_eq!(result.unwrap(), "hello");
}

</storycode:copy>
<a id="f-c959154fc401-string_node_as_string_value"></a>
### `string_node_as_string_value`

<storycode:copy id="f-c959154fc401-string_node_as_string_value">
#[tokio::test]
async fn string_node_as_string_value() {
    let node = StringNode::new("test");
    assert_eq!(node.as_string_value(), Some("test"));
}

</storycode:copy>
<a id="f-c959154fc401-string_node_stream_completes"></a>
### `string_node_stream_completes`

<storycode:copy id="f-c959154fc401-string_node_stream_completes">
#[tokio::test]
async fn string_node_stream_completes() {
    let node: Arc<dyn Node> = Arc::new(StringNode::new("done"));
    let mut stream = node.get_stream(Context::default());
    let first = stream.next().await;
    assert!(first.is_some());
    let second = stream.next().await;
    assert!(second.is_none());
}

</storycode:copy>
<a id="f-c959154fc401-string_node_empty_string"></a>
### `string_node_empty_string`

<storycode:copy id="f-c959154fc401-string_node_empty_string">
#[tokio::test]
async fn string_node_empty_string() {
    let node: Arc<dyn Node> = Arc::new(StringNode::new(""));
    let result = converge_to_string(node, Context::default()).await;
    assert_eq!(result.unwrap(), "");
}

</storycode:copy>
<a id="f-c959154fc401-string_node_is_not_separator"></a>
### `string_node_is_not_separator`

<storycode:copy id="f-c959154fc401-string_node_is_not_separator">
#[test]
fn string_node_is_not_separator() {
    let node = StringNode::new("x");
    assert!(!node.is_separator());
}

</storycode:copy>
<a id="f-c959154fc401-string_node_with_stable_id"></a>
### `string_node_with_stable_id`

<storycode:copy id="f-c959154fc401-string_node_with_stable_id">
#[test]
fn string_node_with_stable_id() {
    let id = vec![1, 2, 3, 4];
    let node = StringNode::new_with_id("test", Some(id.clone()));
    assert_eq!(node.id_bytes(), id);
    assert_eq!(node.value(), "test");
}

</storycode:copy>
<a id="f-c959154fc401-separator_node_is_separator"></a>
### `separator_node_is_separator`

<storycode:copy id="f-c959154fc401-separator_node_is_separator">
// ===========================================================================
// SeparatorNode tests
// ===========================================================================

#[test]
fn separator_node_is_separator() {
    let inner: Arc<dyn Node> = Arc::new(StringNode::new(","));
    let sep = SeparatorNode::new(inner);
    assert!(sep.is_separator());
}

</storycode:copy>
<a id="f-c959154fc401-separator_node_wraps_value"></a>
### `separator_node_wraps_value`

<storycode:copy id="f-c959154fc401-separator_node_wraps_value">
#[tokio::test]
async fn separator_node_wraps_value() {
    let inner: Arc<dyn Node> = Arc::new(StringNode::new(","));
    let sep: Arc<dyn Node> = Arc::new(SeparatorNode::new(inner));
    let result = converge_to_string(sep, Context::default()).await;
    assert_eq!(result.unwrap(), ",");
}

</storycode:copy>
<a id="f-c959154fc401-separator_node_adds_self_to_trace"></a>
### `separator_node_adds_self_to_trace`

<storycode:copy id="f-c959154fc401-separator_node_adds_self_to_trace">
#[tokio::test]
async fn separator_node_adds_self_to_trace() {
    let inner: Arc<dyn Node> = Arc::new(StringNode::new(","));
    let sep: Arc<dyn Node> = Arc::new(SeparatorNode::new(inner));
    let mut stream = sep.get_stream(Context::default());
    let batch = stream.next().await.unwrap();
    assert_eq!(batch.len(), 1);
    // The trace should contain the separator node
    assert!(batch[0].trace().iter().any(|n| n.is_separator()));
}

</storycode:copy>
<a id="f-c959154fc401-separator_node_as_string_value_is_none"></a>
### `separator_node_as_string_value_is_none`

<storycode:copy id="f-c959154fc401-separator_node_as_string_value_is_none">
#[test]
fn separator_node_as_string_value_is_none() {
    let inner: Arc<dyn Node> = Arc::new(StringNode::new(","));
    let sep = SeparatorNode::new(inner);
    assert!(sep.as_string_value().is_none());
}

</storycode:copy>
<a id="f-c959154fc401-insertion_point_empty_produces_output"></a>
### `insertion_point_empty_produces_output`

<storycode:copy id="f-c959154fc401-insertion_point_empty_produces_output">
// ===========================================================================
// InsertionPoint tests
// ===========================================================================

#[tokio::test]
async fn insertion_point_empty_produces_output() {
    let ip = Arc::new(InsertionPoint::new());
    ip.close();
    let result = converge_to_string(ip, Context::default()).await;
    // Empty insertion point should produce empty string
    assert_eq!(result.unwrap(), "");
}

</storycode:copy>
<a id="f-c959154fc401-insertion_point_single_child"></a>
### `insertion_point_single_child`

<storycode:copy id="f-c959154fc401-insertion_point_single_child">
#[tokio::test]
async fn insertion_point_single_child() {
    let ip = Arc::new(InsertionPoint::new());
    ip.add(Arc::new(StringNode::new("only")));
    ip.close();
    let result = converge_to_string(ip, Context::default()).await;
    assert_eq!(result.unwrap(), "only");
}

</storycode:copy>
<a id="f-c959154fc401-insertion_point_concatenates_children"></a>
### `insertion_point_concatenates_children`

<storycode:copy id="f-c959154fc401-insertion_point_concatenates_children">
#[tokio::test]
async fn insertion_point_concatenates_children() {
    let ip = Arc::new(InsertionPoint::new());
    ip.add(Arc::new(StringNode::new("a")));
    ip.add(Arc::new(StringNode::new("b")));
    ip.add(Arc::new(StringNode::new("c")));
    ip.close();
    let result = converge_to_string(ip, Context::default()).await;
    assert_eq!(result.unwrap(), "abc");
}

</storycode:copy>
<a id="f-c959154fc401-insertion_point_with_separator"></a>
### `insertion_point_with_separator`

<storycode:copy id="f-c959154fc401-insertion_point_with_separator">
#[tokio::test]
async fn insertion_point_with_separator() {
    let sep: Arc<dyn Node> = Arc::new(StringNode::new(", "));
    let ip = Arc::new(InsertionPoint::with_separator(sep));
    ip.add(Arc::new(StringNode::new("x")));
    ip.add(Arc::new(StringNode::new("y")));
    ip.add(Arc::new(StringNode::new("z")));
    ip.close();
    let result = converge_to_string(ip, Context::default()).await;
    assert_eq!(result.unwrap(), "x, y, z");
}

</storycode:copy>
<a id="f-c959154fc401-insertion_point_trim_leading_separator"></a>
### `insertion_point_trim_leading_separator`

<storycode:copy id="f-c959154fc401-insertion_point_trim_leading_separator">
#[tokio::test]
async fn insertion_point_trim_leading_separator() {
    let sep: Arc<dyn Node> = Arc::new(StringNode::new("-"));
    let ip = Arc::new(InsertionPoint::with_separator(sep));
    ip.add(Arc::new(StringNode::new("a")));
    ip.close();
    let result = converge_to_string(ip, Context::default()).await;
    // Leading separator should be trimmed even with single child
    assert_eq!(result.unwrap(), "a");
}

</storycode:copy>
<a id="f-c959154fc401-insertion_point_nested"></a>
### `insertion_point_nested`

<storycode:copy id="f-c959154fc401-insertion_point_nested">
#[tokio::test]
async fn insertion_point_nested() {
    let outer = Arc::new(InsertionPoint::new());
    let inner = Arc::new(InsertionPoint::new());
    inner.add(Arc::new(StringNode::new("inner")));
    inner.close();
    outer.add(Arc::new(StringNode::new("before-")));
    outer.add(inner);
    outer.add(Arc::new(StringNode::new("-after")));
    outer.close();
    let result = converge_to_string(outer, Context::default()).await;
    assert_eq!(result.unwrap(), "before-inner-after");
}

</storycode:copy>
<a id="f-c959154fc401-insertion_point_dynamic_add"></a>
### `insertion_point_dynamic_add`

<storycode:copy id="f-c959154fc401-insertion_point_dynamic_add">
#[tokio::test]
async fn insertion_point_dynamic_add() {
    let ip = Arc::new(InsertionPoint::new_with_never_complete(true));

    let ip_clone = ip.clone();
    let ip_node: Arc<dyn Node> = ip.clone();
    let mut stream = ip_node.get_stream(Context::default());

    // Add first child dynamically (after stream started)
    ip_clone.add(Arc::new(StringNode::new("first")));

    // Get first emission
    let first = stream.next().await.unwrap();
    let s: String = first
        .iter()
        .filter_map(|t| t.final_result().as_string_value().map(|s| s.to_string()))
        .collect();
    assert_eq!(s, "first");

    // Add another child dynamically
    ip_clone.add(Arc::new(StringNode::new("second")));

    let second = stream.next().await.unwrap();
    let s2: String = second
        .iter()
        .filter_map(|t| t.final_result().as_string_value().map(|s| s.to_string()))
        .collect();
    assert_eq!(s2, "firstsecond");

    ip_clone.close();
}

</storycode:copy>
<a id="f-c959154fc401-insertion_point_dynamic_remove"></a>
### `insertion_point_dynamic_remove`

<storycode:copy id="f-c959154fc401-insertion_point_dynamic_remove">
#[tokio::test]
async fn insertion_point_dynamic_remove() {
    let ip = Arc::new(InsertionPoint::new_with_never_complete(true));

    let ip_clone = ip.clone();
    let ip_node: Arc<dyn Node> = ip.clone();
    let mut stream = ip_node.get_stream(Context::default());

    let child1: Arc<dyn Node> = Arc::new(StringNode::new("a"));
    let child2: Arc<dyn Node> = Arc::new(StringNode::new("b"));

    // Add both children dynamically
    ip_clone.add(child1.clone());
    ip_clone.add(child2.clone());

    // Get first emission with both children
    let first = stream.next().await.unwrap();
    let s: String = first
        .iter()
        .filter_map(|t| t.final_result().as_string_value().map(|s| s.to_string()))
        .collect();
    assert_eq!(s, "ab");

    // Remove first child
    ip_clone.remove(child1);

    let second = stream.next().await.unwrap();
    let s2: String = second
        .iter()
        .filter_map(|t| t.final_result().as_string_value().map(|s| s.to_string()))
        .collect();
    assert_eq!(s2, "b");

    ip_clone.close();
}

</storycode:copy>
<a id="f-c959154fc401-insertion_point_close_is_idempotent"></a>
### `insertion_point_close_is_idempotent`

<storycode:copy id="f-c959154fc401-insertion_point_close_is_idempotent">
#[tokio::test]
async fn insertion_point_close_is_idempotent() {
    let ip = Arc::new(InsertionPoint::new());
    ip.add(Arc::new(StringNode::new("x")));
    ip.close();
    ip.close(); // second close should be safe
    let result = converge_to_string(ip, Context::default()).await;
    assert_eq!(result.unwrap(), "x");
}

</storycode:copy>
<a id="f-c959154fc401-insertion_point_deeply_nested"></a>
### `insertion_point_deeply_nested`

<storycode:copy id="f-c959154fc401-insertion_point_deeply_nested">
#[tokio::test]
async fn insertion_point_deeply_nested() {
    let level3 = Arc::new(InsertionPoint::new());
    level3.add(Arc::new(StringNode::new("deep")));
    level3.close();

    let level2 = Arc::new(InsertionPoint::new());
    level2.add(level3);
    level2.close();

    let level1 = Arc::new(InsertionPoint::new());
    level1.add(level2);
    level1.close();

    let result = converge_to_string(level1, Context::default()).await;
    assert_eq!(result.unwrap(), "deep");
}

</storycode:copy>
<a id="f-c959154fc401-insertion_point_never_complete_stays_open"></a>
### `insertion_point_never_complete_stays_open`

<storycode:copy id="f-c959154fc401-insertion_point_never_complete_stays_open">
#[tokio::test]
async fn insertion_point_never_complete_stays_open() {
    let ip = Arc::new(InsertionPoint::new_with_never_complete(true));
    ip.add(Arc::new(StringNode::new("initial")));

    let ip_node: Arc<dyn Node> = ip.clone();
    let mut stream = ip_node.get_stream(Context::default());

    let first = stream.next().await.unwrap();
    let s: String = first
        .iter()
        .filter_map(|t| t.final_result().as_string_value().map(|s| s.to_string()))
        .collect();
    assert_eq!(s, "initial");

    // Stream should still be alive - add more data
    ip.add(Arc::new(StringNode::new("-more")));

    let second = stream.next().await.unwrap();
    let s2: String = second
        .iter()
        .filter_map(|t| t.final_result().as_string_value().map(|s| s.to_string()))
        .collect();
    assert_eq!(s2, "initial-more");

    // Close to let stream complete
    ip.close();
}

</storycode:copy>
<a id="f-c959154fc401-insertion_point_adds_self_to_trace"></a>
### `insertion_point_adds_self_to_trace`

<storycode:copy id="f-c959154fc401-insertion_point_adds_self_to_trace">
#[tokio::test]
async fn insertion_point_adds_self_to_trace() {
    let ip = Arc::new(InsertionPoint::new());
    ip.add(Arc::new(StringNode::new("child")));
    ip.close();

    let ip_node: Arc<dyn Node> = ip.clone();
    let mut stream = ip_node.get_stream(Context::default());
    let batch = stream.next().await.unwrap();
    // The trace should contain at least the insertion point
    assert!(!batch[0].trace().is_empty());
}

</storycode:copy>
<a id="f-c959154fc401-dcl_single_source_emits"></a>
### `dcl_single_source_emits`

<storycode:copy id="f-c959154fc401-dcl_single_source_emits">
// ===========================================================================
// DynamicCombineLatest tests
// ===========================================================================

#[tokio::test]
async fn dcl_single_source_emits() {
    let dcl = DynamicCombineLatest::new(|v: Vec<i32>| v.iter().sum::<i32>());
    dcl.add_source(futures::stream::once(async { 42 }));
    let mut stream = dcl.stream();
    let val = stream.next().await.unwrap();
    assert_eq!(val, 42);
}

</storycode:copy>
<a id="f-c959154fc401-dcl_two_sources_combine"></a>
### `dcl_two_sources_combine`

<storycode:copy id="f-c959154fc401-dcl_two_sources_combine">
#[tokio::test]
async fn dcl_two_sources_combine() {
    let dcl = DynamicCombineLatest::new(|v: Vec<i32>| v.iter().sum::<i32>());
    dcl.add_source(futures::stream::once(async { 10 }));
    dcl.add_source(futures::stream::once(async { 20 }));
    let mut stream = dcl.stream();
    let val = stream.next().await.unwrap();
    assert_eq!(val, 30);
}

</storycode:copy>
<a id="f-c959154fc401-dcl_completes_on_quiescence"></a>
### `dcl_completes_on_quiescence`

<storycode:copy id="f-c959154fc401-dcl_completes_on_quiescence">
#[tokio::test]
async fn dcl_completes_on_quiescence() {
    let dcl = DynamicCombineLatest::new(|v: Vec<i32>| v.iter().sum::<i32>());
    dcl.add_source(futures::stream::once(async { 1 }));
    let results: Vec<_> = dcl.stream().collect().await;
    assert!(!results.is_empty());
}

</storycode:copy>
<a id="f-c959154fc401-dcl_remove_source"></a>
### `dcl_remove_source`

<storycode:copy id="f-c959154fc401-dcl_remove_source">
#[tokio::test]
async fn dcl_remove_source() {
    let dcl = DynamicCombineLatest::new(|v: Vec<String>| v.join(","));
    let id1 = dcl.add_source(futures::stream::once(async { "a".to_string() }));
    dcl.add_source(futures::stream::once(async { "b".to_string() }));
    let mut stream = dcl.stream();
    let val = stream.next().await.unwrap();
    assert!(val.contains("a") && val.contains("b"));
    dcl.remove_source(id1);
}

</storycode:copy>
<a id="f-c959154fc401-dcl_ordering_preserved"></a>
### `dcl_ordering_preserved`

<storycode:copy id="f-c959154fc401-dcl_ordering_preserved">
#[tokio::test]
async fn dcl_ordering_preserved() {
    let dcl = DynamicCombineLatest::new(|v: Vec<String>| v.join(""));
    dcl.add_source(futures::stream::once(async { "first".to_string() }));
    dcl.add_source(futures::stream::once(async { "second".to_string() }));
    let mut stream = dcl.stream();
    let val = stream.next().await.unwrap();
    // BTreeMap ordering ensures sources appear in insertion order
    assert_eq!(val, "firstsecond");
}

</storycode:copy>
<a id="f-c959154fc401-dcl_shutdown_ends_stream"></a>
### `dcl_shutdown_ends_stream`

<storycode:copy id="f-c959154fc401-dcl_shutdown_ends_stream">
#[tokio::test]
async fn dcl_shutdown_ends_stream() {
    let dcl = DynamicCombineLatest::new_with_config(|v: Vec<i32>| v.iter().sum::<i32>(), false);
    dcl.add_source(futures::stream::once(async { 1 }));
    let mut stream = dcl.stream();
    let _ = stream.next().await;
    dcl.shutdown();
    // After shutdown, stream should complete
    let result = stream.next().await;
    assert!(result.is_none());
}

</storycode:copy>
<a id="f-c959154fc401-dcl_multi_value_stream"></a>
### `dcl_multi_value_stream`

<storycode:copy id="f-c959154fc401-dcl_multi_value_stream">
#[tokio::test]
async fn dcl_multi_value_stream() {
    let dcl = DynamicCombineLatest::new(|v: Vec<i32>| v.iter().sum::<i32>());
    dcl.add_source(futures::stream::iter(vec![1, 2, 3]));
    let results: Vec<_> = dcl.stream().collect().await;
    // Should get at least 1 emission, possibly 3 if each value triggers
    assert!(!results.is_empty());
    // Last emission should use latest value
    assert_eq!(*results.last().unwrap(), 3);
}

</storycode:copy>
<a id="f-c959154fc401-dcl_identity_combiner"></a>
### `dcl_identity_combiner`

<storycode:copy id="f-c959154fc401-dcl_identity_combiner">
#[tokio::test]
async fn dcl_identity_combiner() {
    let dcl = DynamicCombineLatest::new(|v: Vec<String>| v);
    dcl.add_source(futures::stream::once(async { "only".to_string() }));
    let mut stream = dcl.stream();
    let val = stream.next().await.unwrap();
    assert_eq!(val, vec!["only".to_string()]);
}

</storycode:copy>
<a id="f-c959154fc401-dcl_add_source_after_stream_started"></a>
### `dcl_add_source_after_stream_started`

<storycode:copy id="f-c959154fc401-dcl_add_source_after_stream_started">
#[tokio::test]
async fn dcl_add_source_after_stream_started() {
    let dcl = DynamicCombineLatest::new_with_config(|v: Vec<i32>| v.iter().sum::<i32>(), false);
    dcl.add_source(futures::stream::once(async { 10 }));
    let mut stream = dcl.stream();
    let first = stream.next().await.unwrap();
    assert_eq!(first, 10);

    dcl.add_source(futures::stream::once(async { 5 }));
    let second = stream.next().await.unwrap();
    assert_eq!(second, 15);

    dcl.shutdown();
}

</storycode:copy>
<a id="f-c959154fc401-dcl_three_sources"></a>
### `dcl_three_sources`

<storycode:copy id="f-c959154fc401-dcl_three_sources">
#[tokio::test]
async fn dcl_three_sources() {
    let dcl = DynamicCombineLatest::new(|v: Vec<i32>| v.iter().sum::<i32>());
    dcl.add_source(futures::stream::once(async { 1 }));
    dcl.add_source(futures::stream::once(async { 2 }));
    dcl.add_source(futures::stream::once(async { 3 }));
    let mut stream = dcl.stream();
    let val = stream.next().await.unwrap();
    assert_eq!(val, 6);
}

</storycode:copy>
<a id="f-c959154fc401-context_default_has_no_extensions"></a>
### `context_default_has_no_extensions`

<storycode:copy id="f-c959154fc401-context_default_has_no_extensions">
// ===========================================================================
// Context tests
// ===========================================================================

#[test]
fn context_default_has_no_extensions() {
    let ctx = Context::default();
    assert!(ctx.get_extension::<String>().is_none());
}

</storycode:copy>
<a id="f-c959154fc401-context_extension_roundtrip"></a>
### `context_extension_roundtrip`

<storycode:copy id="f-c959154fc401-context_extension_roundtrip">
#[test]
fn context_extension_roundtrip() {
    let ctx = Context::new().with_extension(42u32);
    let val = ctx.get_extension::<u32>().unwrap();
    assert_eq!(*val, 42);
}

</storycode:copy>
<a id="f-c959154fc401-context_multiple_extension_types"></a>
### `context_multiple_extension_types`

<storycode:copy id="f-c959154fc401-context_multiple_extension_types">
#[test]
fn context_multiple_extension_types() {
    let ctx = Context::new()
        .with_extension(42u32)
        .with_extension("hello".to_string());
    assert_eq!(*ctx.get_extension::<u32>().unwrap(), 42);
    assert_eq!(*ctx.get_extension::<String>().unwrap(), "hello");
}

</storycode:copy>
<a id="f-c959154fc401-context_extension_overwrite"></a>
### `context_extension_overwrite`

<storycode:copy id="f-c959154fc401-context_extension_overwrite">
#[test]
fn context_extension_overwrite() {
    let ctx = Context::new().with_extension(1u32).with_extension(2u32);
    assert_eq!(*ctx.get_extension::<u32>().unwrap(), 2);
}

</storycode:copy>
<a id="f-c959154fc401-context_clone_preserves_extensions"></a>
### `context_clone_preserves_extensions`

<storycode:copy id="f-c959154fc401-context_clone_preserves_extensions">
#[test]
fn context_clone_preserves_extensions() {
    let ctx = Context::new().with_extension(99u64);
    let ctx2 = ctx.clone();
    assert_eq!(*ctx2.get_extension::<u64>().unwrap(), 99);
}

</storycode:copy>
<a id="f-c959154fc401-context_verbose_flag"></a>
### `context_verbose_flag`

<storycode:copy id="f-c959154fc401-context_verbose_flag">
#[test]
fn context_verbose_flag() {
    let mut ctx = Context::new();
    assert!(!ctx.verbose);
    ctx.verbose = true;
    assert!(ctx.verbose);
}

</storycode:copy>
<a id="f-c959154fc401-transform_uppercase"></a>
### `transform_uppercase`

<storycode:copy id="f-c959154fc401-transform_uppercase">
// ===========================================================================
// TransformNode tests
// ===========================================================================

#[tokio::test]
async fn transform_uppercase() {
    let children = Arc::new(InsertionPoint::new());
    children.add(Arc::new(StringNode::new("hello")));
    children.close();

    let xform: Arc<dyn Node> = Arc::new(TransformNode::new(
        children,
        |s| s.to_uppercase(),
        "uppercase",
    ));
    let result = converge_to_string(xform, Context::default()).await;
    assert_eq!(result.unwrap(), "HELLO");
}

</storycode:copy>
<a id="f-c959154fc401-transform_replace"></a>
### `transform_replace`

<storycode:copy id="f-c959154fc401-transform_replace">
#[tokio::test]
async fn transform_replace() {
    let children = Arc::new(InsertionPoint::new());
    children.add(Arc::new(StringNode::new("foo bar foo")));
    children.close();

    let xform: Arc<dyn Node> = Arc::new(TransformNode::new(
        children,
        |s| s.replace("foo", "baz"),
        "replace",
    ));
    let result = converge_to_string(xform, Context::default()).await;
    assert_eq!(result.unwrap(), "baz bar baz");
}

</storycode:copy>
<a id="f-c959154fc401-transform_records_in_trace"></a>
### `transform_records_in_trace`

<storycode:copy id="f-c959154fc401-transform_records_in_trace">
#[tokio::test]
async fn transform_records_in_trace() {
    let children = Arc::new(InsertionPoint::new());
    children.add(Arc::new(StringNode::new("test")));
    children.close();

    let xform: Arc<dyn Node> =
        Arc::new(TransformNode::new(children, |s| s.to_uppercase(), "upper"));
    let mut stream = xform.get_stream(Context::default());
    let batch = stream.next().await.unwrap();
    assert!(!batch.is_empty());
    // The trace should contain the transform node
    assert!(!batch[0].trace().is_empty());
    assert_eq!(batch[0].final_result().as_string_value(), Some("TEST"));
}

</storycode:copy>
<a id="f-c959154fc401-transform_nested"></a>
### `transform_nested`

<storycode:copy id="f-c959154fc401-transform_nested">
#[tokio::test]
async fn transform_nested() {
    let children = Arc::new(InsertionPoint::new());
    children.add(Arc::new(StringNode::new("hello world")));
    children.close();

    let first: Arc<InsertionPoint> = Arc::new(InsertionPoint::new());
    first.add(Arc::new(TransformNode::new(
        children,
        |s| s.to_uppercase(),
        "upper",
    )));
    first.close();

    let xform: Arc<dyn Node> = Arc::new(TransformNode::new(
        first,
        |s| s.replace(' ', "_"),
        "underscore",
    ));
    let result = converge_to_string(xform, Context::default()).await;
    assert_eq!(result.unwrap(), "HELLO_WORLD");
}

</storycode:copy>
<a id="f-c959154fc401-transform_identity"></a>
### `transform_identity`

<storycode:copy id="f-c959154fc401-transform_identity">
#[tokio::test]
async fn transform_identity() {
    let children = Arc::new(InsertionPoint::new());
    children.add(Arc::new(StringNode::new("unchanged")));
    children.close();

    let xform: Arc<dyn Node> =
        Arc::new(TransformNode::new(children, |s| s.to_string(), "identity"));
    let result = converge_to_string(xform, Context::default()).await;
    assert_eq!(result.unwrap(), "unchanged");
}

</storycode:copy>
<a id="f-c959154fc401-transform_empty_children"></a>
### `transform_empty_children`

<storycode:copy id="f-c959154fc401-transform_empty_children">
#[tokio::test]
async fn transform_empty_children() {
    let children = Arc::new(InsertionPoint::new());
    children.close();

    let xform: Arc<dyn Node> =
        Arc::new(TransformNode::new(children, |s| s.to_uppercase(), "upper"));
    let result = converge_to_string(xform, Context::default()).await;
    assert_eq!(result.unwrap(), "");
}

</storycode:copy>
<a id="f-c959154fc401-transform_with_separator"></a>
### `transform_with_separator`

<storycode:copy id="f-c959154fc401-transform_with_separator">
#[tokio::test]
async fn transform_with_separator() {
    let sep: Arc<dyn Node> = Arc::new(StringNode::new(", "));
    let children = Arc::new(InsertionPoint::with_separator(sep));
    children.add(Arc::new(StringNode::new("a")));
    children.add(Arc::new(StringNode::new("b")));
    children.close();

    let xform: Arc<dyn Node> =
        Arc::new(TransformNode::new(children, |s| s.to_uppercase(), "upper"));
    let result = converge_to_string(xform, Context::default()).await;
    assert_eq!(result.unwrap(), "A, B");
}

</storycode:copy>
<a id="f-c959154fc401-transform_multiple_children"></a>
### `transform_multiple_children`

<storycode:copy id="f-c959154fc401-transform_multiple_children">
#[tokio::test]
async fn transform_multiple_children() {
    let children = Arc::new(InsertionPoint::new());
    children.add(Arc::new(StringNode::new("hello")));
    children.add(Arc::new(StringNode::new(" ")));
    children.add(Arc::new(StringNode::new("world")));
    children.close();

    let xform: Arc<dyn Node> =
        Arc::new(TransformNode::new(children, |s| s.to_uppercase(), "upper"));
    let result = converge_to_string(xform, Context::default()).await;
    assert_eq!(result.unwrap(), "HELLO WORLD");
}

</storycode:copy>
<a id="f-c959154fc401-transform_label"></a>
### `transform_label`

<storycode:copy id="f-c959154fc401-transform_label">
#[tokio::test]
async fn transform_label() {
    let children = Arc::new(InsertionPoint::new());
    children.close();
    let xform = TransformNode::new(children, |s| s.to_string(), "my-transform");
    assert_eq!(xform.label(), "my-transform");
}

</storycode:copy>
<a id="f-c959154fc401-converge_single_string"></a>
### `converge_single_string`

<storycode:copy id="f-c959154fc401-converge_single_string">
// ===========================================================================
// converge_to_string tests
// ===========================================================================

#[tokio::test]
async fn converge_single_string() {
    let node: Arc<dyn Node> = Arc::new(StringNode::new("single"));
    let result = converge_to_string(node, Context::default()).await;
    assert_eq!(result.unwrap(), "single");
}

</storycode:copy>
<a id="f-c959154fc401-converge_concatenation"></a>
### `converge_concatenation`

<storycode:copy id="f-c959154fc401-converge_concatenation">
#[tokio::test]
async fn converge_concatenation() {
    let ip = Arc::new(InsertionPoint::new());
    ip.add(Arc::new(StringNode::new("hello")));
    ip.add(Arc::new(StringNode::new(" ")));
    ip.add(Arc::new(StringNode::new("world")));
    ip.close();
    let result = converge_to_string(ip, Context::default()).await;
    assert_eq!(result.unwrap(), "hello world");
}

</storycode:copy>
<a id="f-c959154fc401-converge_with_separator"></a>
### `converge_with_separator`

<storycode:copy id="f-c959154fc401-converge_with_separator">
#[tokio::test]
async fn converge_with_separator() {
    let sep: Arc<dyn Node> = Arc::new(StringNode::new(" | "));
    let ip = Arc::new(InsertionPoint::with_separator(sep));
    ip.add(Arc::new(StringNode::new("a")));
    ip.add(Arc::new(StringNode::new("b")));
    ip.add(Arc::new(StringNode::new("c")));
    ip.close();
    let result = converge_to_string(ip, Context::default()).await;
    assert_eq!(result.unwrap(), "a | b | c");
}

</storycode:copy>
<a id="f-c959154fc401-converge_nested"></a>
### `converge_nested`

<storycode:copy id="f-c959154fc401-converge_nested">
#[tokio::test]
async fn converge_nested() {
    let inner = Arc::new(InsertionPoint::new());
    inner.add(Arc::new(StringNode::new("deep")));
    inner.close();

    let outer = Arc::new(InsertionPoint::new());
    outer.add(Arc::new(StringNode::new("[")));
    outer.add(inner);
    outer.add(Arc::new(StringNode::new("]")));
    outer.close();

    let result = converge_to_string(outer, Context::default()).await;
    assert_eq!(result.unwrap(), "[deep]");
}

</storycode:copy>
<a id="f-c959154fc401-converge_with_transform"></a>
### `converge_with_transform`

<storycode:copy id="f-c959154fc401-converge_with_transform">
#[tokio::test]
async fn converge_with_transform() {
    let children = Arc::new(InsertionPoint::new());
    children.add(Arc::new(StringNode::new("hello")));
    children.close();

    let xform: Arc<dyn Node> =
        Arc::new(TransformNode::new(children, |s| format!("<<{s}>>"), "wrap"));
    let result = converge_to_string(xform, Context::default()).await;
    assert_eq!(result.unwrap(), "<<hello>>");
}

</storycode:copy>
<a id="f-c959154fc401-converge_empty"></a>
### `converge_empty`

<storycode:copy id="f-c959154fc401-converge_empty">
#[tokio::test]
async fn converge_empty() {
    let ip = Arc::new(InsertionPoint::new());
    ip.close();
    let result = converge_to_string(ip, Context::default()).await;
    assert_eq!(result.unwrap(), "");
}

</storycode:copy>
<a id="f-c959154fc401-converge_separator_single_child"></a>
### `converge_separator_single_child`

<storycode:copy id="f-c959154fc401-converge_separator_single_child">
#[tokio::test]
async fn converge_separator_single_child() {
    let sep: Arc<dyn Node> = Arc::new(StringNode::new(", "));
    let ip = Arc::new(InsertionPoint::with_separator(sep));
    ip.add(Arc::new(StringNode::new("only")));
    ip.close();
    let result = converge_to_string(ip, Context::default()).await;
    assert_eq!(result.unwrap(), "only");
}

</storycode:copy>
<a id="f-c959154fc401-integration_full_document_tree"></a>
### `integration_full_document_tree`

<storycode:copy id="f-c959154fc401-integration_full_document_tree">
// ===========================================================================
// Integration tests
// ===========================================================================

#[tokio::test]
async fn integration_full_document_tree() {
    // Simulates a simple document: <header>Title</header><body>Content</body>
    let document = Arc::new(InsertionPoint::new());

    let header = Arc::new(InsertionPoint::new());
    header.add(Arc::new(StringNode::new("Title")));
    header.close();

    let body = Arc::new(InsertionPoint::new());
    body.add(Arc::new(StringNode::new("Content")));
    body.close();

    document.add(header);
    document.add(body);
    document.close();

    let result = converge_to_string(document, Context::default()).await;
    assert_eq!(result.unwrap(), "TitleContent");
}

</storycode:copy>
<a id="f-c959154fc401-integration_transform_chain"></a>
### `integration_transform_chain`

<storycode:copy id="f-c959154fc401-integration_transform_chain">
#[tokio::test]
async fn integration_transform_chain() {
    // Two transforms in sequence: uppercase then wrap
    let leaf = Arc::new(InsertionPoint::new());
    leaf.add(Arc::new(StringNode::new("hello")));
    leaf.close();

    let upper_children = Arc::new(InsertionPoint::new());
    upper_children.add(Arc::new(TransformNode::new(
        leaf,
        |s| s.to_uppercase(),
        "upper",
    )));
    upper_children.close();

    let wrapped: Arc<dyn Node> = Arc::new(TransformNode::new(
        upper_children,
        |s| format!("[{s}]"),
        "wrap",
    ));

    let result = converge_to_string(wrapped, Context::default()).await;
    assert_eq!(result.unwrap(), "[HELLO]");
}

</storycode:copy>
<a id="f-c959154fc401-integration_deep_provenance"></a>
### `integration_deep_provenance`

<storycode:copy id="f-c959154fc401-integration_deep_provenance">
#[tokio::test]
async fn integration_deep_provenance() {
    // Verify that trace depth accumulates through nested structures
    let leaf: Arc<dyn Node> = Arc::new(StringNode::new("data"));
    let ip1 = Arc::new(InsertionPoint::new());
    ip1.add(leaf);
    ip1.close();
    let ip2 = Arc::new(InsertionPoint::new());
    ip2.add(ip1);
    ip2.close();

    let ip2_node: Arc<dyn Node> = ip2;
    let mut stream = ip2_node.get_stream(Context::default());
    let batch = stream.next().await.unwrap();
    assert!(!batch.is_empty());
    // Each InsertionPoint adds to the trace, so depth >= 2
    assert!(batch[0].trace().len() >= 2);
}

</storycode:copy>
<a id="f-c959154fc401-integration_concurrent_add_remove"></a>
### `integration_concurrent_add_remove`

<storycode:copy id="f-c959154fc401-integration_concurrent_add_remove">
#[tokio::test]
async fn integration_concurrent_add_remove() {
    // Test dynamic add/remove: start with A+B, replace B with C, verify final is AC
    let ip = Arc::new(InsertionPoint::new());
    let child_a: Arc<dyn Node> = Arc::new(StringNode::new("A"));
    let child_c: Arc<dyn Node> = Arc::new(StringNode::new("C"));

    ip.add(child_a.clone());
    ip.add(child_c);
    ip.close();

    let result = converge_to_string(ip, Context::default()).await;
    assert_eq!(result.unwrap(), "AC");
}

</storycode:copy>
<a id="f-c959154fc401-integration_context_extension_flow"></a>
### `integration_context_extension_flow`

<storycode:copy id="f-c959154fc401-integration_context_extension_flow">
#[tokio::test]
async fn integration_context_extension_flow() {
    // Verify that context extensions are available through the node tree
    #[derive(Clone)]
    struct TestConfig {
        prefix: String,
    }

    let ctx = Context::new().with_extension(TestConfig {
        prefix: ">>".to_string(),
    });

    // The extension survives being passed through the node tree
    let ctx2 = ctx.clone();
    let config = ctx2.get_extension::<TestConfig>().unwrap();
    assert_eq!(config.prefix, ">>");

    // Nodes can use the context
    let node: Arc<dyn Node> = Arc::new(StringNode::new("test"));
    let result = converge_to_string(node, ctx).await;
    assert_eq!(result.unwrap(), "test");
}

</storycode:copy>
<a id="f-c959154fc401-span_node_has_string_value"></a>
### `span_node_has_string_value`

<storycode:copy id="f-c959154fc401-span_node_has_string_value">
// ===========================================================================
// SpanNode tests
// ===========================================================================

#[test]
fn span_node_has_string_value() {
    use hick_flow::{SourceOrigin, SpanNode};
    let node = SpanNode::new("hello", SourceOrigin::Synthetic);
    assert_eq!(node.as_string_value(), Some("hello"));
    assert_eq!(node.value(), "hello");
}

</storycode:copy>
<a id="f-c959154fc401-span_node_returns_source_origin"></a>
### `span_node_returns_source_origin`

<storycode:copy id="f-c959154fc401-span_node_returns_source_origin">
#[test]
fn span_node_returns_source_origin() {
    use hick_flow::{SourceOrigin, SpanNode};
    use hick_lang::SourceSpan;

    let span = SourceSpan::new(10, 20, 3, 5);
    let origin = SourceOrigin::Literal {
        file: Arc::from("test.md"),
        span,
    };
    let node = SpanNode::new("some text", origin);
    let retrieved = node.source_origin().expect("should have origin");
    match retrieved {
        SourceOrigin::Literal { file, span } => {
            assert_eq!(&**file, "test.md");
            assert_eq!(span.start, 10);
            assert_eq!(span.end, 20);
            assert_eq!(span.start_line, 3);
            assert_eq!(span.start_col, 5);
        }
        _ => panic!("Expected Literal origin"),
    }
}

</storycode:copy>
<a id="f-c959154fc401-span_node_converges_like_string_node"></a>
### `span_node_converges_like_string_node`

<storycode:copy id="f-c959154fc401-span_node_converges_like_string_node">
#[tokio::test]
async fn span_node_converges_like_string_node() {
    use hick_flow::{SourceOrigin, SpanNode};
    let span_node: Arc<dyn Node> = Arc::new(SpanNode::new("hello world", SourceOrigin::Synthetic));
    let result = converge_to_string(span_node, Context::default()).await;
    assert_eq!(result.unwrap(), "hello world");
}

</storycode:copy>
<a id="f-c959154fc401-span_node_in_insertion_point"></a>
### `span_node_in_insertion_point`

<storycode:copy id="f-c959154fc401-span_node_in_insertion_point">
#[tokio::test]
async fn span_node_in_insertion_point() {
    use hick_flow::{SourceOrigin, SpanNode};
    use hick_lang::SourceSpan;

    let ip = Arc::new(InsertionPoint::new());
    ip.add(Arc::new(SpanNode::new(
        "hello ",
        SourceOrigin::Literal {
            file: Arc::from("test.md"),
            span: SourceSpan::new(0, 6, 1, 0),
        },
    )));
    ip.add(Arc::new(SpanNode::new(
        "world",
        SourceOrigin::Literal {
            file: Arc::from("test.md"),
            span: SourceSpan::new(6, 11, 1, 6),
        },
    )));
    ip.close();
    let result = converge_to_string(ip, Context::default()).await;
    assert_eq!(result.unwrap(), "hello world");
}

</storycode:copy>
<a id="f-c959154fc401-converge_with_provenance_tracks_span_nodes"></a>
### `converge_with_provenance_tracks_span_nodes`

<storycode:copy id="f-c959154fc401-converge_with_provenance_tracks_span_nodes">
// ---------------------------------------------------------------------------
// converge_with_provenance tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn converge_with_provenance_tracks_span_nodes() {
    use hick_flow::{SourceOrigin, SpanNode, converge_with_provenance};
    use hick_lang::SourceSpan;

    let ip = Arc::new(InsertionPoint::new());
    ip.add(Arc::new(SpanNode::new(
        "hello ",
        SourceOrigin::Literal {
            file: Arc::from("a.md"),
            span: SourceSpan::new(0, 6, 1, 0),
        },
    )));
    ip.add(Arc::new(SpanNode::new(
        "world",
        SourceOrigin::Paste {
            selector: Arc::from("#ver"),
            file: None,
            span: None,
        },
    )));
    ip.close();

    let (text, map) = converge_with_provenance(ip, Context::default())
        .await
        .unwrap();
    assert_eq!(text, "hello world");
    assert_eq!(map.len(), 2);

    let spans = map.spans();
    assert_eq!(spans[0].output_start, 0);
    assert_eq!(spans[0].output_end, 6);
    match &spans[0].origin {
        SourceOrigin::Literal { file, .. } => assert_eq!(&**file, "a.md"),
        other => panic!("Expected Literal, got {:?}", other),
    }

    assert_eq!(spans[1].output_start, 6);
    assert_eq!(spans[1].output_end, 11);
    match &spans[1].origin {
        SourceOrigin::Paste { selector, .. } => assert_eq!(&**selector, "#ver"),
        other => panic!("Expected Paste, got {:?}", other),
    }
}

</storycode:copy>
<a id="f-c959154fc401-converge_with_provenance_plain_string_nodes_get_synthetic"></a>
### `converge_with_provenance_plain_string_nodes_get_synthetic`

<storycode:copy id="f-c959154fc401-converge_with_provenance_plain_string_nodes_get_synthetic">
#[tokio::test]
async fn converge_with_provenance_plain_string_nodes_get_synthetic() {
    use hick_flow::{SourceOrigin, converge_with_provenance};

    let ip = Arc::new(InsertionPoint::new());
    ip.add(Arc::new(StringNode::new("abc")));
    ip.add(Arc::new(StringNode::new("def")));
    ip.close();

    let (text, map) = converge_with_provenance(ip, Context::default())
        .await
        .unwrap();
    assert_eq!(text, "abcdef");
    assert_eq!(map.len(), 2);

    // Plain StringNodes have no source_origin, so they default to Synthetic
    for span in map.spans() {
        match &span.origin {
            SourceOrigin::Synthetic => {}
            other => panic!("Expected Synthetic, got {:?}", other),
        }
    }
}

</storycode:copy>
<a id="f-c959154fc401-converge_with_provenance_mixed_nodes"></a>
### `converge_with_provenance_mixed_nodes`

<storycode:copy id="f-c959154fc401-converge_with_provenance_mixed_nodes">
#[tokio::test]
async fn converge_with_provenance_mixed_nodes() {
    use hick_flow::{SourceOrigin, SpanNode, converge_with_provenance};
    use hick_lang::SourceSpan;

    let ip = Arc::new(InsertionPoint::new());
    ip.add(Arc::new(StringNode::new("pre-")));
    ip.add(Arc::new(SpanNode::new(
        "tracked",
        SourceOrigin::Literal {
            file: Arc::from("src.md"),
            span: SourceSpan::new(10, 17, 2, 0),
        },
    )));
    ip.add(Arc::new(StringNode::new("-post")));
    ip.close();

    let (text, map) = converge_with_provenance(ip, Context::default())
        .await
        .unwrap();
    assert_eq!(text, "pre-tracked-post");
    assert_eq!(map.len(), 3);

    let spans = map.spans();
    // "pre-" at [0,4) — Synthetic
    assert_eq!(spans[0].output_start, 0);
    assert_eq!(spans[0].output_end, 4);
    match &spans[0].origin {
        SourceOrigin::Synthetic => {}
        other => panic!("Expected Synthetic, got {:?}", other),
    }

    // "tracked" at [4,11) — Literal
    assert_eq!(spans[1].output_start, 4);
    assert_eq!(spans[1].output_end, 11);
    match &spans[1].origin {
        SourceOrigin::Literal { file, .. } => assert_eq!(&**file, "src.md"),
        other => panic!("Expected Literal, got {:?}", other),
    }

    // "-post" at [11,16) — Synthetic
    assert_eq!(spans[2].output_start, 11);
    assert_eq!(spans[2].output_end, 16);
}
</storycode:copy>

### Assemble `crates/hick-flow/tests/flow_tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-flow/tests/flow_tests.rs"><storycode:paste select="#f-c959154fc401-node_trace_new_has_empty_trace"/><storycode:paste select="#f-c959154fc401-node_trace_add_to_trace_preserves_final_result"/><storycode:paste select="#f-c959154fc401-node_trace_transform_changes_final_result"/><storycode:paste select="#f-c959154fc401-node_trace_accumulates_multiple_entries"/><storycode:paste select="#f-c959154fc401-node_trace_clone_is_independent"/><storycode:paste select="#f-c959154fc401-node_trace_transform_then_add_to_trace"/><storycode:paste select="#f-c959154fc401-node_trace_new_final_result_is_correct_arc"/><storycode:paste select="#f-c959154fc401-string_node_emits_value"/><storycode:paste select="#f-c959154fc401-string_node_as_string_value"/><storycode:paste select="#f-c959154fc401-string_node_stream_completes"/><storycode:paste select="#f-c959154fc401-string_node_empty_string"/><storycode:paste select="#f-c959154fc401-string_node_is_not_separator"/><storycode:paste select="#f-c959154fc401-string_node_with_stable_id"/><storycode:paste select="#f-c959154fc401-separator_node_is_separator"/><storycode:paste select="#f-c959154fc401-separator_node_wraps_value"/><storycode:paste select="#f-c959154fc401-separator_node_adds_self_to_trace"/><storycode:paste select="#f-c959154fc401-separator_node_as_string_value_is_none"/><storycode:paste select="#f-c959154fc401-insertion_point_empty_produces_output"/><storycode:paste select="#f-c959154fc401-insertion_point_single_child"/><storycode:paste select="#f-c959154fc401-insertion_point_concatenates_children"/><storycode:paste select="#f-c959154fc401-insertion_point_with_separator"/><storycode:paste select="#f-c959154fc401-insertion_point_trim_leading_separator"/><storycode:paste select="#f-c959154fc401-insertion_point_nested"/><storycode:paste select="#f-c959154fc401-insertion_point_dynamic_add"/><storycode:paste select="#f-c959154fc401-insertion_point_dynamic_remove"/><storycode:paste select="#f-c959154fc401-insertion_point_close_is_idempotent"/><storycode:paste select="#f-c959154fc401-insertion_point_deeply_nested"/><storycode:paste select="#f-c959154fc401-insertion_point_never_complete_stays_open"/><storycode:paste select="#f-c959154fc401-insertion_point_adds_self_to_trace"/><storycode:paste select="#f-c959154fc401-dcl_single_source_emits"/><storycode:paste select="#f-c959154fc401-dcl_two_sources_combine"/><storycode:paste select="#f-c959154fc401-dcl_completes_on_quiescence"/><storycode:paste select="#f-c959154fc401-dcl_remove_source"/><storycode:paste select="#f-c959154fc401-dcl_ordering_preserved"/><storycode:paste select="#f-c959154fc401-dcl_shutdown_ends_stream"/><storycode:paste select="#f-c959154fc401-dcl_multi_value_stream"/><storycode:paste select="#f-c959154fc401-dcl_identity_combiner"/><storycode:paste select="#f-c959154fc401-dcl_add_source_after_stream_started"/><storycode:paste select="#f-c959154fc401-dcl_three_sources"/><storycode:paste select="#f-c959154fc401-context_default_has_no_extensions"/><storycode:paste select="#f-c959154fc401-context_extension_roundtrip"/><storycode:paste select="#f-c959154fc401-context_multiple_extension_types"/><storycode:paste select="#f-c959154fc401-context_extension_overwrite"/><storycode:paste select="#f-c959154fc401-context_clone_preserves_extensions"/><storycode:paste select="#f-c959154fc401-context_verbose_flag"/><storycode:paste select="#f-c959154fc401-transform_uppercase"/><storycode:paste select="#f-c959154fc401-transform_replace"/><storycode:paste select="#f-c959154fc401-transform_records_in_trace"/><storycode:paste select="#f-c959154fc401-transform_nested"/><storycode:paste select="#f-c959154fc401-transform_identity"/><storycode:paste select="#f-c959154fc401-transform_empty_children"/><storycode:paste select="#f-c959154fc401-transform_with_separator"/><storycode:paste select="#f-c959154fc401-transform_multiple_children"/><storycode:paste select="#f-c959154fc401-transform_label"/><storycode:paste select="#f-c959154fc401-converge_single_string"/><storycode:paste select="#f-c959154fc401-converge_concatenation"/><storycode:paste select="#f-c959154fc401-converge_with_separator"/><storycode:paste select="#f-c959154fc401-converge_nested"/><storycode:paste select="#f-c959154fc401-converge_with_transform"/><storycode:paste select="#f-c959154fc401-converge_empty"/><storycode:paste select="#f-c959154fc401-converge_separator_single_child"/><storycode:paste select="#f-c959154fc401-integration_full_document_tree"/><storycode:paste select="#f-c959154fc401-integration_transform_chain"/><storycode:paste select="#f-c959154fc401-integration_deep_provenance"/><storycode:paste select="#f-c959154fc401-integration_concurrent_add_remove"/><storycode:paste select="#f-c959154fc401-integration_context_extension_flow"/><storycode:paste select="#f-c959154fc401-span_node_has_string_value"/><storycode:paste select="#f-c959154fc401-span_node_returns_source_origin"/><storycode:paste select="#f-c959154fc401-span_node_converges_like_string_node"/><storycode:paste select="#f-c959154fc401-span_node_in_insertion_point"/><storycode:paste select="#f-c959154fc401-converge_with_provenance_tracks_span_nodes"/><storycode:paste select="#f-c959154fc401-converge_with_provenance_plain_string_nodes_get_synthetic"/><storycode:paste select="#f-c959154fc401-converge_with_provenance_mixed_nodes"/></storycode:file>

</storycode:doc>
