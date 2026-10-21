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
use hick_literate::prediction::{OutputEncoding, SealedRequest};
use hick_observe::prediction::{Actor, Identity, Ledger};
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
}
pub(crate) struct Bound {
    pub doc: String,
    path: PathBuf,
    session: tokio::sync::Mutex<EditSession>,
    ledger: Arc<Ledger>,
    identity: Identity,
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
        let identity = binding.request.identity.clone();
        let actor = binding.request.actor.clone();
        let session = EditSession::open_prediction(
            &path,
            binding.ledger.clone(),
            binding.request,
            binding.encoding,
        )
        .await?;
        Ok(Arc::new(Self {
            doc,
            path,
            session: tokio::sync::Mutex::new(session),
            ledger: binding.ledger,
            identity,
            actor,
        }))
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
        if expected
            .as_ref()
            .is_some_and(|hash| hash.trim_start_matches("sha256:") != self.identity.source)
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
                source_hash: Some(self.identity.source.clone()),
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
        || (method == "POST" && path == "/prediction/tool")
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
    Ok(Json(
        json!({"document":bound.doc,"source":bound.path.file_name(),"source_hash":bound.identity.source,
        "actor":bound.actor,"actual_values":"withheld","supported":"captured source tools and sealed Verify", "application_integration":"incomplete"}),
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
    Ok(Json(json!({"ok":result.ok,"text":result.text})))
}
