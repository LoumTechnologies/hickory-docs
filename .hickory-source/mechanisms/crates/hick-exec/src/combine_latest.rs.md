<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-exec/src/combine_latest.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-bb0af076313c"></a>
## `crates/hick-exec/src/combine_latest.rs`

<a id="f-bb0af076313c-tests"></a>
### `tests`

`DynamicCombineLatest` — re-exported from `hick-flow`.

<storycode:copy id="f-bb0af076313c-tests">
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
</storycode:copy>

### Assemble `crates/hick-exec/src/combine_latest.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-exec/src/combine_latest.rs"><storycode:paste select="#f-bb0af076313c-tests"/></storycode:file>

</storycode:doc>
