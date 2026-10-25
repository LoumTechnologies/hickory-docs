<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/engine/client/lifecycle.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `6f02079bca622bccfacea7ddbcdeeacd9644cd7ad8acfc2beb322038dd05211d`.

<book:entry path="crates/hickory-cli/src/engine/client/lifecycle.rs">
use super::*;

pub struct ClientGuard {
    task: std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
    connection: Connection,
    detached: std::sync::atomic::AtomicBool,
    detach_gate: tokio::sync::Mutex<()>,
}
impl ClientGuard {
    pub(super) fn new(connection: Connection) -> Self {
        let task = heartbeat(connection.clone());
        Self {
            task: std::sync::Mutex::new(Some(task)),
            connection,
            detached: std::sync::atomic::AtomicBool::new(false),
            detach_gate: tokio::sync::Mutex::new(()),
        }
    }
    async fn stop_heartbeat(&self) {
        let task = self.task.lock().unwrap().take();
        if let Some(task) = task {
            task.abort();
            // Abort requests cancellation; joining proves its reconnect future
            // has stopped before the terminal DELETE is dispatched.
            let _ = task.await;
        }
    }
    /// A normal desktop exit must release its window slot before a quick
    /// replacement attaches. Drop alone cannot await an acknowledgement.
    pub async fn detach(&self) -> Result<()> {
        let _closing = self.detach_gate.lock().await;
        if self.detached.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }
        if self
            .connection
            .private_ended
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            self.stop_heartbeat().await;
            self.detached
                .store(true, std::sync::atomic::Ordering::SeqCst);
            return Ok(());
        }
        let previously_ended = self
            .connection
            .ended
            .swap(true, std::sync::atomic::Ordering::SeqCst);
        self.stop_heartbeat().await;
        if let Err(error) = self.connection.detach().await {
            self.connection
                .ended
                .store(previously_ended, std::sync::atomic::Ordering::SeqCst);
            if !previously_ended {
                *self.task.lock().unwrap() = Some(heartbeat(self.connection.clone()));
            }
            return Err(error);
        }
        self.detached
            .store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}
impl Drop for ClientGuard {
    fn drop(&mut self) {
        if let Some(task) = self.task.lock().unwrap().take() {
            task.abort();
        }
        if !self
            .connection
            .ended
            .load(std::sync::atomic::Ordering::SeqCst)
            && let Ok(runtime) = tokio::runtime::Handle::try_current()
        {
            let connection = self.connection.clone();
            runtime.spawn(async move {
                let _ = connection.detach().await;
            });
        }
    }
}
fn heartbeat(connection: Connection) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(HEARTBEAT).await;
            if connection.ended.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            let lease = connection
                .request(
                    reqwest::Method::POST,
                    &format!("/clients/{}/lease", connection.attach.client),
                )
                .await
                .send()
                .await;
            if !lease.is_ok_and(|r| r.status().is_success())
                && let Err(error) = connection.reconnect().await
            {
                log::warn!("engine reconnect: {error:#}");
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, http::StatusCode, routing::post};
    use std::path::PathBuf;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    fn fixture_connection(url: String) -> Connection {
        let mut attach = Attach::new(
            PathBuf::from("/unused-fixture"),
            crate::ExecutorChoice::Local,
        );
        attach.client = "owner".into();
        Connection {
            endpoint: Arc::new(tokio::sync::RwLock::new(super::super::super::Endpoint {
                url,
                token: "fixture".into(),
                protocol: PROTOCOL,
                pid: std::process::id(),
            })),
            ended: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            private_ended: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            attach,
            slot: "0".into(),
            log_offset: 0,
            http: reqwest::Client::builder().no_proxy().build().unwrap(),
        }
    }
    #[tokio::test]
    async fn detach_requires_ack_restores_heartbeat_on_refusal_and_stops_it_after_success() {
        let deletes = Arc::new(AtomicUsize::new(0));
        let leases = Arc::new(AtomicUsize::new(0));
        let notified = Arc::new(tokio::sync::Notify::new());
        let seen_delete = deletes.clone();
        let seen_lease = leases.clone();
        let lease_received = notified.clone();
        let router = Router::new().route(
            "/clients/owner/lease",
            post(move || {
                seen_lease.fetch_add(1, Ordering::SeqCst);
                lease_received.notify_one();
                async { StatusCode::NO_CONTENT }
            })
            .delete(move || {
                let attempt = seen_delete.fetch_add(1, Ordering::SeqCst);
                async move {
                    if attempt == 0 {
                        StatusCode::SERVICE_UNAVAILABLE
                    } else {
                        StatusCode::NO_CONTENT
                    }
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        let connection = fixture_connection(url);
        let guard = connection.guard();
        assert!(
            guard.detach().await.is_err(),
            "a500 is not an acknowledgement"
        );
        assert!(!connection.ended.load(Ordering::SeqCst));
        tokio::time::timeout(std::time::Duration::from_secs(3), notified.notified())
            .await
            .unwrap();
        assert!(
            leases.load(Ordering::SeqCst) > 0,
            "failed exit must keep ownership alive"
        );
        guard.detach().await.unwrap();
        assert!(connection.ended.load(Ordering::SeqCst));
        let before = leases.load(Ordering::SeqCst);
        tokio::time::sleep(HEARTBEAT + std::time::Duration::from_millis(100)).await;
        assert_eq!(leases.load(Ordering::SeqCst), before);
        drop(guard);
        assert_eq!(
            deletes.load(Ordering::SeqCst),
            2,
            "successful close is not detached twice"
        );
        server.abort();
    }
    #[tokio::test]
    async fn acknowledged_private_end_closes_after_its_engine_is_gone() {
        let router = Router::new().route(
            "/private/end",
            post(|| async { axum::Json(serde_json::json!({"ended":true})) }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let connection = fixture_connection(format!("http://{}", listener.local_addr().unwrap()));
        let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        let guard = connection.guard();
        connection.end_private().await.unwrap();
        assert!(connection.private_ended.load(Ordering::SeqCst));
        server.abort();
        let _ = server.await;
        tokio::time::timeout(std::time::Duration::from_millis(500), guard.detach())
            .await
            .unwrap()
            .unwrap();
        guard.detach().await.unwrap();
    }

    #[tokio::test]
    async fn concurrent_detaches_wait_for_one_acknowledgement() {
        let deletes = Arc::new(AtomicUsize::new(0));
        let started = Arc::new(tokio::sync::Notify::new());
        let release = Arc::new(tokio::sync::Notify::new());
        let count = deletes.clone();
        let observed = started.clone();
        let acknowledgement = release.clone();
        let router = Router::new().route(
            "/clients/owner/lease",
            axum::routing::delete(move || {
                let attempt = count.fetch_add(1, Ordering::SeqCst);
                let observed = observed.clone();
                let acknowledgement = acknowledgement.clone();
                async move {
                    observed.notify_one();
                    if attempt == 0 {
                        acknowledgement.notified().await;
                    }
                    StatusCode::NO_CONTENT
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let connection = fixture_connection(format!("http://{}", listener.local_addr().unwrap()));
        let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        let guard = Arc::new(connection.guard());
        let first = guard.clone();
        let first = tokio::spawn(async move { first.detach().await });
        started.notified().await;
        let second = guard.clone();
        let second = tokio::spawn(async move { second.detach().await });
        tokio::task::yield_now().await;
        assert!(
            !second.is_finished(),
            "a temporary ended fence is not a completed detach"
        );
        release.notify_one();
        first.await.unwrap().unwrap();
        second.await.unwrap().unwrap();
        assert_eq!(deletes.load(Ordering::SeqCst), 1);
        server.abort();
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn detach_waits_for_heartbeat_cancellation_before_http_delete() {
        struct Cleanup(std::sync::Arc<std::sync::atomic::AtomicBool>);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                // Make incomplete cancellation observable on the other worker.
                std::thread::sleep(std::time::Duration::from_millis(50));
                self.0.store(true, Ordering::SeqCst);
            }
        }
        let stopped = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let observed = stopped.clone();
        let router = Router::new().route(
            "/clients/owner/lease",
            axum::routing::delete(move || {
                let stopped = observed.load(Ordering::SeqCst);
                async move {
                    if stopped {
                        StatusCode::NO_CONTENT
                    } else {
                        StatusCode::CONFLICT
                    }
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let connection = fixture_connection(format!("http://{}", listener.local_addr().unwrap()));
        let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        let guard = connection.guard();
        guard.stop_heartbeat().await;
        let (ready, started) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            let _cleanup = Cleanup(stopped);
            ready.send(()).unwrap();
            std::future::pending::<()>().await;
        });
        started.await.unwrap();
        *guard.task.lock().unwrap() = Some(task);
        guard
            .detach()
            .await
            .expect("DELETE must observe completed heartbeat cancellation");
        server.abort();
    }
}
</book:entry>

</book:doc>
