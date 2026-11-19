<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Meta-LSP Forwards Whatever The Child Supports

For: Authors using completion, diagnostics and refactoring inside literate source.

Given an editor pointed at `hick-lsp` for `*.md`, when it makes any request
a mainstream language server answers, then `hick-lsp` forwards it to the
server that owns that block and translates the answer back into document
coordinates. The document is the buffer being edited, so every position in
every reply is a position in the `.md` file — never in the virtual file the
child actually saw.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/the-meta-lsp-forwards-what-the-child-supports.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-cd294aeb3b8e"></a>
## `crates/hick-lsp/src/dispatcher.rs`

Related contracts: [The Meta-LSP Forwards Whatever The Child Supports](contract.md).

<a id="f-cd294aeb3b8e-Dispatcher"></a>
### `Dispatcher`

Request routing for child LSP servers.
The [`Dispatcher`] manages a pool of child LSP processes, one per language
**per root**, and provides access to them by language ID and root URI.
Per root, not per language, because a project-aware server answers for the
workspace it was initialised against and no other: rust-analyzer started
on one document's staging directory does not own `src/main.rs` in the
repository the app has open, and a plain file handed to it there would
get syntax-only answers that read like a forwarding bug.

<storycode:copy id="f-cd294aeb3b8e-Dispatcher">
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

</storycode:copy>
<a id="f-cd294aeb3b8e-key"></a>
### `key`

<storycode:copy id="f-cd294aeb3b8e-key">
fn key(language_id: &str, root_uri: &str) -> (String, String) {
    (language_id.to_string(), root_uri.to_string())
}

</storycode:copy>
<a id="f-cd294aeb3b8e-impl-Dispatcher"></a>
### `impl-Dispatcher`

A saved-module refresh can replace a single private child. Detach its handle
and metadata under the dispatcher lock, then retire it without blocking requests
to other roots. A startup reserves only its own language and root while the child
initializes outside the lock. Its identity must still match when the dispatcher
publishes the new handle. A dropped reservation releases the slot, and terminal
shutdown stops waiting startups as well as retiring published children.

<storycode:copy id="f-cd294aeb3b8e-impl-Dispatcher">
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

</storycode:copy>
<a id="f-cd294aeb3b8e-semantic_legend"></a>
### `semantic_legend`

Pull `capabilities.semanticTokensProvider.legend` out of an initialize
result, whichever of the two shapes the server used.

<storycode:copy id="f-cd294aeb3b8e-semantic_legend">
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

</storycode:copy>
<a id="f-cd294aeb3b8e-tests"></a>
### `tests`

Keep a replacement pending while another root acquires a reservation. The
abandoned replacement must release its own slot. Shut down before starting the
next reservation to verify that no late request can launch its process.

<storycode:copy id="f-cd294aeb3b8e-tests">
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
</storycode:copy>

### Assemble `crates/hick-lsp/src/dispatcher.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

## Retain the actual child capability declaration

Retain each child initialization declaration under its language and root. Optional resolution is offered only through a child that declared it; one provider cannot donate its capability or opaque state to another. Clear these declarations when those child processes shut down.

<storycode:copy id="provider-child-capability-read">

impl Dispatcher {
    pub fn server_capabilities(&self, language: &str, root: &str) -> Option<&serde_json::Value> {
        self.capabilities.get(&key(language, root))
    }
}
</storycode:copy>

<storycode:file path="crates/hick-lsp/src/dispatcher.rs"><storycode:paste select="#f-cd294aeb3b8e-Dispatcher"/><storycode:paste select="#f-cd294aeb3b8e-key"/><storycode:paste select="#f-cd294aeb3b8e-impl-Dispatcher"/><storycode:paste select="#f-cd294aeb3b8e-semantic_legend"/><storycode:paste select="#f-cd294aeb3b8e-tests"/><storycode:paste select="#provider-child-capability-read"/></storycode:file>

<a id="output-3d1198a4d2c1"></a>
## `crates/hick-lsp/src/semantic.rs`

Related contracts: [The Meta-LSP Forwards Whatever The Child Supports](contract.md).

<a id="f-3d1198a4d2c1-Token"></a>
### `Token`

Semantic tokens across virtual files.
This is the feature that makes a `hick:file` block look like code rather
than text, and it is the one that cannot be forwarded by rewriting a range
or two. LSP encodes semantic tokens as a flat array of five-number groups,
each *relative to the token before it*:
\```text
[ Δline, Δstart, length, tokenType, tokenModifiers, … ]
\```
Deltas are meaningless once the tokens move. A document interleaves
several virtual files with prose between them, so mapping each token into
document coordinates reorders them — the tokens of the second block land
after the first block's, but their deltas were computed against their own
file's origin. So this decodes to absolute positions, maps each one, sorts
by where it now is, and re-encodes.
Legends are per server. Two children (pyright and rust-analyzer, say) can
disagree about which integer means `function`, so a merged result has to
be re-indexed against one legend the editor was told about — otherwise the
Rust in a document is coloured with Python's palette.

<storycode:copy id="f-3d1198a4d2c1-Token">
//! Semantic tokens across virtual files.
//!
//! This is the feature that makes a `hick:file` block look like code rather
//! than text, and it is the one that cannot be forwarded by rewriting a range
//! or two. LSP encodes semantic tokens as a flat array of five-number groups,
//! each *relative to the token before it*:
//!
//! ```text
//! [ Δline, Δstart, length, tokenType, tokenModifiers, … ]
//! ```
//!
//! Deltas are meaningless once the tokens move. A document interleaves
//! several virtual files with prose between them, so mapping each token into
//! document coordinates reorders them — the tokens of the second block land
//! after the first block's, but their deltas were computed against their own
//! file's origin. So this decodes to absolute positions, maps each one, sorts
//! by where it now is, and re-encodes.
//!
//! Legends are per server. Two children (pyright and rust-analyzer, say) can
//! disagree about which integer means `function`, so a merged result has to
//! be re-indexed against one legend the editor was told about — otherwise the
//! Rust in a document is coloured with Python's palette.

use std::collections::HashMap;

use crate::position_map::PositionMap;

/// One token, in absolute coordinates, with its type named rather than
/// numbered — names are the only thing two servers agree on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub line: u32,
    pub start: u32,
    pub length: u32,
    pub token_type: String,
    pub modifiers: Vec<String>,
}

</storycode:copy>
<a id="f-3d1198a4d2c1-decode"></a>
### `decode`

Decode a server's `data` array using the legend it declared.

<storycode:copy id="f-3d1198a4d2c1-decode">
/// Decode a server's `data` array using the legend it declared.
pub fn decode(data: &[u32], types: &[String], modifiers: &[String]) -> Vec<Token> {
    let mut out = Vec::new();
    let mut line = 0u32;
    let mut start = 0u32;
    for chunk in data.chunks_exact(5) {
        let (d_line, d_start, length, type_index, modifier_bits) =
            (chunk[0], chunk[1], chunk[2], chunk[3], chunk[4]);
        // A non-zero line delta resets the column, exactly as the encoding
        // says; getting this wrong shifts every token on the line.
        if d_line > 0 {
            line += d_line;
            start = d_start;
        } else {
            start += d_start;
        }
        let token_type = types
            .get(type_index as usize)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let mut names = Vec::new();
        for (bit, name) in modifiers.iter().enumerate() {
            if modifier_bits & (1 << bit) != 0 {
                names.push(name.clone());
            }
        }
        out.push(Token {
            line,
            start,
            length,
            token_type,
            modifiers: names,
        });
    }
    out
}

</storycode:copy>
<a id="f-3d1198a4d2c1-to_source"></a>
### `to_source`

Map tokens from one virtual file into document coordinates.
A token whose position has no source — text the weaver produced, not the
author — is dropped rather than guessed at. Colouring bytes nobody wrote
would be colouring a fiction.

<storycode:copy id="f-3d1198a4d2c1-to_source">
/// Map tokens from one virtual file into document coordinates.
///
/// A token whose position has no source — text the weaver produced, not the
/// author — is dropped rather than guessed at. Colouring bytes nobody wrote
/// would be colouring a fiction.
pub fn to_source(tokens: Vec<Token>, map: &PositionMap) -> Vec<Token> {
    tokens
        .into_iter()
        .filter_map(|token| {
            let (line, start) = map.to_source(token.line, token.start)?;
            Some(Token {
                line,
                start,
                ..token
            })
        })
        .collect()
}

</storycode:copy>
<a id="f-3d1198a4d2c1-encode"></a>
### `encode`

Re-encode absolute tokens against `legend`, in document order.

<storycode:copy id="f-3d1198a4d2c1-encode">
/// Re-encode absolute tokens against `legend`, in document order.
pub fn encode(
    mut tokens: Vec<Token>,
    legend_types: &[String],
    legend_modifiers: &[String],
) -> Vec<u32> {
    // Sort first: tokens arrive per virtual file, and a document interleaves
    // those files with prose. Encoding out of order produces negative deltas,
    // which the protocol cannot express and editors render as garbage.
    tokens.sort_by(|a, b| a.line.cmp(&b.line).then(a.start.cmp(&b.start)));

    let type_index: HashMap<&str, u32> = legend_types
        .iter()
        .enumerate()
        .map(|(i, name)| (name.as_str(), i as u32))
        .collect();
    let modifier_index: HashMap<&str, u32> = legend_modifiers
        .iter()
        .enumerate()
        .map(|(i, name)| (name.as_str(), i as u32))
        .collect();

    let mut out = Vec::with_capacity(tokens.len() * 5);
    let mut last_line = 0u32;
    let mut last_start = 0u32;
    for token in tokens {
        // A type this build's legend does not carry is dropped: emitting an
        // index the editor was not told about colours it as something else.
        let Some(type_id) = type_index.get(token.token_type.as_str()).copied() else {
            continue;
        };
        let mut bits = 0u32;
        for modifier in &token.modifiers {
            if let Some(bit) = modifier_index.get(modifier.as_str()) {
                bits |= 1 << bit;
            }
        }
        let d_line = token.line - last_line;
        let d_start = if d_line == 0 {
            token.start.saturating_sub(last_start)
        } else {
            token.start
        };
        out.extend_from_slice(&[d_line, d_start, token.length, type_id, bits]);
        last_line = token.line;
        last_start = token.start;
    }
    out
}

</storycode:copy>
<a id="f-3d1198a4d2c1-legend_types"></a>
### `legend_types`

The legend this server advertises: the union of what the children use, in
a fixed order so a client can rely on it across requests.

<storycode:copy id="f-3d1198a4d2c1-legend_types">
/// The legend this server advertises: the union of what the children use, in
/// a fixed order so a client can rely on it across requests.
pub fn legend_types() -> Vec<String> {
    [
        "namespace",
        "type",
        "class",
        "enum",
        "interface",
        "struct",
        "typeParameter",
        "parameter",
        "variable",
        "property",
        "enumMember",
        "event",
        "function",
        "method",
        "macro",
        "keyword",
        "modifier",
        "comment",
        "string",
        "number",
        "regexp",
        "operator",
        "decorator",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

</storycode:copy>
<a id="f-3d1198a4d2c1-legend_modifiers"></a>
### `legend_modifiers`

<storycode:copy id="f-3d1198a4d2c1-legend_modifiers">
pub fn legend_modifiers() -> Vec<String> {
    [
        "declaration",
        "definition",
        "readonly",
        "static",
        "deprecated",
        "abstract",
        "async",
        "modification",
        "documentation",
        "defaultLibrary",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

</storycode:copy>
<a id="f-3d1198a4d2c1-tests"></a>
### `tests`

<storycode:copy id="f-3d1198a4d2c1-tests">
#[cfg(test)]
mod tests {
    use super::*;
    use crate::virtual_file::VirtualSegment;

    fn types() -> Vec<String> {
        vec!["function".into(), "variable".into(), "keyword".into()]
    }
    fn modifiers() -> Vec<String> {
        vec!["declaration".into(), "readonly".into()]
    }

    #[test]
    fn decoding_follows_the_delta_encoding() {
        // Two tokens on line 0, one on line 2. The second token's column is
        // relative to the first; the third resets because its line moved.
        let data = vec![0, 5, 3, 0, 0, 0, 4, 2, 1, 1, 2, 1, 6, 2, 0];
        let tokens = decode(&data, &types(), &modifiers());
        assert_eq!(tokens.len(), 3);
        assert_eq!((tokens[0].line, tokens[0].start), (0, 5));
        assert_eq!((tokens[1].line, tokens[1].start), (0, 9));
        assert_eq!((tokens[2].line, tokens[2].start), (2, 1));
        assert_eq!(tokens[0].token_type, "function");
        assert_eq!(tokens[1].modifiers, vec!["declaration".to_string()]);
    }

    #[test]
    fn encoding_is_the_inverse_of_decoding() {
        let data = vec![0, 5, 3, 0, 0, 0, 4, 2, 1, 1, 2, 1, 6, 2, 0];
        let tokens = decode(&data, &types(), &modifiers());
        assert_eq!(encode(tokens, &types(), &modifiers()), data);
    }

    #[test]
    fn tokens_from_two_blocks_are_re_sorted_before_encoding() {
        // The failure this prevents: block two's tokens map above block
        // one's, and encoding them in arrival order needs a negative delta,
        // which the protocol cannot express.
        let tokens = vec![
            Token {
                line: 40,
                start: 0,
                length: 3,
                token_type: "function".into(),
                modifiers: vec![],
            },
            Token {
                line: 10,
                start: 2,
                length: 4,
                token_type: "variable".into(),
                modifiers: vec![],
            },
        ];
        let data = encode(tokens, &types(), &modifiers());
        assert_eq!(data[0], 10, "the earliest token comes first");
        // Every line delta is non-negative by construction.
        for chunk in data.chunks_exact(5) {
            assert!(chunk[0] < 1_000_000, "line delta wrapped: {chunk:?}");
        }
    }

    #[test]
    fn a_token_the_legend_does_not_carry_is_dropped_not_mis_coloured() {
        let tokens = vec![Token {
            line: 0,
            start: 0,
            length: 1,
            token_type: "somethingNobodyElseHas".into(),
            modifiers: vec![],
        }];
        assert!(encode(tokens, &types(), &modifiers()).is_empty());
    }

    #[test]
    fn tokens_land_where_the_document_has_them() {
        // A block whose text starts on document line 11 (the tag is on 10):
        // a token on the block's first line must come back on line 11.
        let segments = vec![VirtualSegment {
            text: "def f():\n    return 1\n".to_string(),
            source_span: None,
            source_line: 11,
            source_column: 0,
        }];
        let map = PositionMap::build(&segments);
        let mapped = to_source(
            vec![Token {
                line: 0,
                start: 4,
                length: 2,
                token_type: "function".into(),
                modifiers: vec![],
            }],
            &map,
        );
        assert_eq!(mapped.len(), 1, "the token maps into the document");
        assert!(mapped[0].line >= 10, "landed at {}", mapped[0].line);
    }

    #[test]
    fn a_token_with_no_source_is_dropped() {
        // Bytes the weaver produced are nobody's code; colouring them would
        // colour a fiction.
        let map = PositionMap::build(&[]);
        assert!(
            to_source(
                vec![Token {
                    line: 3,
                    start: 0,
                    length: 1,
                    token_type: "function".into(),
                    modifiers: vec![]
                }],
                &map
            )
            .is_empty()
        );
    }

    #[test]
    fn the_advertised_legend_covers_what_servers_send() {
        // If the legend is missing a common type, every token of that type is
        // silently dropped — so this is worth pinning.
        let types = legend_types();
        for expected in [
            "function",
            "variable",
            "parameter",
            "property",
            "keyword",
            "string",
            "comment",
        ] {
            assert!(
                types.contains(&expected.to_string()),
                "legend lacks {expected}"
            );
        }
        assert!(legend_modifiers().contains(&"declaration".to_string()));
    }
}
</storycode:copy>

### Assemble `crates/hick-lsp/src/semantic.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lsp/src/semantic.rs"><storycode:paste select="#f-3d1198a4d2c1-Token"/><storycode:paste select="#f-3d1198a4d2c1-decode"/><storycode:paste select="#f-3d1198a4d2c1-to_source"/><storycode:paste select="#f-3d1198a4d2c1-encode"/><storycode:paste select="#f-3d1198a4d2c1-legend_types"/><storycode:paste select="#f-3d1198a4d2c1-legend_modifiers"/><storycode:paste select="#f-3d1198a4d2c1-tests"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-lsp/src/backend.rs](../../../mechanisms/crates/hick-lsp/src/backend.rs.md#output-c936496a03c1)
- [crates/hick-lsp/src/child_lsp.rs](../../../mechanisms/crates/hick-lsp/src/child_lsp.rs.md#output-ebc4fbd32a25)
- [crates/hick-lsp/src/dispatcher.rs](contract.md#output-cd294aeb3b8e)
- [crates/hick-lsp/src/semantic.rs](contract.md#output-3d1198a4d2c1)

### Tests and fixtures

- [crates/hickory-cli/tests/lsp_desktop.rs](../../../mechanisms/crates/hickory-cli/tests/lsp_desktop.rs.md#output-c14f2fec35ca)
- [crates/hickory-cli/tests/lsp_languages.rs](../../../mechanisms/crates/hickory-cli/tests/lsp_languages.rs.md#output-7c4650d33fc8)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
