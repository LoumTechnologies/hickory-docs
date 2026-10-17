//! An attached editor can boot while output ownership is being established.
use anyhow::{Result, anyhow};
use tokio::sync::watch;

#[derive(Clone)]
pub(super) struct Startup(watch::Receiver<Option<Result<(), String>>>);

impl Startup {
    pub fn result(&self) -> Option<Result<(), String>> {
        self.0.borrow().clone().or_else(|| {
            self.0
                .has_changed()
                .is_err()
                .then(|| Err("Workspace preparation stopped. Reopen the folder.".into()))
        })
    }

    pub fn channel() -> (watch::Sender<Option<Result<(), String>>>, Self) {
        let (tx, rx) = watch::channel(None);
        (tx, Self(rx))
    }

    pub async fn wait(&self) -> Result<()> {
        let mut receiver = self.0.clone();
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
