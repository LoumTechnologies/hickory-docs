use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

pub(crate) struct HickBackend {
    client: Client,
    /// Open document states, keyed by URI.
    documents: Arc<RwLock<HashMap<Url, DocumentState>>>,
}

struct DocumentState {
    source: String,
    #[allow(dead_code)]
    version: i32,
}

impl HickBackend {
    pub(crate) fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Parse the document source and publish diagnostics.
    ///
    /// On success, clears any existing diagnostics. On failure, publishes a
    /// single diagnostic with the parse error so the user gets immediate
    /// feedback in the editor.
    async fn parse_and_publish_diagnostics(&self, uri: &Url, source: &str) {
        let diagnostics = match hick_lang::parse(source) {
            Ok(_) => vec![],
            Err(e) => {
                let (line, message) = parse_error_location(&e);
                // LSP lines are 0-based; hick-lang lines are 1-based.
                let lsp_line = if line > 0 { line - 1 } else { 0 } as u32;
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
                }]
            }
        };

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }
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
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let source = params.text_document.text;
        let version = params.text_document.version;

        self.parse_and_publish_diagnostics(&uri, &source).await;

        let mut docs = self.documents.write().await;
        docs.insert(
            uri,
            DocumentState {
                source,
                version,
            },
        );
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        // We use FULL sync, so there's exactly one change with the full text.
        if let Some(change) = params.content_changes.into_iter().next() {
            let source = change.text;

            self.parse_and_publish_diagnostics(&uri, &source).await;

            let mut docs = self.documents.write().await;
            docs.insert(
                uri,
                DocumentState {
                    source,
                    version,
                },
            );
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        // Clear diagnostics for the closed document.
        self.client
            .publish_diagnostics(uri.clone(), vec![], None)
            .await;

        let mut docs = self.documents.write().await;
        docs.remove(&uri);
    }

    async fn completion(
        &self,
        _params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        // Child LSP proxying will be wired up in a later step.
        Ok(None)
    }

    async fn hover(&self, _params: HoverParams) -> Result<Option<Hover>> {
        // Child LSP proxying will be wired up in a later step.
        Ok(None)
    }

    async fn goto_definition(
        &self,
        _params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        // Child LSP proxying will be wired up in a later step.
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
    }
}
