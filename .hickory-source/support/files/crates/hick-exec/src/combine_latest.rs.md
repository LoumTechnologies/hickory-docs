<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-exec/src/combine_latest.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `ebc496bde4f7d5df2a12278f355c9fc0a08bc460e73d6aa28a803066e1c210da`.

<book:entry path="crates/hick-exec/src/combine_latest.rs">
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
</book:entry>

</book:doc>
