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

    /// Find the virtual file (and translated position) for a position in a
    /// .hick document, if any virtual file covers that source line.
    async fn virtual_target(
        &self,
        hick_uri: &Url,
        pos: Position,
    ) -> Option<(Url, String, Position)> {
        let index = self.vfile_index.read().await;
        for (vf_uri, mapping) in index.iter() {
            if &mapping.hick_uri == hick_uri
                && let Some((vl, vc)) = mapping.position_map.to_virtual(pos.line, pos.character)
            {
                return Some((
                    vf_uri.clone(),
                    mapping.language_id.clone(),
                    Position::new(vl, vc),
                ));
            }
        }
        None
    }

    /// Send a positional request to the child LSP owning `vf_uri`.
    ///
    /// Best-effort: a missing/failed child (e.g. pyright not installed)
    /// returns `None`, never an error — callers degrade to structural answers.
    async fn child_request(
        &self,
        method: &str,
        vf_uri: &Url,
        language_id: &str,
        pos: Position,
        extra: Option<serde_json::Value>,
    ) -> Option<serde_json::Value> {
        let handle = {
            let dispatcher = self.dispatcher.lock().await;
            dispatcher.get_child(language_id).ok()?.clone()
        };
        let mut params = serde_json::json!({
            "textDocument": { "uri": vf_uri.as_str() },
            "position": { "line": pos.line, "character": pos.character },
        });
        if let Some(serde_json::Value::Object(extra)) = extra
            && let Some(obj) = params.as_object_mut()
        {
            for (k, v) in extra {
                obj.insert(k, v);
            }
        }
        match handle.request(method, params).await {
            Ok(v) if !v.is_null() => Some(v),
            Ok(_) => None,
            Err(e) => {
                tracing::debug!(method, language_id, error = %e, "child LSP request failed");
                None
            }
        }
    }

    /// Snapshot of the vfile index for pure result translation.
    async fn index_snapshot(&self) -> HashMap<String, (Url, PositionMap)> {
        let index = self.vfile_index.read().await;
        index
            .iter()
            .map(|(uri, m)| {
                (
                    uri.as_str().to_string(),
                    (m.hick_uri.clone(), m.position_map.clone()),
                )
            })
            .collect()
    }

    /// Structural (copy/paste) locations for a request position.
    async fn structural_spans(
        &self,
        hick_uri: &Url,
        pos: Position,
        kind: StructuralKind,
    ) -> Vec<Location> {
        let docs = self.documents.read().await;
        let Some(state) = docs.get(hick_uri).and_then(|e| e.state.as_ref()) else {
            return Vec::new();
        };
        let source = &state.doc.source;
        let Some(offset) = crate::structural::position_to_byte(source, pos.line, pos.character)
        else {
            return Vec::new();
        };
        let spans = match kind {
            StructuralKind::Definition => crate::structural::definition(&state.doc, offset)
                .into_iter()
                .collect::<Vec<_>>(),
            StructuralKind::References {
                include_declaration,
            } => crate::structural::references(&state.doc, offset, include_declaration),
        };
        spans
            .into_iter()
            .map(|(s, e)| {
                let (sl, sc) = crate::structural::byte_to_position(source, s);
                let (el, ec) = crate::structural::byte_to_position(source, e);
                Location {
                    uri: hick_uri.clone(),
                    range: Range {
                        start: Position::new(sl, sc),
                        end: Position::new(el, ec),
                    },
                }
            })
            .collect()
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

/// Which structural answer to compute.
#[derive(Debug, Clone, Copy)]
enum StructuralKind {
    Definition,
    References { include_declaration: bool },
}

/// Translate every `{uri, range}` / `{targetUri, targetRange…}` location in a
/// child LSP result from virtual-file coordinates back to .hick coordinates.
///
/// Locations whose URI is not a known virtual file, or whose positions fall on
/// lines the position map cannot translate (e.g. pasted/synthetic content),
/// are left untouched — the server-side bridge maps those through run
/// provenance instead.
pub(crate) fn translate_locations(
    value: serde_json::Value,
    index: &HashMap<String, (Url, PositionMap)>,
) -> serde_json::Value {
    use serde_json::Value;

    fn translate_range(range: &Value, map: &PositionMap) -> Option<Value> {
        let sl = range.pointer("/start/line")?.as_u64()? as u32;
        let sc = range.pointer("/start/character")?.as_u64()? as u32;
        let el = range.pointer("/end/line")?.as_u64()? as u32;
        let ec = range.pointer("/end/character")?.as_u64()? as u32;
        let (sl, sc) = map.to_source(sl, sc)?;
        let (el, ec) = map.to_source(el, ec)?;
        Some(serde_json::json!({
            "start": { "line": sl, "character": sc },
            "end": { "line": el, "character": ec },
        }))
    }

    fn walk(value: &mut Value, index: &HashMap<String, (Url, PositionMap)>) {
        match value {
            Value::Array(items) => {
                for item in items {
                    walk(item, index);
                }
            }
            Value::Object(obj) => {
                // Location: { uri, range }
                let plain = obj
                    .get("uri")
                    .and_then(|u| u.as_str())
                    .and_then(|u| index.get(u))
                    .and_then(|(hick, map)| {
                        let range = translate_range(obj.get("range")?, map)?;
                        Some((hick.as_str().to_string(), range))
                    });
                if let Some((uri, range)) = plain {
                    obj.insert("uri".into(), Value::String(uri));
                    obj.insert("range".into(), range);
                    return;
                }
                // LocationLink: { targetUri, targetRange, targetSelectionRange }
                let link = obj
                    .get("targetUri")
                    .and_then(|u| u.as_str())
                    .and_then(|u| index.get(u))
                    .and_then(|(hick, map)| {
                        let range = translate_range(obj.get("targetRange")?, map)?;
                        let sel = obj
                            .get("targetSelectionRange")
                            .and_then(|r| translate_range(r, map));
                        Some((hick.as_str().to_string(), range, sel))
                    });
                if let Some((uri, range, sel)) = link {
                    obj.insert("targetUri".into(), Value::String(uri));
                    obj.insert("targetRange".into(), range);
                    if let Some(sel) = sel {
                        obj.insert("targetSelectionRange".into(), sel);
                    }
                    return;
                }
                for (_, v) in obj.iter_mut() {
                    walk(v, index);
                }
            }
            _ => {}
        }
    }

    let mut value = value;
    walk(&mut value, index);
    value
}

/// Translate every `{start: {line, character}, end: …}` range in a child
/// result (hover ranges, completion text edits) with one position map — used
/// for results that carry ranges without URIs, all belonging to the request's
/// own virtual file. Untranslatable ranges are removed rather than left in
/// virtual coordinates.
pub(crate) fn translate_bare_ranges(
    value: serde_json::Value,
    map: &PositionMap,
) -> serde_json::Value {
    use serde_json::Value;

    fn is_range(v: &Value) -> bool {
        v.pointer("/start/line").is_some()
            && v.pointer("/start/character").is_some()
            && v.pointer("/end/line").is_some()
            && v.pointer("/end/character").is_some()
    }

    fn translate(v: &Value, map: &PositionMap) -> Option<Value> {
        let sl = v.pointer("/start/line")?.as_u64()? as u32;
        let sc = v.pointer("/start/character")?.as_u64()? as u32;
        let el = v.pointer("/end/line")?.as_u64()? as u32;
        let ec = v.pointer("/end/character")?.as_u64()? as u32;
        let (sl, sc) = map.to_source(sl, sc)?;
        let (el, ec) = map.to_source(el, ec)?;
        Some(serde_json::json!({
            "start": { "line": sl, "character": sc },
            "end": { "line": el, "character": ec },
        }))
    }

    fn walk(value: &mut Value, map: &PositionMap) {
        match value {
            Value::Array(items) => {
                for item in items {
                    walk(item, map);
                }
            }
            Value::Object(obj) => {
                let keys: Vec<String> = obj.keys().cloned().collect();
                for key in keys {
                    let child = obj.get(&key).cloned().unwrap_or(Value::Null);
                    if is_range(&child) {
                        match translate(&child, map) {
                            Some(t) => {
                                obj.insert(key, t);
                            }
                            None => {
                                obj.remove(&key);
                            }
                        }
                    } else if let Some(v) = obj.get_mut(&key) {
                        walk(v, map);
                    }
                }
            }
            _ => {}
        }
    }

    let mut value = value;
    walk(&mut value, map);
    value
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
                references_provider: Some(OneOf::Left(true)),
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

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let Some((vf_uri, lang, vpos)) = self.virtual_target(&uri, pos).await else {
            return Ok(None);
        };
        let Some(result) = self
            .child_request("textDocument/completion", &vf_uri, &lang, vpos, None)
            .await
        else {
            return Ok(None);
        };
        // Completion ranges (textEdit etc.) are in virtual-file coordinates.
        let map = {
            let index = self.vfile_index.read().await;
            index.get(&vf_uri).map(|m| m.position_map.clone())
        };
        let result = match map {
            Some(map) => translate_bare_ranges(result, &map),
            None => result,
        };
        Ok(serde_json::from_value(result).ok())
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let Some((vf_uri, lang, vpos)) = self.virtual_target(&uri, pos).await else {
            return Ok(None);
        };
        let Some(result) = self
            .child_request("textDocument/hover", &vf_uri, &lang, vpos, None)
            .await
        else {
            return Ok(None);
        };
        let map = {
            let index = self.vfile_index.read().await;
            index.get(&vf_uri).map(|m| m.position_map.clone())
        };
        let result = match map {
            Some(map) => translate_bare_ranges(result, &map),
            None => result,
        };
        Ok(serde_json::from_value(result).ok())
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        // Structural first: paste → its copy definition.
        let structural = self
            .structural_spans(&uri, pos, StructuralKind::Definition)
            .await;
        if let Some(loc) = structural.into_iter().next() {
            return Ok(Some(GotoDefinitionResponse::Scalar(loc)));
        }

        let Some((vf_uri, lang, vpos)) = self.virtual_target(&uri, pos).await else {
            return Ok(None);
        };
        let Some(result) = self
            .child_request("textDocument/definition", &vf_uri, &lang, vpos, None)
            .await
        else {
            return Ok(None);
        };
        let index = self.index_snapshot().await;
        let result = translate_locations(result, &index);
        Ok(serde_json::from_value(result).ok())
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let include_declaration = params.context.include_declaration;

        // Structural copy/paste references always contribute.
        let mut out = self
            .structural_spans(
                &uri,
                pos,
                StructuralKind::References {
                    include_declaration,
                },
            )
            .await;

        // Child references, translated back to .hick coordinates, merged in.
        if let Some((vf_uri, lang, vpos)) = self.virtual_target(&uri, pos).await
            && let Some(result) = self
                .child_request(
                    "textDocument/references",
                    &vf_uri,
                    &lang,
                    vpos,
                    Some(serde_json::json!({
                        "context": { "includeDeclaration": include_declaration }
                    })),
                )
                .await
        {
            let index = self.index_snapshot().await;
            let translated = translate_locations(result, &index);
            if let Ok(locs) = serde_json::from_value::<Vec<Location>>(translated) {
                out.extend(locs);
            }
        }

        out.sort_by(|a, b| {
            (
                a.uri.as_str(),
                a.range.start.line,
                a.range.start.character,
                a.range.end.line,
            )
                .cmp(&(
                    b.uri.as_str(),
                    b.range.start.line,
                    b.range.start.character,
                    b.range.end.line,
                ))
        });
        out.dedup();
        if out.is_empty() {
            Ok(None)
        } else {
            Ok(Some(out))
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::virtual_file::VirtualSegment;
    use hick_lang::SourceSpan;

    fn seg(text: &str, start_line: usize, col: usize) -> VirtualSegment {
        VirtualSegment {
            text: text.to_string(),
            source_span: Some(SourceSpan::new(0, text.len(), start_line, col)),
            source_line: start_line,
            source_column: col,
        }
    }

    fn index_with(vf: &str, hick: &str, map: PositionMap) -> HashMap<String, (Url, PositionMap)> {
        let mut index = HashMap::new();
        index.insert(vf.to_string(), (Url::parse(hick).unwrap(), map));
        index
    }

    #[test]
    fn references_result_translates_to_hick_coordinates() {
        // Virtual file lines 0..3 map to .hick source lines 9..12 (1-based
        // 10..13), each indented 4 columns.
        let map = PositionMap::build(&[seg("fn a() {}\nfn b() {\n    a();\n}\n", 10, 4)]);
        let index = index_with("file:///tmp/vf/main.rs", "hick:///doc.hick", map);

        let child_result = serde_json::json!([
            { "uri": "file:///tmp/vf/main.rs",
              "range": { "start": { "line": 0, "character": 3 },
                         "end":   { "line": 0, "character": 4 } } },
            { "uri": "file:///tmp/vf/main.rs",
              "range": { "start": { "line": 2, "character": 4 },
                         "end":   { "line": 2, "character": 5 } } },
        ]);

        let translated = translate_locations(child_result, &index);
        let locs: Vec<Location> = serde_json::from_value(translated).unwrap();
        assert_eq!(locs.len(), 2);
        assert_eq!(locs[0].uri.as_str(), "hick:///doc.hick");
        assert_eq!(locs[0].range.start, Position::new(9, 7));
        assert_eq!(locs[0].range.end, Position::new(9, 8));
        assert_eq!(locs[1].range.start, Position::new(11, 8));
        assert_eq!(locs[1].range.end, Position::new(11, 9));
    }

    #[test]
    fn references_in_unknown_vfile_pass_through() {
        let map = PositionMap::build(&[seg("x\n", 1, 0)]);
        let index = index_with("file:///tmp/vf/main.rs", "hick:///doc.hick", map);

        let child_result = serde_json::json!([
            { "uri": "file:///somewhere/else.rs",
              "range": { "start": { "line": 5, "character": 0 },
                         "end":   { "line": 5, "character": 3 } } },
        ]);

        let translated = translate_locations(child_result.clone(), &index);
        assert_eq!(translated, child_result);
    }

    #[test]
    fn references_on_untranslatable_lines_keep_virtual_location() {
        // Map covers only virtual line 0; a result on line 7 (e.g. inside
        // pasted content beyond the mapped segment) must survive untouched so
        // the server-side bridge can map it through provenance.
        let map = PositionMap::build(&[seg("only one line\n", 3, 0)]);
        let index = index_with("file:///tmp/vf/main.rs", "hick:///doc.hick", map);

        let child_result = serde_json::json!([
            { "uri": "file:///tmp/vf/main.rs",
              "range": { "start": { "line": 7, "character": 2 },
                         "end":   { "line": 7, "character": 6 } } },
        ]);

        let translated = translate_locations(child_result.clone(), &index);
        assert_eq!(translated, child_result);
    }

    #[test]
    fn location_links_translate_target_ranges() {
        let map = PositionMap::build(&[seg("fn a() {}\n", 5, 2)]);
        let index = index_with("file:///tmp/vf/main.rs", "hick:///doc.hick", map);

        let child_result = serde_json::json!([
            { "targetUri": "file:///tmp/vf/main.rs",
              "targetRange": { "start": { "line": 0, "character": 0 },
                               "end":   { "line": 0, "character": 9 } },
              "targetSelectionRange": { "start": { "line": 0, "character": 3 },
                                        "end":   { "line": 0, "character": 4 } } },
        ]);

        let translated = translate_locations(child_result, &index);
        assert_eq!(
            translated[0]["targetUri"].as_str().unwrap(),
            "hick:///doc.hick"
        );
        assert_eq!(translated[0]["targetRange"]["start"]["line"], 4);
        assert_eq!(translated[0]["targetRange"]["start"]["character"], 2);
        assert_eq!(
            translated[0]["targetSelectionRange"]["start"]["character"],
            5
        );
    }

    #[test]
    fn bare_ranges_translate_or_drop() {
        let map = PositionMap::build(&[seg("hello\n", 4, 2)]);
        let hover = serde_json::json!({
            "contents": { "kind": "plaintext", "value": "info" },
            "range": { "start": { "line": 0, "character": 0 },
                       "end":   { "line": 0, "character": 5 } },
        });
        let translated = translate_bare_ranges(hover, &map);
        assert_eq!(translated["range"]["start"]["line"], 3);
        assert_eq!(translated["range"]["start"]["character"], 2);

        let unmappable = serde_json::json!({
            "contents": "x",
            "range": { "start": { "line": 9, "character": 0 },
                       "end":   { "line": 9, "character": 1 } },
        });
        let translated = translate_bare_ranges(unmappable, &map);
        assert!(translated.get("range").is_none());
        assert_eq!(translated["contents"], "x");
    }
}
