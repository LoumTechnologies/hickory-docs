<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/engine/writes.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5b2a07360017"></a>
## `crates/hickory-cli/src/engine/writes.rs`

<a id="f-5b2a07360017-macro_invocation"></a>
### `macro_invocation`

One ordering for filesystem acts. Nested helpers use their caller's act.

<storycode:copy id="f-5b2a07360017-macro_invocation">
//! One ordering for filesystem acts. Nested helpers use their caller's act.
tokio::task_local! { static HELD: (); }
</storycode:copy>
<a id="f-5b2a07360017-during"></a>
### `during`

<storycode:copy id="f-5b2a07360017-during">
pub(crate) async fn during<T>(
    writes: &std::sync::Arc<tokio::sync::Mutex<()>>,
    work: impl std::future::Future<Output = T>,
) -> T {
    if HELD.try_with(|_| ()).is_ok() {
        return work.await;
    }
    let _write = writes.lock().await;
    HELD.scope((), work).await
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/engine/writes.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/engine/writes.rs"><storycode:paste select="#f-5b2a07360017-macro_invocation"/><storycode:paste select="#f-5b2a07360017-during"/></storycode:file>

</storycode:doc>
