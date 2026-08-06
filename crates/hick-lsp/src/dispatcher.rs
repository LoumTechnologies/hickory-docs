//! Request routing for child LSP servers.
//!
//! The [`Dispatcher`] manages a pool of child LSP processes, one per language,
//! and provides access to them by language ID.

use std::collections::HashMap;

use tokio::sync::mpsc;

use crate::child_lsp::{ChildLspError, ChildLspHandle, ChildNotification};

/// Routes LSP requests to the appropriate child server by language.
///
/// Each language gets at most one child LSP process. Processes are spawned
/// lazily on first access and initialized with the workspace root.
pub struct Dispatcher {
    /// Map from language_id to child LSP handle.
    children: HashMap<String, ChildLspHandle>,
    /// Shared sender for all child notifications.
    notification_tx: mpsc::UnboundedSender<ChildNotification>,
}

impl Dispatcher {
    /// Create a new empty dispatcher.
    ///
    /// The `notification_tx` is shared with all spawned children — they all
    /// send their notifications (e.g., `publishDiagnostics`) to this channel.
    pub fn new(notification_tx: mpsc::UnboundedSender<ChildNotification>) -> Self {
        Self {
            children: HashMap::new(),
            notification_tx,
        }
    }

    /// Get or spawn a child LSP for the given language.
    ///
    /// If a child already exists for the language, returns a reference to it.
    /// Otherwise spawns a new child, initializes it with the given `root_uri`,
    /// and stores it for future requests.
    pub async fn get_or_spawn(
        &mut self,
        language_id: &str,
        root_uri: &str,
    ) -> Result<&ChildLspHandle, ChildLspError> {
        if !self.children.contains_key(language_id) {
            let handle = ChildLspHandle::spawn(
                language_id,
                self.notification_tx.clone(),
            )
            .await?;
            handle.initialize(root_uri).await?;
            self.children.insert(language_id.to_string(), handle);
        }

        Ok(self
            .children
            .get(language_id)
            .expect("just inserted or already present"))
    }

    /// Get an existing child LSP handle without spawning.
    ///
    /// Returns `Err` if no child exists for the given language.
    pub fn get_child(
        &self,
        language_id: &str,
    ) -> Result<&ChildLspHandle, ChildLspError> {
        self.children
            .get(language_id)
            .ok_or_else(|| ChildLspError::UnknownLanguage {
                language_id: language_id.to_string(),
            })
    }

    /// Shut down all child LSP servers.
    pub async fn shutdown_all(&mut self) {
        for (language_id, handle) in self.children.drain() {
            if let Err(e) = handle.shutdown().await {
                tracing::warn!(
                    language_id,
                    error = %e,
                    "error shutting down child LSP server"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_dispatcher_is_empty() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let d = Dispatcher::new(tx);
        assert!(d.children.is_empty());
    }
}
