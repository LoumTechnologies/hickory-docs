//! Startup-bound private HTTP execution. No runtime enable/disable endpoint.
//! Unsupported surfaces fail closed until their target quarantine is integrated.
use super::{
    LocalState, RunRecord,
    api::{ApiError, ApiResult},
};
use axum::{
    Json,
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use hick_literate::prediction::{OutputEncoding, SealedRequest, SourceQuestion};
use hick_observe::prediction::{Actor, Identity, Ledger, Predicate, Scenario, Seal};
use hickory_agent::{EditSession, ToolInvocation, execute_tool};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{path::PathBuf, sync::Arc};

/// Supplied by a trusted startup host, never decoded from model/HTTP arguments.
/// The host captures inputs/environment and scenario correspondence independently.
pub struct Binding {
    pub ledger: Arc<Ledger>,
    pub request: SealedRequest,
    pub encoding: OutputEncoding,
    pub question: Option<Scenario>,
}
pub(crate) struct Bound {
    pub doc: String,
    pub(crate) startup_fingerprint: std::sync::OnceLock<String>,
    path: PathBuf,
    session: tokio::sync::Mutex<EditSession>,
    ledger: Arc<Ledger>,
    identity: std::sync::RwLock<Identity>,
    question: Scenario,
    source_question: SourceQuestion,
    actor: Actor,
}
impl Bound {
    pub(super) async fn open(
        state: &LocalState,
        path: PathBuf,
        binding: Binding,
    ) -> anyhow::Result<Arc<Self>> {
        anyhow::ensure!(
            state.params.is_empty(),
            "Private HTTP execution does not accept uncaptured parameters"
        );
        let doc = state.id_for_doc_path(
            path.to_str()
                .ok_or_else(|| anyhow::anyhow!("Source path requires UTF-8"))?,
        );
        let question = match binding.question {
            Some(question) => question,
            None => {
                let history = binding.ledger.history(None)?;
                let event = history
                    .ordering
                    .iter()
                    .find(|entry| {
                        entry["event"]["kind"] == "sealed"
                            && entry["event"]["id"] == binding.request.seal
                    })
                    .ok_or_else(|| anyhow::anyhow!("Captured host question unavailable"))?;
                serde_json::from_value::<Seal>(event["event"]["seal"].clone())?.scenario
            }
        };
        let identity = binding.request.identity.clone();
        let actor = binding.request.actor.clone();
        let target = binding.request.target.clone();
        let session = EditSession::open_prediction(
            &path,
            binding.ledger.clone(),
            binding.request,
            binding.encoding,
        )
        .await?;
        let source_question = SourceQuestion::capture(
            &SealedRequest {
                seal: String::new(),
                request_id: String::new(),
                actor: actor.clone(),
                identity: identity.clone(),
                target,
            },
            session.approved_source(),
            binding.encoding,
        )?;
        Ok(Arc::new(Self {
            doc,
            startup_fingerprint: std::sync::OnceLock::new(),
            path,
            session: tokio::sync::Mutex::new(session),
            ledger: binding.ledger,
            identity: std::sync::RwLock::new(identity),
            question,
            source_question,
            actor,
        }))
    }
    async fn seal_prediction(
        &self,
        expected: &str,
        predicted: Predicate,
        rationale: String,
    ) -> Result<Value, String> {
        let mut session = self.session.lock().await;
        session
            .refresh_source()
            .await
            .map_err(|_| "Captured source changed outside this session".to_owned())?;
        let mut identity = self.identity.read().unwrap().clone();
        let source = session.approved_source();
        use sha2::{Digest, Sha256};
        let captured = format!("{:x}", Sha256::digest(source.as_bytes()));
        if expected != captured {
            return Err("Prediction source snapshot is stale".into());
        }
        identity.source = captured;
        let target = self
            .source_question
            .rebind(&identity, source)
            .map_err(|_| {
                "The host-selected question changed or the repaired source is unsupported"
                    .to_owned()
            })?;
        let receipt=self.ledger.seal(self.ledger.history(None).map_err(|_|"Prediction history unavailable")?.generation,
            Seal {scenario:self.question.clone(),identity:identity.clone(),target:target.clone(),predicted,rationale},self.actor.clone())
            .map_err(|_|"Prediction seal refused: inspect the requirement, bounded predicate and current history".to_owned())?;
        session
            .replace_prediction(SealedRequest {
                seal: receipt.id.clone(),
                request_id: format!("sealed:{}", receipt.id),
                actor: self.actor.clone(),
                identity: identity.clone(),
                target,
            })
            .map_err(|_| "Prediction policy replacement refused".to_owned())?;
        *self.identity.write().unwrap() = identity;
        Ok(
            json!({"sealed":receipt.id,"source_hash":expected,"generation":receipt.generation,"executed":false,"current_certification":false}),
        )
    }
    pub(crate) async fn mcp_tool(&self, executor: crate::ExecutorChoice, params: &Value) -> Value {
        let refuse = || json!({"content":[{"type":"text","text":"Private prediction policy refuses this tool or document; use captured source reading/editing and sealed Verify"}],"isError":true});
        let Some(name) = params["name"].as_str() else {
            return refuse();
        };
        let mut arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));
        if let Some(doc) = arguments["doc"].as_str() {
            let selected = self.path.parent().unwrap().join(doc).canonicalize().ok();
            if selected.as_ref() != Some(&self.path) {
                return refuse();
            }
        }
        if name == "seal_prediction" {
            let Ok(body) = serde_json::from_value::<SealRequest>(arguments.clone()) else {
                return refuse();
            };
            return match self
                .seal_prediction(&body.expected_source_hash, body.predicted, body.rationale)
                .await
            {
                Ok(value) => {
                    json!({"content":[{"type":"text","text":value.to_string()}],"isError":false})
                }
                Err(reason) => json!({"content":[{"type":"text","text":reason}],"isError":true}),
            };
        }
        if !["read_doc", "edit_doc", "verify"].contains(&name) {
            return refuse();
        }
        if let Some(args) = arguments.as_object_mut() {
            args.remove("doc");
        }
        let invocation = match crate::mcp::Server::invocation(name, &arguments) {
            Ok(inv) => inv,
            Err(_) => return refuse(),
        };
        let executor = match executor.build().await {
            Ok(executor) => executor,
            Err(_) => return refuse(),
        };
        let mut session = self.session.lock().await;
        let result = execute_tool(&mut session, executor.clone(), &invocation).await;
        let _ = executor.shutdown().await;
        json!({"content":[{"type":"text","text":result.text}],"isError":!result.ok})
    }
    pub(super) fn start(
        self: &Arc<Self>,
        state: &LocalState,
        id: &str,
        expected: Option<String>,
    ) -> ApiResult<String> {
        if id != self.doc {
            return Err(ApiError::unprocessable(
                "Private prediction belongs to a different document",
            ));
        }
        let identity = self.identity.read().unwrap().clone();
        if expected
            .as_ref()
            .is_some_and(|hash| hash.trim_start_matches("sha256:") != identity.source)
        {
            return Err(ApiError::conflict(
                "Requested source differs from the private seal",
            ));
        }
        let run = format!("{:016x}", super::rand_id());
        state.runs.lock().unwrap().insert(
            run.clone(),
            RunRecord {
                document: id.into(),
                requested_source_hash: expected,
                error: None,
                source_hash: Some(identity.source.clone()),
                status: "running".into(),
                started_at: super::now_rfc3339(),
                blocks: json!([]),
            },
        );
        let bound = self.clone();
        let state = state.clone();
        let key = run.clone();
        tokio::spawn(async move {
            let mut session = bound.session.lock().await;
            let result = match state.executor.build().await {
                Ok(executor) => {
                    let invocation = ToolInvocation::synthetic("verify", vec![], None)
                        .expect("fixed private Verify invocation");
                    let result = execute_tool(&mut session, executor.clone(), &invocation).await;
                    let _ = executor.shutdown().await;
                    Some(result)
                }
                Err(_) => None,
            };
            let mut runs = state.runs.lock().unwrap();
            if let Some(record) = runs.get_mut(&key) {
                record.status = if result.as_ref().is_some_and(|result| result.ok) {
                    "ok"
                } else {
                    "failed"
                }
                .into();
                // Shared-tool admission errors are bounded; successful parsing
                // is limited to the gate's answer-free ManagedOutput schema.
                record.blocks = result
                    .as_ref()
                    .and_then(|result| {
                        serde_json::from_str::<hick_literate::prediction::ManagedOutput>(
                            &result.text,
                        )
                        .ok()
                    })
                    .map(|summary| json!([{"kind":"prediction","status":record.status,"summary":summary.summary,"dispatched":summary.dispatched}]))
                    .unwrap_or_else(|| json!([]));
                if record.status != "ok" {
                    record.error=Some("Private prediction verification is non-passing; held-out values and diagnostics remain withheld".into());
                }
            }
        });
        Ok(run)
    }
}

pub(super) async fn quarantine(
    State(state): State<LocalState>,
    request: Request,
    next: Next,
) -> Response {
    let Some(bound) = &state.private_prediction else {
        return next.run(request).await;
    };
    let path = request.uri().path().trim_start_matches("/api");
    let method = request.method().as_str();
    let allowed = (method == "GET" && ["/prediction/state", "/prediction/history"].contains(&path))
        || (method == "POST"
            && [
                "/prediction/tool",
                "/prediction/seal",
                "/prediction/preview",
            ]
            .contains(&path))
        || (method == "POST"
            && [
                format!("/docs/{}/run", bound.doc),
                format!("/docs/{}/check", bound.doc),
            ]
            .contains(&path.to_owned()))
        || (method == "GET" && path.starts_with("/runs/") && !path[6..].contains('/'));
    if allowed {
        next.run(request).await
    } else {
        ApiError::unprocessable("This surface is unavailable in the private prediction session; use captured source tools, sealed Verify and withheld history").into_response()
    }
}

pub(super) async fn status(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let bound = state
        .private_prediction
        .as_ref()
        .ok_or_else(|| ApiError::unprocessable("No private prediction session"))?;
    let session = bound.session.lock().await;
    let captured = session.source_snapshots();
    let sealed_identity = bound.identity.read().unwrap().clone();
    let mut current_identity = sealed_identity.clone();
    if let Some(hash) = captured[0]["sha256"].as_str() {
        current_identity.source = hash.to_owned();
    }
    let target = bound
        .source_question
        .rebind(&current_identity, session.approved_source())
        .ok();
    Ok(Json(
        json!({"document":bound.doc,"source":bound.path.file_name(),"source_hash":current_identity.source,
        "sealed_source_hash":sealed_identity.source,"current":captured[0]["current"],"actor":bound.actor,
        "question":bound.question,"target":target,"identities":current_identity,"actual_values":"withheld",
        "environment_scope":"Host startup identities; this read does not revalidate or freeze the execution environment",
        "supported":["read_doc","edit_doc","seal_prediction","verify"],"application_integration":"incomplete"}),
    ))
}
pub(super) async fn history(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let bound = state
        .private_prediction
        .as_ref()
        .ok_or_else(|| ApiError::unprocessable("No private prediction session"))?;
    let history = bound
        .ledger
        .history(None)
        .map_err(|_| ApiError::unprocessable("Private prediction history unavailable"))?;
    Ok(Json(serde_json::to_value(history).map_err(|_| {
        ApiError::internal("Private prediction history unavailable")
    })?))
}
#[derive(Deserialize)]
pub(super) struct SourceTool {
    name: String,
    #[serde(default)]
    args: Vec<(String, String)>,
    input: Option<String>,
}
pub(super) async fn source_tool(
    State(state): State<LocalState>,
    Json(body): Json<SourceTool>,
) -> ApiResult<Json<Value>> {
    let bound = state
        .private_prediction
        .as_ref()
        .ok_or_else(|| ApiError::unprocessable("No private prediction session"))?;
    if !["read_doc", "edit_doc"].contains(&body.name.as_str()) {
        return Err(ApiError::unprocessable(
            "Only captured source reading/editing is supported; use sealed Verify for execution",
        ));
    }
    let invocation = ToolInvocation::synthetic(body.name, body.args, body.input)
        .map_err(|_| ApiError::bad_request("Invalid captured source tool request"))?;
    let executor = state
        .executor
        .build()
        .await
        .map_err(|_| ApiError::unprocessable("Private tool executor unavailable"))?;
    let mut session = bound.session.lock().await;
    let result = execute_tool(&mut session, executor.clone(), &invocation).await;
    let _ = executor.shutdown().await;
    Ok(Json(
        json!({"ok":result.ok,"text":result.text,"source":result.reads.first().map(|read|&read.content),"source_hash":result.reads.first().map(|read|&read.sha256)}),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SealRequest {
    expected_source_hash: String,
    predicted: Predicate,
    rationale: String,
}
pub(super) async fn seal(
    State(state): State<LocalState>,
    Json(body): Json<SealRequest>,
) -> ApiResult<Json<Value>> {
    let bound = state
        .private_prediction
        .as_ref()
        .ok_or_else(|| ApiError::unprocessable("No private prediction session"))?;
    bound
        .seal_prediction(&body.expected_source_hash, body.predicted, body.rationale)
        .await
        .map(Json)
        .map_err(ApiError::unprocessable)
}

/// Conditional MCP capabilities for a host-bound private workspace.
pub(crate) fn catalogue() -> Value {
    let mut catalogue = crate::mcp_catalogue::tool_catalogue();
    catalogue["tools"].as_array_mut().unwrap().retain(|tool| {
        matches!(
            tool["name"].as_str(),
            Some("read_doc" | "edit_doc" | "verify")
        )
    });
    catalogue["tools"].as_array_mut().unwrap().push(json!({"name":"seal_prediction","description":"Seal a prediction and rationale against the current captured source before private Verify. The host owns the question, requirement, inputs, actor and environment. Source repairs need a new seal with the original expectation; changed expectations cannot earn promotion. No actual target value is returned.",
        "inputSchema":{"type":"object","properties":{"expected_source_hash":{"type":"string","pattern":"^[a-f0-9]{64}$"},"rationale":{"type":"string","minLength":1,"maxLength":8192},"predicted":{"type":"object","description":"A bounded predicate: {kind: equals, value: JSON}, {kind: number_at_most, value: number}, or {kind: fields, value: map of predicates}."}},"required":["expected_source_hash","predicted","rationale"],"additionalProperties":false}}));
    catalogue
}

/// Host edit review uses the same captured EditSession, never an ordinary live preview.
pub(super) async fn preview(
    State(state): State<LocalState>,
    Json(mut params): Json<Value>,
) -> ApiResult<Json<Value>> {
    let bound = state
        .private_prediction
        .as_ref()
        .ok_or_else(|| ApiError::unprocessable("No private prediction session"))?;
    if params["name"] != "edit_doc" {
        return Err(ApiError::unprocessable(
            "Only captured source edits can be previewed",
        ));
    }
    let args = params
        .get_mut("arguments")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| ApiError::bad_request("Edit arguments required"))?;
    if let Some(doc) = args.remove("doc") {
        let selected = doc
            .as_str()
            .and_then(|doc| bound.path.parent().unwrap().join(doc).canonicalize().ok());
        if selected.as_ref() != Some(&bound.path) {
            return Err(ApiError::unprocessable("Different document"));
        }
    }
    let inv = crate::mcp::Server::invocation("edit_doc", &json!(args))
        .map_err(|_| ApiError::bad_request("Invalid captured edit"))?;
    let mut session = bound.session.lock().await;
    let (surface, before, after) = session
        .preview_edit(&inv)
        .await
        .map_err(|_| ApiError::conflict("Captured edit is stale or unsupported"))?;
    Ok(Json(
        json!({"surface":surface,"before":before,"after":after,"revision":session.revision(),"sources":session.source_snapshots()}),
    ))
}
