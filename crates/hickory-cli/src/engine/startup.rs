//! An attached editor can boot while output ownership is being established.
use anyhow::{Result, anyhow};
use tokio::sync::watch;

#[derive(Clone)]
pub(super) struct Startup {
    receiver: watch::Receiver<Option<Result<(), String>>>,
    progress: std::sync::Arc<std::sync::Mutex<Progress>>,
    started: std::time::Instant,
}
#[derive(Clone)]
struct Progress {
    phase: String,
    completed: usize,
    total: usize,
    document: Option<String>,
    changed: std::time::Instant,
}

impl Startup {
    pub fn complete() -> Self {
        let (sender, startup) = Self::channel();
        let _ = sender.send(Some(Ok(())));
        startup
    }
    pub fn result(&self) -> Option<Result<(), String>> {
        self.receiver.borrow().clone().or_else(|| {
            self.receiver
                .has_changed()
                .is_err()
                .then(|| Err("Workspace preparation stopped. Reopen the folder.".into()))
        })
    }

    pub fn channel() -> (watch::Sender<Option<Result<(), String>>>, Self) {
        let (tx, rx) = watch::channel(None);
        let now = std::time::Instant::now();
        (
            tx,
            Self {
                receiver: rx,
                started: now,
                progress: std::sync::Arc::new(std::sync::Mutex::new(Progress {
                    phase: "queued".into(),
                    completed: 0,
                    total: 0,
                    document: None,
                    changed: now,
                })),
            },
        )
    }

    pub fn progress(&self, phase: &str, completed: usize, total: usize, document: Option<String>) {
        *self.progress.lock().unwrap() = Progress {
            phase: phase.into(),
            completed,
            total,
            document,
            changed: std::time::Instant::now(),
        };
    }
    pub fn status(&self) -> serde_json::Value {
        let progress = self.progress.lock().unwrap();
        serde_json::json!({ "ready": false, "phase": progress.phase, "completed": progress.completed,
            "total": progress.total, "document": progress.document,
            "elapsed_ms": self.started.elapsed().as_millis() as u64,
            "stage_elapsed_ms": progress.changed.elapsed().as_millis() as u64 })
    }

    pub async fn wait(&self) -> Result<()> {
        let mut receiver = self.receiver.clone();
        loop {
            if let Some(result) = receiver.borrow_and_update().clone() {
                return result.map_err(|error| anyhow!(error));
            }
            receiver
                .changed()
                .await
                .map_err(|_| anyhow!("Workspace preparation stopped. Reopen the folder."))?;
        }
    }
}

/// Only editor bootstrap/storage may bypass initialization. In particular,
/// file reads, writes, execution, and collaboration sockets must wait until
/// generated-file ownership and the reverse-edit baselines are established.
pub(super) fn editor_bootstrap(path: &str, method: &axum::http::Method) -> bool {
    use axum::http::Method;
    matches!(
        (method, path),
        (
            &Method::GET,
            "/api/health" | "/api/settings/ui" | "/api/files" | "/api/projects"
        ) | (
            &Method::GET | &Method::PUT | &Method::DELETE,
            "/api/workspace/ui" | "/api/workspace/drafts"
        ) | (
            &Method::POST,
            "/api/window/close" | "/api/save-file-dialog" | "/api/pick-folder"
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn progress_is_shared_without_unlocking_preparation() {
        let (sender, startup) = Startup::channel();
        let observer = startup.clone();
        startup.progress("weaving", 2, 5, Some("sessions/large.md".into()));
        let status = observer.status();
        assert_eq!(status["ready"], false);
        assert_eq!(status["completed"], 2);
        assert_eq!(status["total"], 5);
        assert_eq!(status["document"], "sessions/large.md");
        assert!(status["elapsed_ms"].is_number());
        assert!(observer.result().is_none());
        let waiting = tokio::spawn(async move { observer.wait().await });
        tokio::task::yield_now().await;
        assert!(!waiting.is_finished());
        sender.send(Some(Ok(()))).unwrap();
        waiting.await.unwrap().unwrap();
    }

    // Guarantee: docs/guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation.md
    #[tokio::test]
    async fn a_stopped_preparation_reports_failure_instead_of_waiting_forever() {
        let (sender, startup) = Startup::channel();
        drop(sender);
        assert!(
            startup
                .result()
                .unwrap()
                .unwrap_err()
                .contains("Reopen the folder")
        );
        assert!(startup.wait().await.is_err());
    }
}
