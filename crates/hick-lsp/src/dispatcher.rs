//! Request routing for child LSP servers.
//!
//! The [`Dispatcher`] manages a pool of child LSP processes, one per language,
//! and provides access to them by language ID.

use std::collections::HashMap;
use tokio::sync::Mutex;

use crate::child_lsp::{ChildLsp, ChildLspError};

/// Routes LSP requests to the appropriate child server by language.
///
/// Each language gets at most one child LSP process. Processes are spawned
/// lazily on first access and initialized with the workspace root.
pub(crate) struct Dispatcher {
    /// Map from language_id to child LSP server.
    children: HashMap<String, Mutex<ChildLsp>>,
}

impl Dispatcher {
    /// Create a new empty dispatcher with no child servers.
    pub(crate) fn new() -> Self {
        Self {
            children: HashMap::new(),
        }
    }

    /// Get or spawn a child LSP for the given language.
    ///
    /// If a child already exists for the language, returns a reference to its
    /// mutex. Otherwise spawns a new child, initializes it with the given
    /// `root_uri`, and stores it for future requests.
    pub(crate) async fn get_or_spawn(
        &mut self,
        language_id: &str,
        root_uri: &str,
    ) -> Result<&Mutex<ChildLsp>, ChildLspError> {
        if !self.children.contains_key(language_id) {
            let mut child = ChildLsp::spawn(language_id).await?;
            child.initialize(root_uri).await?;
            self.children
                .insert(language_id.to_string(), Mutex::new(child));
        }

        Ok(self
            .children
            .get(language_id)
            .expect("just inserted or already present"))
    }

    /// Shut down all child LSP servers.
    ///
    /// Each child receives a `shutdown` request and `exit` notification.
    /// Errors during individual shutdowns are logged but do not prevent other
    /// children from being shut down.
    pub(crate) async fn shutdown_all(&mut self) {
        for (language_id, child_mutex) in self.children.drain() {
            let mut child = child_mutex.into_inner();
            if let Err(e) = child.shutdown().await {
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
        let d = Dispatcher::new();
        assert!(d.children.is_empty());
    }
}
