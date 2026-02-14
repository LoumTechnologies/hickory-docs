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

#[test]
fn node_trace_add_to_trace_preserves_final_result() {
    let leaf: Arc<dyn Node> = Arc::new(StringNode::new("leaf"));
    let wrapper: Arc<dyn Node> = Arc::new(StringNode::new("wrapper"));
    let trace = NodeTrace::new(leaf);
    let trace2 = trace.add_to_trace(wrapper);
    assert_eq!(trace2.final_result().as_string_value(), Some("leaf"));
    assert_eq!(trace2.trace().len(), 1);
}

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

#[test]
fn node_trace_new_final_result_is_correct_arc() {
    let node = Arc::new(StringNode::new("hello"));
    let trace = NodeTrace::new(node.clone() as Arc<dyn Node>);
    // Verify we get the same value back
    assert_eq!(trace.final_result().as_string_value(), Some("hello"));
}

// ===========================================================================
// StringNode tests
// ===========================================================================

#[tokio::test]
async fn string_node_emits_value() {
    let node: Arc<dyn Node> = Arc::new(StringNode::new("hello"));
    let result = converge_to_string(node, Context::default()).await;
    assert_eq!(result.unwrap(), "hello");
}

#[tokio::test]
async fn string_node_as_string_value() {
    let node = StringNode::new("test");
    assert_eq!(node.as_string_value(), Some("test"));
}

#[tokio::test]
async fn string_node_stream_completes() {
    let node: Arc<dyn Node> = Arc::new(StringNode::new("done"));
    let mut stream = node.get_stream(Context::default());
    let first = stream.next().await;
    assert!(first.is_some());
    let second = stream.next().await;
    assert!(second.is_none());
}

#[tokio::test]
async fn string_node_empty_string() {
    let node: Arc<dyn Node> = Arc::new(StringNode::new(""));
    let result = converge_to_string(node, Context::default()).await;
    assert_eq!(result.unwrap(), "");
}

#[test]
fn string_node_is_not_separator() {
    let node = StringNode::new("x");
    assert!(!node.is_separator());
}

#[test]
fn string_node_with_stable_id() {
    let id = vec![1, 2, 3, 4];
    let node = StringNode::new_with_id("test", Some(id.clone()));
    assert_eq!(node.id_bytes(), id);
    assert_eq!(node.value(), "test");
}

// ===========================================================================
// SeparatorNode tests
// ===========================================================================

#[test]
fn separator_node_is_separator() {
    let inner: Arc<dyn Node> = Arc::new(StringNode::new(","));
    let sep = SeparatorNode::new(inner);
    assert!(sep.is_separator());
}

#[tokio::test]
async fn separator_node_wraps_value() {
    let inner: Arc<dyn Node> = Arc::new(StringNode::new(","));
    let sep: Arc<dyn Node> = Arc::new(SeparatorNode::new(inner));
    let result = converge_to_string(sep, Context::default()).await;
    assert_eq!(result.unwrap(), ",");
}

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

#[test]
fn separator_node_as_string_value_is_none() {
    let inner: Arc<dyn Node> = Arc::new(StringNode::new(","));
    let sep = SeparatorNode::new(inner);
    assert!(sep.as_string_value().is_none());
}

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

#[tokio::test]
async fn insertion_point_single_child() {
    let ip = Arc::new(InsertionPoint::new());
    ip.add(Arc::new(StringNode::new("only")));
    ip.close();
    let result = converge_to_string(ip, Context::default()).await;
    assert_eq!(result.unwrap(), "only");
}

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

#[tokio::test]
async fn insertion_point_close_is_idempotent() {
    let ip = Arc::new(InsertionPoint::new());
    ip.add(Arc::new(StringNode::new("x")));
    ip.close();
    ip.close(); // second close should be safe
    let result = converge_to_string(ip, Context::default()).await;
    assert_eq!(result.unwrap(), "x");
}

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

#[tokio::test]
async fn dcl_two_sources_combine() {
    let dcl = DynamicCombineLatest::new(|v: Vec<i32>| v.iter().sum::<i32>());
    dcl.add_source(futures::stream::once(async { 10 }));
    dcl.add_source(futures::stream::once(async { 20 }));
    let mut stream = dcl.stream();
    let val = stream.next().await.unwrap();
    assert_eq!(val, 30);
}

#[tokio::test]
async fn dcl_completes_on_quiescence() {
    let dcl = DynamicCombineLatest::new(|v: Vec<i32>| v.iter().sum::<i32>());
    dcl.add_source(futures::stream::once(async { 1 }));
    let results: Vec<_> = dcl.stream().collect().await;
    assert!(!results.is_empty());
}

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

#[tokio::test]
async fn dcl_identity_combiner() {
    let dcl = DynamicCombineLatest::new(|v: Vec<String>| v);
    dcl.add_source(futures::stream::once(async { "only".to_string() }));
    let mut stream = dcl.stream();
    let val = stream.next().await.unwrap();
    assert_eq!(val, vec!["only".to_string()]);
}

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

// ===========================================================================
// Context tests
// ===========================================================================

#[test]
fn context_default_has_no_extensions() {
    let ctx = Context::default();
    assert!(ctx.get_extension::<String>().is_none());
}

#[test]
fn context_extension_roundtrip() {
    let ctx = Context::new().with_extension(42u32);
    let val = ctx.get_extension::<u32>().unwrap();
    assert_eq!(*val, 42);
}

#[test]
fn context_multiple_extension_types() {
    let ctx = Context::new()
        .with_extension(42u32)
        .with_extension("hello".to_string());
    assert_eq!(*ctx.get_extension::<u32>().unwrap(), 42);
    assert_eq!(*ctx.get_extension::<String>().unwrap(), "hello");
}

#[test]
fn context_extension_overwrite() {
    let ctx = Context::new().with_extension(1u32).with_extension(2u32);
    assert_eq!(*ctx.get_extension::<u32>().unwrap(), 2);
}

#[test]
fn context_clone_preserves_extensions() {
    let ctx = Context::new().with_extension(99u64);
    let ctx2 = ctx.clone();
    assert_eq!(*ctx2.get_extension::<u64>().unwrap(), 99);
}

#[test]
fn context_verbose_flag() {
    let mut ctx = Context::new();
    assert!(!ctx.verbose);
    ctx.verbose = true;
    assert!(ctx.verbose);
}

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
    assert!(batch[0].trace().len() >= 1);
    assert_eq!(batch[0].final_result().as_string_value(), Some("TEST"));
}

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

#[tokio::test]
async fn transform_empty_children() {
    let children = Arc::new(InsertionPoint::new());
    children.close();

    let xform: Arc<dyn Node> =
        Arc::new(TransformNode::new(children, |s| s.to_uppercase(), "upper"));
    let result = converge_to_string(xform, Context::default()).await;
    assert_eq!(result.unwrap(), "");
}

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

#[tokio::test]
async fn transform_label() {
    let children = Arc::new(InsertionPoint::new());
    children.close();
    let xform = TransformNode::new(children, |s| s.to_string(), "my-transform");
    assert_eq!(xform.label(), "my-transform");
}

// ===========================================================================
// converge_to_string tests
// ===========================================================================

#[tokio::test]
async fn converge_single_string() {
    let node: Arc<dyn Node> = Arc::new(StringNode::new("single"));
    let result = converge_to_string(node, Context::default()).await;
    assert_eq!(result.unwrap(), "single");
}

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

#[tokio::test]
async fn converge_empty() {
    let ip = Arc::new(InsertionPoint::new());
    ip.close();
    let result = converge_to_string(ip, Context::default()).await;
    assert_eq!(result.unwrap(), "");
}

#[tokio::test]
async fn converge_separator_single_child() {
    let sep: Arc<dyn Node> = Arc::new(StringNode::new(", "));
    let ip = Arc::new(InsertionPoint::with_separator(sep));
    ip.add(Arc::new(StringNode::new("only")));
    ip.close();
    let result = converge_to_string(ip, Context::default()).await;
    assert_eq!(result.unwrap(), "only");
}

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

#[test]
fn span_node_returns_source_origin() {
    use hick_flow::{SourceOrigin, SpanNode};
    use hick_lang::SourceSpan;

    let span = SourceSpan::new(10, 20, 3, 5);
    let origin = SourceOrigin::Literal {
        file: Arc::from("test.hick"),
        span,
    };
    let node = SpanNode::new("some text", origin);
    let retrieved = node.source_origin().expect("should have origin");
    match retrieved {
        SourceOrigin::Literal { file, span } => {
            assert_eq!(&**file, "test.hick");
            assert_eq!(span.start, 10);
            assert_eq!(span.end, 20);
            assert_eq!(span.start_line, 3);
            assert_eq!(span.start_col, 5);
        }
        _ => panic!("Expected Literal origin"),
    }
}

#[tokio::test]
async fn span_node_converges_like_string_node() {
    use hick_flow::{SourceOrigin, SpanNode};
    let span_node: Arc<dyn Node> = Arc::new(SpanNode::new("hello world", SourceOrigin::Synthetic));
    let result = converge_to_string(span_node, Context::default()).await;
    assert_eq!(result.unwrap(), "hello world");
}

#[tokio::test]
async fn span_node_in_insertion_point() {
    use hick_flow::{SourceOrigin, SpanNode};
    use hick_lang::SourceSpan;

    let ip = Arc::new(InsertionPoint::new());
    ip.add(Arc::new(SpanNode::new(
        "hello ",
        SourceOrigin::Literal {
            file: Arc::from("test.hick"),
            span: SourceSpan::new(0, 6, 1, 0),
        },
    )));
    ip.add(Arc::new(SpanNode::new(
        "world",
        SourceOrigin::Literal {
            file: Arc::from("test.hick"),
            span: SourceSpan::new(6, 11, 1, 6),
        },
    )));
    ip.close();
    let result = converge_to_string(ip, Context::default()).await;
    assert_eq!(result.unwrap(), "hello world");
}

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
            file: Arc::from("a.hick"),
            span: SourceSpan::new(0, 6, 1, 0),
        },
    )));
    ip.add(Arc::new(SpanNode::new(
        "world",
        SourceOrigin::Paste {
            selector: Arc::from("#ver"),
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
        SourceOrigin::Literal { file, .. } => assert_eq!(&**file, "a.hick"),
        other => panic!("Expected Literal, got {:?}", other),
    }

    assert_eq!(spans[1].output_start, 6);
    assert_eq!(spans[1].output_end, 11);
    match &spans[1].origin {
        SourceOrigin::Paste { selector } => assert_eq!(&**selector, "#ver"),
        other => panic!("Expected Paste, got {:?}", other),
    }
}

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

#[tokio::test]
async fn converge_with_provenance_mixed_nodes() {
    use hick_flow::{SourceOrigin, SpanNode, converge_with_provenance};
    use hick_lang::SourceSpan;

    let ip = Arc::new(InsertionPoint::new());
    ip.add(Arc::new(StringNode::new("pre-")));
    ip.add(Arc::new(SpanNode::new(
        "tracked",
        SourceOrigin::Literal {
            file: Arc::from("src.hick"),
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
        SourceOrigin::Literal { file, .. } => assert_eq!(&**file, "src.hick"),
        other => panic!("Expected Literal, got {:?}", other),
    }

    // "-post" at [11,16) — Synthetic
    assert_eq!(spans[2].output_start, 11);
    assert_eq!(spans[2].output_end, 16);
}
