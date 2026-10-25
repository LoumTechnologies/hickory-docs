<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/engine/writes.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `1e677c13ede6d075e7c122dd1642e6a0835b5f5e22ebbab2375b164024633302`.

<book:entry path="crates/hickory-cli/src/engine/writes.rs">
//! One ordering for filesystem acts. Nested helpers use their caller's act.
tokio::task_local! { static HELD: (); }
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
</book:entry>

</book:doc>
