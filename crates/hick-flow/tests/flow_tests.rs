use futures::StreamExt;
use hick_flow::DynamicCombineLatest;

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
