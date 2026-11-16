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
    *method == Method::GET || post_observation(method, path)
}
/// These named POST routes observe supplied or captured source without publishing it.
pub(crate) fn post_observation(method: &axum::http::Method, path: &str) -> bool {
    use axum::http::Method;
    *method == Method::POST
        && matches!(
            path,
            "/api/tests/debug-plan"
                | "/api/tests/source-candidates"
                | "/api/construction/reading"
                | "/api/changes/plan-resources"
                | "/api/changes/resource-targets"
                | "/api/changes/resource-inputs"
                | "/api/changes/cleanup"
                | "/api/changes/resource-consequences"
                | "/api/changes/prepare-restoration"
        )
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
            "/api/tests/debug-plan",
            "/api/construction/reading",
            "/api/changes/plan-resources",
            "/api/changes/resource-targets",
            "/api/changes/resource-inputs",
            "/api/changes/cleanup",
            "/api/changes/resource-consequences",
            "/api/changes/prepare-restoration",
        ] {
            assert!(observation(&Method::POST, path));
            assert_eq!(
                dispatch(&writes, observation(&Method::POST, path), async { 1 })
                    .map(Result::unwrap)
                    .now_or_never(),
                Some(1)
            );
        }
        for path in [
            "/api/tests/run-source",
            "/api/file",
            "/api/files/op",
            "/api/changes/publish-resources",
            "/api/changes/resource-save",
            "/api/changes/restore-resources",
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
        assert_eq!(dispatch(&writes, false, async { 3 }).await.unwrap(), 3);
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
        task.await.unwrap().unwrap();
    }
}
/// A dispatched write outlives its HTTP waiter. Dropping a response future must
/// not release ordering while its blocking publisher is still changing source.
pub(crate) async fn dispatch<T: Send + 'static>(
    writes: &std::sync::Arc<tokio::sync::Mutex<()>>,
    observation: bool,
    work: impl std::future::Future<Output = T> + Send + 'static,
) -> Result<T, tokio::task::JoinError> {
    if observation || HELD.try_with(|_| ()).is_ok() {
        return Ok(work.await);
    }
    let writes = writes.clone();
    tokio::spawn(async move { during(&writes, work).await }).await
}
#[cfg(test)]
mod request_lifetime_tests {
    use super::*;
    use futures::FutureExt;
    use std::{sync::Arc, time::Duration};
    use tokio::sync::{Mutex, oneshot};
    #[tokio::test]
    async fn losing_the_http_waiter_cannot_release_an_active_write_gate() {
        let writes = Arc::new(Mutex::new(()));
        let (enter, entered) = oneshot::channel();
        let (release, blocked) = oneshot::channel();
        let (complete, completed) = oneshot::channel();
        let caller = {
            let writes = writes.clone();
            tokio::spawn(async move {
                dispatch(&writes, false, async move {
                    enter.send(()).unwrap();
                    blocked.await.unwrap();
                    complete.send(()).unwrap();
                })
                .await
            })
        };
        entered.await.unwrap();
        caller.abort();
        assert!(caller.await.unwrap_err().is_cancelled());
        assert!(writes.try_lock().is_err());
        assert_eq!(dispatch(&writes, true, async { 7 }).await.unwrap(), 7);
        assert!(during(&writes, async { 8 }).now_or_never().is_none());
        release.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(2), completed)
            .await
            .unwrap()
            .unwrap();
        let _available = tokio::time::timeout(Duration::from_secs(2), writes.lock())
            .await
            .unwrap();
    }
    #[tokio::test]
    async fn an_admitted_queued_write_outlives_its_waiter_and_nested_work_uses_its_act() {
        let writes = Arc::new(Mutex::new(()));
        let held = writes.lock().await;
        let (complete, completed) = oneshot::channel();
        assert!(
            dispatch(&writes, false, async move {
                complete.send(9).unwrap();
            })
            .now_or_never()
            .is_none()
        );
        drop(held);
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(2), completed)
                .await
                .unwrap()
                .unwrap(),
            9
        );
        let nested = during(&writes, async {
            dispatch(&writes, false, async { 11 }).await.unwrap()
        });
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(2), nested)
                .await
                .unwrap(),
            11
        );
    }
}
