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
