//! Request routing for child LSP servers.
//!
//! The [`Dispatcher`] manages a pool of child LSP processes, one per language
//! **per root**, and provides access to them by language ID and root URI.
//!
//! Per root, not per language, because a project-aware server answers for the
//! workspace it was initialised against and no other: rust-analyzer started
//! on one document's staging directory does not own `src/main.rs` in the
//! repository the app has open, and a plain file handed to it there would
//! get syntax-only answers that read like a forwarding bug.

use std::collections::HashMap;
use std::sync::{Arc, Weak};

use tokio::sync::{mpsc, watch};

use crate::child_lsp::{ChildLspError, ChildLspHandle, ChildNotification};

/// Routes LSP requests to the appropriate child server by language.
///
/// Each (language, root) pair gets at most one child LSP process. Processes
/// are spawned lazily on first access and initialized with that root.
pub struct Dispatcher {
    /// Map from (language_id, root_uri) to child LSP handle.
    children: HashMap<(String, String), ChildLspHandle>,
    capabilities: HashMap<(String, String), serde_json::Value>,
    /// Shared sender for all child notifications.
    notification_tx: mpsc::UnboundedSender<ChildNotification>,
    /// Each child's semantic-token legend, captured from its initialize
    /// result.
    ///
    /// Kept because the legend is per server: pyright and rust-analyzer can
    /// disagree about which integer means `function`, so decoding one child's
    /// tokens with another's legend colours the code wrongly rather than
    /// failing visibly.
    legends: HashMap<(String, String), (Vec<String>, Vec<String>)>,
    starting: HashMap<(String, String), Weak<()>>,
    stopped: watch::Sender<bool>,
}

pub(crate) struct ChildStartup {
    key: (String, String),
    token: Arc<()>,
    notification_tx: mpsc::UnboundedSender<ChildNotification>,
    stopped: watch::Receiver<bool>,
    semantic_only: bool,
}

pub(crate) struct StartedChild {
    startup: ChildStartup,
    child: ChildLspHandle,
    initialized: serde_json::Value,
}

fn key(language_id: &str, root_uri: &str) -> (String, String) {
    (language_id.to_string(), root_uri.to_string())
}

impl ChildStartup {
    pub(crate) async fn start(mut self) -> Result<StartedChild, ChildLspError> {
        let (child, initialized) = tokio::select! {
            biased;
            _ = self.stopped.wait_for(|stopped| *stopped) => return Err(ChildLspError::DispatcherStopped),
            ready = async {
                let (language, root) = &self.key;
                let child = ChildLspHandle::spawn(language, root, self.notification_tx.clone()).await?;
                let initialized = child.initialize_replacement(root, self.semantic_only).await?;
                Ok::<_, ChildLspError>((child, initialized))
            } => ready?,
        };
        Ok(StartedChild { startup: self, child, initialized })
    }
}

impl Dispatcher {
    /// Create a new empty dispatcher.
    ///
    /// The `notification_tx` is shared with all spawned children — they all
    /// send their notifications (e.g., `publishDiagnostics`) to this channel.
    pub fn new(notification_tx: mpsc::UnboundedSender<ChildNotification>) -> Self {
        Self {
            children: HashMap::new(),
            capabilities: HashMap::new(),
            notification_tx,
            legends: HashMap::new(),
            starting: HashMap::new(),
            stopped: watch::channel(false).0,
        }
    }

    /// Get or spawn a child LSP for the given language and root.
    ///
    /// If a child already exists for the pair, returns a reference to it.
    /// Otherwise spawns a new child, initializes it with the given `root_uri`,
    /// and stores it for future requests.
    pub async fn get_or_spawn(
        &mut self,
        language_id: &str,
        root_uri: &str,
    ) -> Result<&ChildLspHandle, ChildLspError> {
        self.get_or_spawn_with_diagnostics(language_id, root_uri, false).await
    }

    pub(crate) async fn get_or_spawn_with_diagnostics(
        &mut self,
        language_id: &str,
        root_uri: &str,
        document_diagnostics: bool,
    ) -> Result<&ChildLspHandle, ChildLspError> {
        if *self.stopped.borrow() {
            return Err(ChildLspError::DispatcherStopped);
        }
        let k = key(language_id, root_uri);
        if !self.children.contains_key(&k) {
            if self.starting.get(&k).is_some_and(|token| token.strong_count() > 0) {
                return Err(ChildLspError::ProviderStarting);
            }
            let handle =
                ChildLspHandle::spawn(language_id, root_uri, self.notification_tx.clone()).await?;
            let result = if document_diagnostics {
                handle.initialize_with_document_diagnostics(root_uri, true).await?
            } else {
                handle.initialize(root_uri).await?
            };
            self.install_child(k.clone(), handle, &result);
        }

        Ok(self
            .children
            .get(&k)
            .expect("just inserted or already present"))
    }

    pub(crate) fn detach_child(&mut self, language_id: &str, root_uri: &str) -> Option<ChildLspHandle> {
        let key = key(language_id, root_uri);
        self.capabilities.remove(&key);
        self.legends.remove(&key);
        self.children.remove(&key)
    }

    pub(crate) fn reserve_child_start(
        &mut self,
        language_id: &str,
        root_uri: &str,
        semantic_only: bool,
    ) -> Option<ChildStartup> {
        if *self.stopped.borrow() {
            return None;
        }
        let key = key(language_id, root_uri);
        self.starting.retain(|_, token| token.strong_count() > 0);
        if self.children.contains_key(&key) || self.starting.contains_key(&key) {
            return None;
        }
        let token = Arc::new(());
        self.starting.insert(key.clone(), Arc::downgrade(&token));
        Some(ChildStartup {
            key,
            token,
            notification_tx: self.notification_tx.clone(),
            stopped: self.stopped.subscribe(),
            semantic_only,
        })
    }

    pub(crate) fn publish_child(&mut self, started: StartedChild) -> Option<ChildLspHandle> {
        let StartedChild { startup, child, initialized } = started;
        let ChildStartup { key, token, .. } = startup;
        let current = self.starting.get(&key).and_then(Weak::upgrade);
        if *self.stopped.borrow()
            || !current.is_some_and(|expected| Arc::ptr_eq(&token, &expected))
            || self.children.contains_key(&key)
        {
            return None;
        }
        self.starting.remove(&key);
        self.install_child(key, child.clone(), &initialized);
        Some(child)
    }

    fn install_child(&mut self, key: (String, String), child: ChildLspHandle, initialized: &serde_json::Value) {
        self.capabilities.insert(
            key.clone(),
            initialized.get("capabilities").cloned().unwrap_or(serde_json::Value::Null),
        );
        if let Some(legend) = semantic_legend(initialized) {
            self.legends.insert(key.clone(), legend);
        }
        self.children.insert(key, child);
    }

    /// Get an existing child LSP handle without spawning.
    ///
    /// Returns `Err` if no child exists for the given language and root.
    pub fn get_child(
        &self,
        language_id: &str,
        root_uri: &str,
    ) -> Result<&ChildLspHandle, ChildLspError> {
        self.children
            .get(&key(language_id, root_uri))
            .ok_or_else(|| ChildLspError::UnknownLanguage {
                language_id: language_id.to_string(),
            })
    }

    /// The semantic-token legend a child declared, if it supports them.
    pub fn token_legend(
        &self,
        language_id: &str,
        root_uri: &str,
    ) -> Option<(Vec<String>, Vec<String>)> {
        self.legends.get(&key(language_id, root_uri)).cloned()
    }

    /// Shut down all child LSP servers.
    pub async fn refresh_environments(&self) {
        for ((_, root), handle) in &self.children {
            let _ = handle.refresh_environment(root).await;
        }
    }

    /// Shut down all child LSP servers.
    pub async fn shutdown_all(&mut self) {
        self.stopped.send_replace(true);
        self.starting.clear();
        self.capabilities.clear();
        self.legends.clear();
        for ((language_id, root_uri), handle) in self.children.drain() {
            if let Err(e) = handle.shutdown().await {
                tracing::warn!(
                    language_id,
                    root_uri,
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

    #[tokio::test]
    async fn abandoned_start_unblocks_its_slot_and_shutdown_refuses_late_processes() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let mut dispatcher = Dispatcher::new(tx);
        let language = "fixture-never-launch";
        let root = "file:///private/a/";
        let pending = dispatcher.reserve_child_start(language, root, false).unwrap();
        assert!(dispatcher.reserve_child_start(language, root, false).is_none());
        assert!(matches!(dispatcher.get_or_spawn(language, root).await, Err(ChildLspError::ProviderStarting)));
        let other = dispatcher.reserve_child_start(language, "file:///private/b/", false).unwrap();
        drop(pending);

        let replacement = dispatcher.reserve_child_start(language, root, false).unwrap();
        dispatcher.shutdown_all().await;
        assert!(dispatcher.reserve_child_start(language, root, false).is_none());
        assert!(matches!(replacement.start().await, Err(ChildLspError::DispatcherStopped)));
        assert!(matches!(other.start().await, Err(ChildLspError::DispatcherStopped)));
        assert!(matches!(dispatcher.get_or_spawn(language, root).await, Err(ChildLspError::DispatcherStopped)));
    }

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

impl Dispatcher {
    pub fn server_capabilities(&self, language: &str, root: &str) -> Option<&serde_json::Value> {
        self.capabilities.get(&key(language, root))
    }
}
