use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{Mutex, RwLock, mpsc};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::child_lsp::{ChildNotification, lsp_command};
use crate::dispatcher::Dispatcher;
use crate::document::HickDocumentState;
use crate::position_map::PositionMap;

/// Accumulated child diagnostics: hick document URI -> (virtual file URI -> diagnostics).
type ChildDiagnosticsStore = Arc<RwLock<HashMap<Url, HashMap<Url, Vec<Diagnostic>>>>>;

pub struct HickBackend {
    client: Client,
    /// Open document states, keyed by URI.
    documents: Arc<RwLock<HashMap<Url, DocEntry>>>,
    /// Dispatcher for child LSP servers.
    dispatcher: Arc<Mutex<Dispatcher>>,
    /// Receiver for notifications from child LSPs.
    notification_rx: Arc<Mutex<mpsc::UnboundedReceiver<ChildNotification>>>,
    /// Mapping from virtual file URI -> (hick document URI, PositionMap).
    vfile_index: Arc<RwLock<HashMap<Url, VFileMapping>>>,
    /// Accumulated child diagnostics keyed by (hick_uri, vfile_uri).
    /// When any child sends diagnostics, we update its entry and republish
    /// the merged set for the hick document, avoiding later children
    /// overwriting earlier children's diagnostics.
    child_diagnostics: ChildDiagnosticsStore,
}

struct DocEntry {
    /// The parsed document state (None if parse failed).
    state: Option<HickDocumentState>,
    /// Virtual file URIs that are currently "open" in child LSPs.
    open_vfiles: Vec<Url>,
    /// Version counter for virtual files sent to children.
    vfile_version: i32,
}

/// Maps a virtual file URI back to the source .hick document.
struct VFileMapping {
    /// The .hick document URI.
    hick_uri: Url,
    /// The language ID of this virtual file (for routing to child LSPs).
    language_id: String,
    /// Position map for translating coordinates.
    position_map: PositionMap,
}

impl HickBackend {
    pub fn new(client: Client) -> Self {
        let (notification_tx, notification_rx) = mpsc::unbounded_channel();

        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
            dispatcher: Arc::new(Mutex::new(Dispatcher::new(notification_tx))),
            notification_rx: Arc::new(Mutex::new(notification_rx)),
            vfile_index: Arc::new(RwLock::new(HashMap::new())),
            child_diagnostics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Derive a stable virtual file URI from the .hick document URI and virtual
    /// file path, preserving directory structure so that tools like
    /// rust-analyzer can resolve paths from Cargo.toml correctly.
    fn vfile_uri(hick_uri: &Url, vfile_path: &str) -> Url {
        let hash = simple_hash(hick_uri.as_str());
        Url::parse(&format!("file:///tmp/hick-lsp-vfiles/{hash}/{vfile_path}"))
            .unwrap_or_else(|_| hick_uri.clone())
    }

    /// Process a document change: parse, generate virtual files, forward to
    /// child LSPs.
    async fn process_document(&self, hick_uri: &Url, source: &str) {
        tracing::debug!(%hick_uri, source_len = source.len(), "process_document called");

        // 1. Try to parse the hick document.
        let state = match HickDocumentState::from_source(source) {
            Ok(s) => {
                tracing::debug!(
                    vfile_count = s.virtual_files.len(),
                    "parsed hick document successfully"
                );
                s
            }
            Err(e) => {
                let (line, message) = parse_error_location(&e);
                let lsp_line = if line > 0 { line - 1 } else { 0 } as u32;
                self.client
                    .publish_diagnostics(
                        hick_uri.clone(),
                        vec![Diagnostic {
                            range: Range {
                                start: Position {
                                    line: lsp_line,
                                    character: 0,
                                },
                                end: Position {
                                    line: lsp_line,
                                    character: u32::MAX,
                                },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("hick-lsp".to_string()),
                            message,
                            ..Default::default()
                        }],
                        None,
                    )
                    .await;

                self.close_vfiles_for(hick_uri, &[]).await;

                let mut docs = self.documents.write().await;
                if let Some(entry) = docs.get_mut(hick_uri) {
                    entry.state = None;
                }
                return;
            }
        };

        // 2. Compute the new set of virtual file URIs so we can prune stale
        //    accumulator entries without clearing diagnostics that will be
        //    refreshed momentarily.
        let new_vfile_uris: Vec<Url> = state
            .virtual_files
            .iter()
            .map(|vf| Self::vfile_uri(hick_uri, &vf.path))
            .collect();

        // 3. Close old virtual files, pruning diagnostics for removed files.
        self.close_vfiles_for(hick_uri, &new_vfile_uris).await;

        // 4. Determine root URI for child LSP initialization.
        //
        // We use the virtual file directory so that child LSPs like
        // rust-analyzer don't discover unrelated projects in the .hick
        // file's parent directory.
        let hash = simple_hash(hick_uri.as_str());
        let root_uri = format!("file:///tmp/hick-lsp-vfiles/{hash}/");

        let vfile_version = {
            let docs = self.documents.read().await;
            docs.get(hick_uri).map(|e| e.vfile_version + 1).unwrap_or(1)
        };

        // 5. Write all virtual files to disk so that tools like
        //    rust-analyzer can discover project structure (e.g., Cargo.toml).
        for vf in &state.virtual_files {
            let vf_uri = Self::vfile_uri(hick_uri, &vf.path);
            if let Ok(path) = vf_uri.to_file_path() {
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(&path, vf.content());
            }
        }

        // 6. For each virtual file with a known language, forward to child LSP.
        let mut new_open_vfiles = Vec::new();

        for vf in &state.virtual_files {
            tracing::debug!(path = %vf.path, language_id = ?vf.language_id, "processing virtual file");

            let lang_id = match vf.language_id {
                Some(id) => id,
                None => continue,
            };

            // Skip languages where we don't have an LSP command.
            if lsp_command(lang_id).is_err() {
                tracing::debug!(lang_id, "no LSP command for language, skipping");
                continue;
            }

            let vf_uri = Self::vfile_uri(hick_uri, &vf.path);
            let content = vf.content();
            tracing::debug!(%vf_uri, content_len = content.len(), "forwarding to child LSP");
            let position_map = PositionMap::build(&vf.segments);

            // Store the mapping.
            {
                let mut index = self.vfile_index.write().await;
                index.insert(
                    vf_uri.clone(),
                    VFileMapping {
                        hick_uri: hick_uri.clone(),
                        language_id: lang_id.to_string(),
                        position_map,
                    },
                );
            }

            // Send to child LSP.
            tracing::debug!(lang_id, %root_uri, "acquiring dispatcher lock for child LSP");
            let mut dispatcher = self.dispatcher.lock().await;
            tracing::debug!(lang_id, "dispatching to child LSP");
            match dispatcher.get_or_spawn(lang_id, &root_uri).await {
                Ok(handle) => {
                    tracing::debug!(lang_id, "child LSP ready, sending didOpen");
                    let result = handle
                        .notify(
                            "textDocument/didOpen",
                            serde_json::json!({
                                "textDocument": {
                                    "uri": vf_uri.as_str(),
                                    "languageId": lang_id,
                                    "version": vfile_version,
                                    "text": content,
                                }
                            }),
                        )
                        .await;

                    if let Err(e) = result {
                        tracing::warn!(
                            language_id = lang_id,
                            path = %vf.path,
                            error = %e,
                            "failed to send didOpen to child LSP"
                        );
                    }

                    new_open_vfiles.push(vf_uri);
                }
                Err(e) => {
                    tracing::warn!(
                        language_id = lang_id,
                        error = %e,
                        "failed to spawn child LSP"
                    );
                }
            }
        }

        // 6. Update stored state.
        {
            let mut docs = self.documents.write().await;
            let entry = docs.entry(hick_uri.clone()).or_insert_with(|| DocEntry {
                state: None,
                open_vfiles: Vec::new(),
                vfile_version: 0,
            });
            entry.state = Some(state);
            entry.open_vfiles = new_open_vfiles;
            entry.vfile_version = vfile_version;
        }
    }

    /// Close all virtual files associated with a .hick document.
    ///
    /// Sends `textDocument/didClose` to each child LSP so that re-opening the
    /// same URI later is not a protocol violation. Only prunes accumulated
    /// diagnostics for virtual files that are no longer in `new_vfile_uris`,
    /// keeping existing diagnostics for files that will be refreshed.
    async fn close_vfiles_for(&self, hick_uri: &Url, new_vfile_uris: &[Url]) {
        let old_vfiles = {
            let docs = self.documents.read().await;
            match docs.get(hick_uri) {
                Some(entry) => entry.open_vfiles.clone(),
                None => return,
            }
        };

        // Collect (uri, language_id) pairs before sending didClose.
        let to_close: Vec<(Url, String)> = {
            let mut index = self.vfile_index.write().await;
            old_vfiles
                .iter()
                .filter_map(|vf_uri| {
                    index
                        .remove(vf_uri)
                        .map(|m| (vf_uri.clone(), m.language_id))
                })
                .collect()
        };

        // Remove accumulated diagnostics only for virtual files that no
        // longer exist, so that diagnostics for surviving files persist
        // until the child LSP sends fresh ones.
        {
            let mut diags = self.child_diagnostics.write().await;
            if let Some(per_hick) = diags.get_mut(hick_uri) {
                per_hick.retain(|vf_uri, _| new_vfile_uris.contains(vf_uri));
            }
        }

        // Republish the remaining accumulated diagnostics (minus removed files).
        let remaining = {
            let store = self.child_diagnostics.read().await;
            store
                .get(hick_uri)
                .map(|per_hick| per_hick.values().flatten().cloned().collect::<Vec<_>>())
                .unwrap_or_default()
        };
        self.client
            .publish_diagnostics(hick_uri.clone(), remaining, None)
            .await;

        let dispatcher = self.dispatcher.lock().await;
        for (vf_uri, language_id) in &to_close {
            if let Ok(handle) = dispatcher.get_child(language_id) {
                let _ = handle
                    .notify(
                        "textDocument/didClose",
                        serde_json::json!({
                            "textDocument": { "uri": vf_uri.as_str() }
                        }),
                    )
                    .await;
            }
        }
    }

    /// Start a background task that drains child LSP notifications and
    /// translates diagnostics back to .hick coordinates.
    fn spawn_notification_handler(&self) {
        let client = self.client.clone();
        let notification_rx = Arc::clone(&self.notification_rx);
        let vfile_index = Arc::clone(&self.vfile_index);
        let child_diagnostics = Arc::clone(&self.child_diagnostics);

        tokio::spawn(async move {
            tracing::debug!("notification handler started");
            let mut rx = notification_rx.lock().await;
            while let Some(notif) = rx.recv().await {
                tracing::debug!(
                    method = %notif.method,
                    language_id = %notif.language_id,
                    "received child LSP notification"
                );
                if notif.method == "textDocument/publishDiagnostics"
                    && let Err(e) =
                        handle_child_diagnostics(&client, &vfile_index, &child_diagnostics, &notif)
                            .await
                {
                    tracing::debug!(
                        error = %e,
                        "failed to process child diagnostics"
                    );
                }
            }
            tracing::debug!("notification handler exited");
        });
    }
}

/// Process a publishDiagnostics notification from a child LSP.
///
/// Instead of publishing each child's diagnostics independently (which would
/// cause later children to overwrite earlier ones), we accumulate diagnostics
/// per virtual file and publish the merged set for the whole .hick document.
async fn handle_child_diagnostics(
    client: &Client,
    vfile_index: &Arc<RwLock<HashMap<Url, VFileMapping>>>,
    accumulated: &ChildDiagnosticsStore,
    notif: &ChildNotification,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let uri_str = notif
        .params
        .get("uri")
        .and_then(|u| u.as_str())
        .ok_or("missing uri in publishDiagnostics")?;
    let vf_uri = Url::parse(uri_str)?;

    let index = vfile_index.read().await;
    let mapping = match index.get(&vf_uri) {
        Some(m) => m,
        None => {
            let diag_count = notif
                .params
                .get("diagnostics")
                .and_then(|d| d.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            tracing::debug!(
                %vf_uri,
                diag_count,
                language_id = %notif.language_id,
                "no vfile mapping for URI, dropping diagnostics"
            );
            return Ok(());
        }
    };

    let hick_uri = mapping.hick_uri.clone();
    let position_map = &mapping.position_map;

    let child_diagnostics = notif
        .params
        .get("diagnostics")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();

    let mut translated = Vec::new();
    for diag_val in &child_diagnostics {
        let range = match diag_val.get("range") {
            Some(r) => r,
            None => continue,
        };

        let start_line = range
            .pointer("/start/line")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let start_char = range
            .pointer("/start/character")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let end_line = range
            .pointer("/end/line")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let end_char = range
            .pointer("/end/character")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        // Translate from virtual file coordinates to .hick source coordinates.
        let (src_start_line, src_start_char) = position_map
            .to_source(start_line, start_char)
            .unwrap_or((start_line, start_char));
        let (src_end_line, src_end_char) = position_map
            .to_source(end_line, end_char)
            .unwrap_or((end_line, end_char));

        let severity = diag_val
            .get("severity")
            .and_then(|s| s.as_u64())
            .and_then(|s| match s {
                1 => Some(DiagnosticSeverity::ERROR),
                2 => Some(DiagnosticSeverity::WARNING),
                3 => Some(DiagnosticSeverity::INFORMATION),
                4 => Some(DiagnosticSeverity::HINT),
                _ => None,
            });

        let message = diag_val
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("")
            .to_string();

        let source = diag_val
            .get("source")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        translated.push(Diagnostic {
            range: Range {
                start: Position {
                    line: src_start_line,
                    character: src_start_char,
                },
                end: Position {
                    line: src_end_line,
                    character: src_end_char,
                },
            },
            severity,
            source,
            message,
            ..Default::default()
        });
    }

    drop(index);

    // Update this virtual file's diagnostics and publish the merged set.
    let merged = {
        let mut store = accumulated.write().await;
        let per_hick = store.entry(hick_uri.clone()).or_default();
        per_hick.insert(vf_uri, translated);
        per_hick.values().flatten().cloned().collect::<Vec<_>>()
    };

    client.publish_diagnostics(hick_uri, merged, None).await;

    Ok(())
}

#[tower_lsp::async_trait]
impl LanguageServer for HickBackend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions::default()),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("hick-lsp initialized");
        self.client
            .log_message(MessageType::INFO, "hick-lsp initialized")
            .await;

        self.spawn_notification_handler();
    }

    async fn shutdown(&self) -> Result<()> {
        let mut dispatcher = self.dispatcher.lock().await;
        dispatcher.shutdown_all().await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let source = params.text_document.text;

        {
            let mut docs = self.documents.write().await;
            docs.entry(uri.clone()).or_insert_with(|| DocEntry {
                state: None,
                open_vfiles: Vec::new(),
                vfile_version: 0,
            });
        }

        self.process_document(&uri, &source).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        if let Some(change) = params.content_changes.into_iter().next() {
            self.process_document(&uri, &change.text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        self.close_vfiles_for(&uri, &[]).await;

        let mut docs = self.documents.write().await;
        docs.remove(&uri);
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(None)
    }

    async fn hover(&self, _params: HoverParams) -> Result<Option<Hover>> {
        Ok(None)
    }

    async fn goto_definition(
        &self,
        _params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        Ok(None)
    }
}

/// Extract a line number and user-friendly message from a [`hick_lang::ParseError`].
fn parse_error_location(err: &hick_lang::ParseError) -> (usize, String) {
    match err {
        hick_lang::ParseError::Syntax { line, message } => (*line, message.clone()),
        hick_lang::ParseError::MissingRoot => (1, err.to_string()),
        hick_lang::ParseError::UnclosedTag { line, .. } => (*line, err.to_string()),
        hick_lang::ParseError::UnexpectedClose { line, .. } => (*line, err.to_string()),
        hick_lang::ParseError::UnclosedComment { line } => (*line, err.to_string()),
    }
}

fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}
