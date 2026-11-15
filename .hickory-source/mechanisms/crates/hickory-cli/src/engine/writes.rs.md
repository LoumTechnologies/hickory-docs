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

## Keep observations independent of filesystem acts

Named analysis requests read snapshots without holding the filesystem write gate. Unknown POST requests retain conservative serialization. The tests hold the gate while observations finish, and wait until a pending observation has actually entered before checking that a save can acquire it.

<storycode:copy id="observation-dispatch">
/// Preserve the existing GET policy and identify named POST observations explicitly.
/// New or unknown POST routes remain serialized filesystem acts by default.
pub(crate) fn observation(method: &axum::http::Method, path: &str) -> bool {
    use axum::http::Method;
    *method == Method::GET
        || (*method == Method::POST
            && matches!(
                path,
                "/api/tests/source-candidates"
                    | "/api/construction/reading"
                    | "/api/changes/plan-resources"
                    | "/api/changes/resource-targets"
                    | "/api/changes/resource-inputs"
                    | "/api/changes/cleanup"
                    | "/api/changes/resource-consequences"
            ))
}
/// Observation uses its own capability locks; it must not hold up unrelated saves.
pub(crate) async fn dispatch<T>(
    writes: &std::sync::Arc<tokio::sync::Mutex<()>>,
    observation: bool,
    work: impl std::future::Future<Output = T>,
) -> T {
    if observation {
        work.await
    } else {
        during(writes, work).await
    }
}
#[cfg(test)]
mod observation_tests {
    use super::*;
    use axum::http::Method;
    use futures::FutureExt;
    #[tokio::test]
    async fn known_observations_do_not_wait_for_or_take_the_file_write_gate() {
        let writes = std::sync::Arc::new(tokio::sync::Mutex::new(()));
        let held = writes.lock().await;
        for path in [
            "/api/tests/source-candidates",
            "/api/construction/reading",
            "/api/changes/plan-resources",
            "/api/changes/resource-targets",
            "/api/changes/resource-inputs",
            "/api/changes/cleanup",
            "/api/changes/resource-consequences",
        ] {
            assert!(observation(&Method::POST, path));
            assert_eq!(
                dispatch(&writes, observation(&Method::POST, path), async { 1 }).now_or_never(),
                Some(1)
            );
        }
        for path in [
            "/api/tests/run-source",
            "/api/file",
            "/api/files/op",
            "/api/changes/publish-resources",
            "/api/construction/reading/extra",
        ] {
            assert!(!observation(&Method::POST, path));
            assert!(
                dispatch(&writes, observation(&Method::POST, path), async { 2 })
                    .now_or_never()
                    .is_none()
            );
        }
        assert!(!observation(&Method::PUT, "/api/changes/resource-inputs"));
        drop(held);
        assert_eq!(dispatch(&writes, false, async { 3 }).await, 3);
    }
    #[tokio::test]
    async fn an_observation_does_not_hold_the_gate_while_its_capability_is_pending() {
        let writes = std::sync::Arc::new(tokio::sync::Mutex::new(()));
        let (release, waiting) = tokio::sync::oneshot::channel::<()>();
        let (entered, started) = tokio::sync::oneshot::channel::<()>();
        let task = {
            let writes = writes.clone();
            tokio::spawn(async move {
                dispatch(&writes, true, async {
                    entered.send(()).unwrap();
                    let _ = waiting.await;
                })
                .await
            })
        };
        started.await.unwrap();
        assert!(writes.try_lock().is_ok());
        release.send(()).unwrap();
        task.await.unwrap();
    }
}
</storycode:copy>

<storycode:file path="crates/hickory-cli/src/engine/writes.rs"><storycode:paste select="#f-5b2a07360017-macro_invocation"/><storycode:paste select="#f-5b2a07360017-during"/><storycode:paste select="#observation-dispatch"/></storycode:file>

</storycode:doc>
