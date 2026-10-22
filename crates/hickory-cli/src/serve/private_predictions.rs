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
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

/// Supplied by a trusted startup host, never decoded from model/HTTP arguments.
/// The host captures inputs/environment and scenario correspondence independently.
pub struct Binding {
    pub ledger: Arc<Ledger>,
    pub request: SealedRequest,
    pub encoding: OutputEncoding,
    pub question: Option<Scenario>,
    pub runtime: Option<hick_literate::prediction::environment::Profile>,
    pub inputs: Option<hick_literate::prediction::SourceInputs>,
    pub retained: Option<super::PrivatePredictionDebuggerPlan>,
}
pub(crate) struct Bound {
    pub doc: String,
    pub(crate) startup_fingerprint: std::sync::OnceLock<String>,
    pub(super) path: PathBuf,
    pub(super) session: tokio::sync::Mutex<EditSession>,
    pub(super) ledger: Arc<Ledger>,
    pub(super) identity: std::sync::RwLock<Identity>,
    pub(super) question: Scenario,
    pub(super) source_question: SourceQuestion,
    pub(super) recovery: std::sync::OnceLock<super::private_prediction_recovery::Recovery>,
    pub(super) runtime: Option<hick_literate::prediction::environment::Profile>,
    pub(super) inputs: Option<hick_literate::prediction::SourceInputs>,
    pub(super) actor: Actor,
    epoch: AtomicU64,
    retired: AtomicBool,
    pub(super) retained_plan: Option<super::PrivatePredictionDebuggerPlan>,
    pub(super) retained: tokio::sync::Mutex<super::private_prediction_debugger::Controller>,
    pub(super) agent: tokio::sync::Mutex<Option<super::private_prediction_agent::Run>>,
    pub(super) edits: Arc<super::acp::edits::Edits>,
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
        anyhow::ensure!(
            actor.kind == "agent",
            "This managed prediction service requires an agent predictor; human-only inspection needs a separately isolated host surface"
        );
        let target = binding.request.target.clone();
        if let Some(plan) = &binding.retained {
            plan.validate(&question, &target)?;
            anyhow::ensure!(
                binding.runtime.is_some() && binding.inputs.is_some(),
                "Retained coach needs configured runtime and fixed host inputs"
            );
        }
        let mut session = EditSession::open_prediction(
            &path,
            binding.ledger.clone(),
            binding.request,
            binding.encoding,
        )
        .await?;
        if let Some(inputs) = &binding.inputs {
            inputs.validate(&identity, session.approved_source())?;
            anyhow::ensure!(
                inputs.value == question.inputs,
                "Host question inputs differ from source binding"
            );
        }
        if let Some(runtime) = &binding.runtime {
            anyhow::ensure!(
                binding.inputs.is_some(),
                "Current runtime certification requires host-reviewed input anchors"
            );
            anyhow::ensure!(
                state.executor == crate::ExecutorChoice::Local
                    && !matches!(target, hick_observe::prediction::Target::Output { .. }),
                "This runtime profile supports local Python capture/combined targets only"
            );
            let hick_literate::prediction::environment::Profile::PythonDebugpy { project } =
                runtime;
            anyhow::ensure!(
                project.canonicalize()? == path.parent().unwrap().canonicalize()?,
                "Runtime capture must use this document's actual adapter-discovery project"
            );
            anyhow::ensure!(
                runtime.capture().await? == identity.environment,
                "Runtime differs from startup identity"
            );
            session.bind_prediction_environment(runtime.clone())?;
        }
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
            recovery: std::sync::OnceLock::new(),
            runtime: binding.runtime,
            inputs: binding.inputs,
            actor,
            epoch: AtomicU64::new(0),
            retired: AtomicBool::new(false),
            retained_plan: binding.retained,
            retained: tokio::sync::Mutex::new(Default::default()),
            agent: tokio::sync::Mutex::new(None),
            edits: Default::default(),
        }))
    }
    pub(crate) fn is_retired(&self) -> bool {
        self.retired.load(Ordering::SeqCst)
    }
    pub(crate) async fn retire(&self, state: &LocalState) -> anyhow::Result<()> {
        let agent = self.agent.lock().await;
        anyhow::ensure!(
            !super::private_prediction_agent::running(agent.as_ref()),
            "Stop the private investigation and wait for it to finish before ending"
        );
        let _session=self.session.try_lock().map_err(|_|anyhow::anyhow!("Private source or verification operation is busy; wait for completion before ending"))?;
        anyhow::ensure!(
            !state
                .runs
                .lock()
                .unwrap()
                .values()
                .any(|run| run.status == "running"),
            "Wait for private verification to finish before ending"
        );
        self.retired.store(true, Ordering::SeqCst);
        self.retained.lock().await.reset().await;
        Ok(())
    }
    pub(super) fn source_path(&self) -> &std::path::Path {
        &self.path
    }
    async fn seal_prediction(
        &self,
        expected: &str,
        predicted: Predicate,
        rationale: String,
    ) -> Result<Value, String> {
        let mut session = self.session.lock().await;
        if self.retired.load(Ordering::SeqCst) {
            return Err("Private session ended; reopen with a fresh context".into());
        }
        session
            .refresh_source()
            .await
            .map_err(|_| "Captured source changed outside this session".to_owned())?;
        let mut identity = self.identity.read().unwrap().clone();
        if let Some(runtime) = &self.runtime {
            if runtime
                .capture()
                .await
                .map_err(|_| "Runtime identity unavailable")?
                != identity.environment
            {
                return Err("Runtime changed; trusted host recovery is required".into());
            }
        }
        let source = session.approved_source();
        if let Some(inputs) = &self.inputs {
            inputs
                .validate(&identity, source)
                .map_err(|_| "Host-selected inputs changed; trusted host review is required")?;
        }
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
        self.epoch.fetch_add(1, Ordering::SeqCst);
        Ok(
            json!({"sealed":receipt.id,"source_hash":expected,"generation":receipt.generation,"executed":false,"current_certification":false}),
        )
    }
    pub(crate) async fn mcp_tool(
        self: &Arc<Self>,
        executor: crate::ExecutorChoice,
        params: &Value,
    ) -> Value {
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
        if ["seal_debugger_prediction", "continue_debugger_prediction"].contains(&name) {
            return match self.debugger_tool(name, arguments).await {
                Ok(value) => {
                    let failed = value.get("summary").is_some_and(|summary| {
                        !matches!(
                            summary["classification"].as_str(),
                            Some(
                                "first_attempt_verified"
                                    | "informed_prediction_verified"
                                    | "verified_after_correction"
                                    | "verified_after_prior_attempt"
                            )
                        )
                    });
                    json!({"content":[{"type":"text","text":value.to_string()}],"structuredContent":value,"isError":failed})
                }
                Err(_) => {
                    json!({"isError":true,"content":[{"type":"text","text":"Retained prediction unavailable: check host plan, current source/runtime, initial/future seal ordering and bounded predicate. No target values are disclosed."}]})
                }
            };
        }
        if name == "seal_prediction" {
            let Ok(body) = serde_json::from_value::<SealRequest>(arguments.clone()) else {
                return json!({"isError":true,"structuredContent":{"error_code":"invalid_prediction_schema"},"content":[{"type":"text","text":"Invalid prediction arguments. Provide expected_source_hash, rationale and predicted. Every fields value must itself be a predicate, for example {kind: fields, value: {output: {kind: equals, value: YOUR_EXPECTED_JSON}, debugger: {kind: equals, value: YOUR_EXPECTED_VALUE}}}. Do not put raw output/debugger scalars directly under fields.value. Nothing executed."}]});
            };
            return match self
                .seal_prediction(&body.expected_source_hash, body.predicted, body.rationale)
                .await
            {
                Ok(value) => {
                    json!({"content":[{"type":"text","text":value.to_string()}],"structuredContent":value,"isError":false})
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
        if self.retired.load(Ordering::SeqCst) {
            return refuse();
        }
        if name == "edit_doc" {
            self.retained.lock().await.reset().await;
        }
        let result = execute_tool(&mut session, executor.clone(), &invocation).await;
        let _ = executor.shutdown().await;
        if name == "edit_doc"
            && result.ok
            && self
                .retain_approved_source(session.approved_source())
                .is_err()
        {
            return json!({"isError":true,"content":[{"type":"text","text":"Source edit applied, but durable recovery receipt could not be retained; trusted host recovery is required before reopening"}]});
        }
        let structured = if name == "read_doc" && result.ok {
            result.reads.first().map(|read|json!({"source":read.content,"source_hash":read.sha256,"snapshot":read.sha256,"edit_baseline":true})).unwrap_or_else(||json!({}))
        } else if name == "verify" {
            serde_json::from_str::<hick_literate::prediction::ManagedOutput>(&result.text)
                .ok()
                .and_then(|value| serde_json::to_value(value).ok())
                .unwrap_or_else(
                    || json!({"accepted":false,"reason":"Private prediction verification refused"}),
                )
        } else {
            json!({"accepted":result.ok})
        };
        json!({"content":[{"type":"text","text":result.text}],"structuredContent":structured,"isError":!result.ok})
    }
    pub(super) fn start(
        self: &Arc<Self>,
        state: &LocalState,
        id: &str,
        expected: Option<String>,
    ) -> ApiResult<String> {
        if self.retired.load(Ordering::SeqCst) {
            return Err(ApiError::unprocessable("Private session ended"));
        }
        if id != self.doc {
            return Err(ApiError::unprocessable(
                "Private prediction belongs to a different document",
            ));
        }
        let identity = self.identity.read().unwrap().clone();
        let epoch = self.epoch.load(Ordering::SeqCst);
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
            use sha2::{Digest, Sha256};
            let current_identity = bound.identity.read().unwrap().clone();
            let admitted = !bound.retired.load(Ordering::SeqCst)
                && bound.epoch.load(Ordering::SeqCst) == epoch
                && current_identity == identity
                && format!("{:x}", Sha256::digest(session.approved_source())) == identity.source;
            let result = if !admitted {
                None
            } else {
                match state.executor.build().await {
                    Ok(executor) => {
                        let invocation = ToolInvocation::synthetic("verify", vec![], None)
                            .expect("fixed private Verify invocation");
                        let result =
                            execute_tool(&mut session, executor.clone(), &invocation).await;
                        let _ = executor.shutdown().await;
                        Some(result)
                    }
                    Err(_) => None,
                }
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
        let path = request.uri().path().trim_start_matches("/api");
        // Other windows retain their final UI receipts, but cannot launch a
        // terminal, debugger, Run or alternative context reader around policy.
        if path == "/v1/ui/windows" || path.starts_with("/v1/ui/windows/") {
            return next.run(request).await;
        }
        let _ordinary = if path != "/tour/private-practice" {
            Some(crate::engine::private_policy::ordinary())
        } else {
            None
        };
        if crate::engine::private_policy::overlaps(state.index.root()).unwrap_or(true)
            || crate::engine::private_prediction_active()
                .await
                .unwrap_or(true)
        {
            return ApiError::unprocessable("This ordinary engine surface is paused while a private practice is active. End that private practice to resume; its history remains retained.").into_response();
        }
        return next.run(request).await;
    };
    if bound.is_retired() {
        // Retain the End action's immutable result after execution admission has
        // closed. This is receipt/owner cleanup only, never a tool or read path.
        let path = request.uri().path().trim_start_matches("/api");
        let receipt = request.method() == "POST"
            && path.starts_with("/v1/ui/windows/")
            && path.ends_with("/receipts");
        let close = request.method() == "DELETE"
            && path
                .strip_prefix("/v1/ui/windows/")
                .is_some_and(|id| !id.is_empty() && !id.contains('/'));
        if receipt || close {
            return next.run(request).await;
        }
        return ApiError::unprocessable("Private session ended; open a fresh window to resume")
            .into_response();
    }
    let path = request.uri().path().trim_start_matches("/api");
    let method = request.method().as_str();
    let allowed = path == "/v1/ui/windows"
        || path.starts_with("/v1/ui/windows/")
        || (method == "GET"
            && [
                "/prediction/state",
                "/prediction/history",
                "/prediction/agent",
                "/prediction/evidence",
                "/prediction/completion",
            ]
            .contains(&path))
        || (method == "POST"
            && [
                "/prediction/tool",
                "/prediction/seal",
                "/prediction/preview",
                "/prediction/agent",
                "/prediction/agent/decision",
                "/prediction/agent/stop",
                "/prediction/promote",
                "/prediction/debugger",
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
    let retained = bound.retained.lock().await.status();
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
        json!({"retained_debugger":retained,"retained_plan":bound.retained_plan,"document":bound.doc,"source":bound.path.file_name().and_then(|name|name.to_str()),"source_hash":current_identity.source,
        "sealed_source_hash":sealed_identity.source,"current":captured[0]["current"],"actor":bound.actor,
        "question":bound.question,"target":target,"identities":current_identity,"actual_values":"withheld","runtime_revalidation":bound.runtime.is_some(),
        "environment_scope":if bound.runtime.is_some(){"Configured Python runtime is revalidated before dispatch and settlement; this status read does not freeze or revalidate the environment"}else{"Host startup identities; this read does not revalidate or freeze the execution environment"},
        "supported":if bound.retained_plan.is_some(){vec!["read_doc","edit_doc","seal_prediction","verify","seal_debugger_prediction","continue_debugger_prediction"]}else{vec!["read_doc","edit_doc","seal_prediction","verify"]},"application_integration":"incomplete"}),
    ))
}
pub(super) async fn history(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let bound = state
        .private_prediction
        .as_ref()
        .ok_or_else(|| ApiError::unprocessable("No private prediction session"))?;
    let history = bound
        .ledger
        .predictor_history(
            None,
            &bound.actor,
            &bound
                .scenario_keys()
                .map_err(|_| ApiError::unprocessable("Host questions unavailable"))?,
        )
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
    if bound.retired.load(Ordering::SeqCst) {
        return Err(ApiError::unprocessable("Private session ended"));
    }
    if invocation.name == "edit_doc" {
        bound.retained.lock().await.reset().await;
    }
    let result = execute_tool(&mut session, executor.clone(), &invocation).await;
    let _ = executor.shutdown().await;
    if invocation.name == "edit_doc" && result.ok {
        bound.retain_approved_source(session.approved_source()).map_err(|_|ApiError::unprocessable("Source edit applied, but durable recovery receipt unavailable; trusted host recovery is required"))?;
    }
    Ok(Json(
        json!({"ok":result.ok,"text":result.text,"source":result.reads.first().map(|read|&read.content),"source_hash":result.reads.first().map(|read|&read.sha256)}),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SealRequest {
    pub(super) expected_source_hash: String,
    pub(super) predicted: Predicate,
    pub(super) rationale: String,
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
pub(crate) fn catalogue(with_retained: bool) -> Value {
    let mut catalogue = crate::mcp_catalogue::tool_catalogue();
    catalogue["tools"].as_array_mut().unwrap().retain(|tool| {
        matches!(
            tool["name"].as_str(),
            Some("read_doc" | "edit_doc" | "verify")
        )
    });
    for tool in catalogue["tools"].as_array_mut().unwrap() {
        if tool["name"] == "read_doc" {
            tool["description"] = json!(
                "Read the complete approved source. The MCP result's structuredContent contains source, source_hash and snapshot for a full edit baseline; content[0].text is a hashline presentation. Never guess field locations."
            );
        }
        if tool["name"] == "verify" {
            tool["description"] = json!(
                "Run the shared sealed private gate. Read structuredContent.summary.classification, prediction_matched and original_requirement_satisfied, and isError. isError=true is non-passing. Absence of an execution list never means success. Target values and diagnostics stay withheld."
            );
        }
    }
    catalogue["tools"].as_array_mut().unwrap().push(json!({"name":"seal_prediction","description":"Seal a prediction and rationale against the current captured source before private Verify. The host owns the question, requirement, inputs, actor and environment. Source repairs need a new seal with the original expectation; changed expectations cannot earn promotion. No actual target value is returned.",
        "inputSchema":{"type":"object","properties":{"expected_source_hash":{"type":"string","pattern":"^[a-f0-9]{64}$"},"rationale":{"type":"string","minLength":1,"maxLength":8192},"predicted":{"type":"object","description":"A bounded predicate: {kind: equals, value: JSON}, {kind: number_at_most, value: number}, or {kind: fields, value: map of predicates}."}},"required":["expected_source_hash","predicted","rationale"],"additionalProperties":false}}));
    let seal = catalogue["tools"]
        .as_array_mut()
        .unwrap()
        .last_mut()
        .unwrap();
    seal["inputSchema"]["$defs"] = json!({"predicate":Predicate::schema_definition()});
    seal["inputSchema"]["properties"]["predicted"] = json!({"$ref":"#/$defs/predicate"});
    if with_retained {
        let mut retained_seal = catalogue["tools"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()
            .clone();
        retained_seal["name"] = json!("seal_debugger_prediction");
        retained_seal["description"] = json!(
            "When the trusted state has retained_plan, seal the initial debugger prediction before launch, or the next prediction while paused before Continue. The host fixes both questions; source/runtime/inputs stay bound and prior visible metadata is retained. No target values are returned."
        );
        catalogue["tools"]
            .as_array_mut()
            .unwrap()
            .push(retained_seal);
        catalogue["tools"].as_array_mut().unwrap().push(json!({"name":"continue_debugger_prediction","description":"Execute the pending initial seal or Continue the retained debugger only after its future-stop seal. Returns only the shared gate verdict, never selected target values. No pending seal returns the previous receipt without execution.","inputSchema":{"type":"object","properties":{},"additionalProperties":false}}));
    }
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

pub(super) async fn evidence(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let bound = state
        .private_prediction
        .as_ref()
        .ok_or_else(|| ApiError::unprocessable("No private prediction session"))?;
    let history = bound
        .ledger
        .predictor_history(
            None,
            &bound.actor,
            &bound
                .scenario_keys()
                .map_err(|_| ApiError::unprocessable("Host questions unavailable"))?,
        )
        .map_err(|_| ApiError::unprocessable("Prediction evidence unavailable"))?;
    let scenarios = history
        .attempts
        .iter()
        .map(|attempt| attempt.scenario.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let selected = if scenarios.is_empty() {
        None
    } else {
        Some(
            bound
                .ledger
                .select_predictor_history(&scenarios, &bound.actor)
                .map_err(|_| ApiError::unprocessable("Prediction evidence unavailable"))?,
        )
    };
    Ok(Json(json!({"evidence":selected})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PromotionRequest {
    attempt: String,
}
/// Revalidate the host-owned identities; callers cannot supply or waive them.
pub(super) async fn promote(
    State(state): State<LocalState>,
    Json(body): Json<PromotionRequest>,
) -> ApiResult<Json<Value>> {
    let bound = state
        .private_prediction
        .as_ref()
        .ok_or_else(|| ApiError::unprocessable("No private prediction session"))?;
    let runtime = bound.runtime.as_ref().ok_or_else(|| {
        ApiError::unprocessable(
            "Current certification requires a configured runtime revalidation profile",
        )
    })?;
    let mut session = bound.session.lock().await;
    if bound.is_retired() {
        return Err(ApiError::unprocessable("Private session ended"));
    }
    session
        .refresh_source()
        .await
        .map_err(|_| ApiError::conflict("Captured source changed outside the session"))?;
    let identity = bound.identity.read().unwrap().clone();
    bound
        .inputs
        .as_ref()
        .ok_or_else(|| ApiError::unprocessable("Host input binding unavailable"))?
        .validate(&identity, session.approved_source())
        .map_err(|_| ApiError::conflict("Host-selected inputs changed"))?;
    use sha2::{Digest, Sha256};
    if format!("{:x}", Sha256::digest(session.approved_source())) != identity.source
        || runtime
            .capture()
            .await
            .map_err(|_| ApiError::unprocessable("Runtime identity unavailable"))?
            != identity.environment
        || session
            .source_snapshots()
            .iter()
            .any(|source| source["current"] != true)
    {
        return Err(ApiError::conflict(
            "Source or runtime differs from the sealed identities; promotion refused",
        ));
    }
    let scenario = bound
        .question
        .key()
        .map_err(|_| ApiError::unprocessable("Host question unavailable"))?;
    if !bound
        .ledger
        .predictor_history(Some(&identity), &bound.actor, &[scenario])
        .map_err(|_| ApiError::unprocessable("Prediction history unavailable"))?
        .attempts
        .iter()
        .any(|attempt| attempt.attempt == body.attempt)
    {
        return Err(ApiError::conflict(
            "Attempt belongs to another predictor or host question",
        ));
    }
    if !bound
        .retained_ready(&identity)
        .map_err(|_| ApiError::unprocessable("Retained debugger evidence unavailable"))?
    {
        return Err(ApiError::conflict(
            "Original initial/future debugger questions must pass for the current source before promotion",
        ));
    }
    let summary=bound.ledger.promote(&body.attempt,&identity,Actor{kind:"host".into(),id:"private_identity_revalidation".into()})
        .map_err(|_|ApiError::conflict("Prediction, original requirement, provenance or current identities do not permit promotion"))?;
    Ok(Json(
        json!({"summary":summary,"source_hash":identity.source,"checked_at":super::now_rfc3339(),"scope":"Host source/runtime identity revalidation at this request; before/after runtime fingerprints do not freeze the OS or certify every dependency"}),
    ))
}

/// Completion check for the coach's explicit repair-and-verify contract.
pub(super) async fn completion(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let bound = state
        .private_prediction
        .as_ref()
        .ok_or_else(|| ApiError::unprocessable("No private prediction session"))?;
    let mut session = bound.session.lock().await;
    if session.refresh_source().await.is_err() {
        return Ok(Json(
            json!({"ready":false,"reason":"Captured source is unavailable or changed outside the session"}),
        ));
    }
    let mut identity = bound.identity.read().unwrap().clone();
    use sha2::{Digest, Sha256};
    identity.source = format!("{:x}", Sha256::digest(session.approved_source()));
    if let Some(inputs) = &bound.inputs {
        if inputs
            .validate(&identity, session.approved_source())
            .is_err()
        {
            return Ok(Json(
                json!({"ready":false,"reason":"Host input binding changed"}),
            ));
        }
    }
    if let Some(runtime) = &bound.runtime {
        match runtime.capture().await {
            Ok(environment) => identity.environment = environment,
            Err(_) => {
                return Ok(Json(
                    json!({"ready":false,"reason":"Runtime revalidation unavailable"}),
                ));
            }
        }
    }
    let history = bound
        .ledger
        .history(Some(&identity))
        .map_err(|_| ApiError::unprocessable("Prediction history unavailable"))?;
    let scenario = bound
        .question
        .key()
        .map_err(|_| ApiError::unprocessable("Host question unavailable"))?;
    let actor = serde_json::to_value(&bound.actor).unwrap();
    let latest = history.attempts.iter().rev().find(|attempt| {
        attempt.scenario == scenario
            && history.ordering.iter().any(|entry| {
                entry["event"]["kind"] == "started"
                    && entry["event"]["id"] == attempt.attempt
                    && entry["event"]["actor"] == actor
            })
    });
    let ready = bound.retained_ready(&identity).unwrap_or(false)
        && latest.is_some_and(|attempt| {
            matches!(
                attempt.classification,
                hick_observe::prediction::Classification::FirstAttemptVerified
                    | hick_observe::prediction::Classification::InformedPredictionVerified
                    | hick_observe::prediction::Classification::VerifiedAfterCorrection
                    | hick_observe::prediction::Classification::VerifiedAfterPriorAttempt
            )
        });
    Ok(Json(
        json!({"ready":ready,"comparison":latest,"current_certification":false,"reason":if ready {"Original prediction and requirement pass for the revalidated source/runtime; promotion remains separate"}else{"Original prediction and requirement are not passing for this actor's current source/runtime; continue the requested repair or stop for human review"}}),
    ))
}

#[cfg(test)]
#[path = "private_prediction_tests.rs"]
mod tests;
