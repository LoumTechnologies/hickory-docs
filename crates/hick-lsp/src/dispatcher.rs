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
    /// Each child's semantic-token legend, captured from its initialize
    /// result.
    ///
    /// Kept because the legend is per server: pyright and rust-analyzer can
    /// disagree about which integer means `function`, so decoding one child's
    /// tokens with another's legend colours the code wrongly rather than
    /// failing visibly.
    legends: HashMap<String, (Vec<String>, Vec<String>)>,
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
            legends: HashMap::new(),
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
            let handle = ChildLspHandle::spawn(language_id, self.notification_tx.clone()).await?;
            let result = handle.initialize(root_uri).await?;
            if let Some(legend) = semantic_legend(&result) {
                self.legends.insert(language_id.to_string(), legend);
            }
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
    pub fn get_child(&self, language_id: &str) -> Result<&ChildLspHandle, ChildLspError> {
        self.children
            .get(language_id)
            .ok_or_else(|| ChildLspError::UnknownLanguage {
                language_id: language_id.to_string(),
            })
    }

    /// The semantic-token legend a child declared, if it supports them.
    pub fn token_legend(&self, language_id: &str) -> Option<(Vec<String>, Vec<String>)> {
        self.legends.get(language_id).cloned()
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

/// Pull `capabilities.semanticTokensProvider.legend` out of an initialize
/// result, whichever of the two shapes the server used.
fn semantic_legend(result: &serde_json::Value) -> Option<(Vec<String>, Vec<String>)> {
    let provider = result.pointer("/capabilities/semanticTokensProvider")?;
    // The provider is either the options object or `{ legend, ... }` nested
    // under a registration; both appear in the wild.
    let legend = provider.get("legend").or_else(|| {
        provider
            .pointer("/documentSelector")
            .and(provider.get("legend"))
    })?;
    let strings = |key: &str| -> Vec<String> {
        legend
            .get(key)
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };
    let types = strings("tokenTypes");
    if types.is_empty() {
        return None;
    }
    Some((types, strings("tokenModifiers")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_legend_is_read_from_the_initialize_result() {
        let result = serde_json::json!({
            "capabilities": {
                "semanticTokensProvider": {
                    "legend": {
                        "tokenTypes": ["function", "variable"],
                        "tokenModifiers": ["declaration"]
                    },
                    "full": true
                }
            }
        });
        let (types, modifiers) = semantic_legend(&result).expect("legend found");
        assert_eq!(types, vec!["function", "variable"]);
        assert_eq!(modifiers, vec!["declaration"]);
    }

    #[test]
    fn a_server_without_semantic_tokens_has_no_legend() {
        // Decoding its tokens against somebody else's legend would colour the
        // code wrongly rather than failing visibly, so absence must be
        // absence.
        let result = serde_json::json!({ "capabilities": { "hoverProvider": true } });
        assert!(semantic_legend(&result).is_none());
    }

    #[test]
    fn new_dispatcher_is_empty() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let d = Dispatcher::new(tx);
        assert!(d.children.is_empty());
    }
}
