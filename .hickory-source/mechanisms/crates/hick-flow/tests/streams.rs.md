<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Check the stream algorithm without a document

These witnesses use integer and string streams directly. They check source
ordering, updates, cancellation and completion without a parser or domain context.

<storycode:copy id="stream-test-imports">
use futures::StreamExt;
use hick_flow::DynamicCombineLatest;

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

<storycode:file path="crates/hick-flow/tests/flow_tests.rs"><storycode:paste select="#stream-test-imports"/><storycode:paste select="#f-c959154fc401-dcl_single_source_emits"/><storycode:paste select="#f-c959154fc401-dcl_two_sources_combine"/><storycode:paste select="#f-c959154fc401-dcl_completes_on_quiescence"/><storycode:paste select="#f-c959154fc401-dcl_remove_source"/><storycode:paste select="#f-c959154fc401-dcl_ordering_preserved"/><storycode:paste select="#f-c959154fc401-dcl_shutdown_ends_stream"/><storycode:paste select="#f-c959154fc401-dcl_multi_value_stream"/><storycode:paste select="#f-c959154fc401-dcl_identity_combiner"/><storycode:paste select="#f-c959154fc401-dcl_add_source_after_stream_started"/><storycode:paste select="#f-c959154fc401-dcl_three_sources"/></storycode:file>
</storycode:doc>
