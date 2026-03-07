//! `DynamicCombineLatest` — re-exported from `hick-flow`.

pub use hick_flow::DynamicCombineLatest;

// ---------------------------------------------------------------------------
// Tests (validate the re-export surface)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[tokio::test]
    async fn single_source_emits() {
        let dcl = DynamicCombineLatest::new(|v: Vec<i32>| v.iter().sum::<i32>());
        dcl.add_source(futures::stream::once(async { 42 }));

        let mut stream = dcl.stream();
        let val = stream.next().await.unwrap();
        assert_eq!(val, 42);
    }

    #[tokio::test]
    async fn two_sources_combine() {
        let dcl = DynamicCombineLatest::new(|v: Vec<i32>| v.iter().sum::<i32>());
        dcl.add_source(futures::stream::once(async { 10 }));
        dcl.add_source(futures::stream::once(async { 20 }));

        let mut stream = dcl.stream();
        let val = stream.next().await.unwrap();
        assert_eq!(val, 30);
    }

    #[tokio::test]
    async fn completes_on_quiescence() {
        let dcl = DynamicCombineLatest::new(|v: Vec<i32>| v.iter().sum::<i32>());
        dcl.add_source(futures::stream::once(async { 1 }));

        let results: Vec<_> = dcl.stream().collect().await;
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn remove_source_works() {
        let dcl = DynamicCombineLatest::new(|v: Vec<String>| v.join(","));

        let id1 = dcl.add_source(futures::stream::once(async { "a".to_string() }));
        dcl.add_source(futures::stream::once(async { "b".to_string() }));

        let mut stream = dcl.stream();
        let val = stream.next().await.unwrap();
        assert!(val.contains("a") && val.contains("b"));

        dcl.remove_source(id1);
    }
}
