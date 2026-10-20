//! Window-bound semantic UI RPC. Models cannot select another window or actor.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::{Mutex as AsyncMutex, mpsc, oneshot};

#[derive(Clone, Serialize)]
pub struct Request {
    id: String,
    actor: Value,
    method: String,
    arguments: Value,
}
struct Window {
    send: mpsc::Sender<Request>,
    receive: AsyncMutex<mpsc::Receiver<Request>>,
    pending: Mutex<HashMap<String, oneshot::Sender<Value>>>,
    touched: Mutex<Instant>,
}
#[derive(Default)]
pub struct Hub {
    windows: Mutex<HashMap<String, Arc<Window>>>,
    receipts: Mutex<()>,
}
impl Hub {
    pub(crate) fn ensure_window(&self, id: &str) -> anyhow::Result<()> {
        self.window(id).map(|_| ())
    }
    fn window(&self, id: &str) -> anyhow::Result<Arc<Window>> {
        let window = self
            .windows
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Originating UI window is unavailable; reopen it and send a new turn"
                )
            })?;
        anyhow::ensure!(
            window.touched.lock().unwrap().elapsed() < Duration::from_secs(90),
            "Originating UI window stopped responding"
        );
        Ok(window)
    }

    pub async fn call(
        &self,
        window: &str,
        actor: &str,
        method: &str,
        arguments: Value,
    ) -> anyhow::Result<Value> {
        anyhow::ensure!(
            matches!(method, "ui_observe" | "ui_act" | "ui_cancel"),
            "Unknown UI method"
        );
        anyhow::ensure!(
            arguments.is_object() && serde_json::to_vec(&arguments)?.len() <= 65_536,
            "UI arguments must be an object smaller than 64 KiB"
        );
        let window = self.window(window)?;
        let id = format!("{:016x}{:016x}", super::rand_id(), super::rand_id());
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = window.pending.lock().unwrap();
            anyhow::ensure!(pending.len() < 32, "Too many outstanding UI requests");
            pending.insert(id.clone(), tx);
        }
        // Drop cleanup also runs when the agent task is cancelled during this await.
        struct Pending(Arc<Window>, String, Option<(String, String)>);
        impl Drop for Pending {
            fn drop(&mut self) {
                let abandoned = self.0.pending.lock().unwrap().remove(&self.1).is_some();
                if abandoned && let Some((actor, request_id)) = &self.2 {
                    // Notification only; this does not assert rollback.
                    let _ = self.0.send.try_send(Request {
                        id: String::new(),
                        actor: json!({"id":actor,"kind":"agent"}),
                        method: "ui_cancel".into(),
                        arguments: json!({"request_id":request_id}),
                    });
                }
            }
        }
        let cancellation = (method == "ui_act")
            .then(|| {
                arguments["request_id"]
                    .as_str()
                    .map(|id| (actor.to_owned(), id.to_owned()))
            })
            .flatten();
        let _pending = Pending(window.clone(), id.clone(), cancellation);
        window
            .send
            .try_send(Request {
                id,
                actor: json!({"id":actor,"kind":"agent"}),
                method: method.into(),
                arguments,
            })
            .map_err(|_| anyhow::anyhow!("UI window request queue is full or closed"))?;
        match tokio::time::timeout(Duration::from_secs(35), rx).await {
            Ok(Ok(result)) => Ok(result),
            _ => anyhow::bail!(
                "UI result unavailable; an action may still complete. Inspect current state and retry only with the same request_id."
            ),
        }
    }
}

async fn open(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let id = format!("{:016x}{:016x}", super::rand_id(), super::rand_id());
    let (send, receive) = mpsc::channel(32);
    let mut windows = state.ui_actions.windows.lock().unwrap();
    windows.retain(|_, window| window.touched.lock().unwrap().elapsed() < Duration::from_secs(90));
    if windows.len() >= 32 {
        return Err(ApiError::unavailable("UI window limit reached"));
    }
    windows.insert(
        id.clone(),
        Arc::new(Window {
            send,
            receive: AsyncMutex::new(receive),
            pending: Mutex::new(HashMap::new()),
            touched: Mutex::new(Instant::now()),
        }),
    );
    // Workspace identity is opaque and scoped to this local host, not a file path.
    Ok(Json(
        json!({"window":id,"workspace":format!("{:x}", Sha256::digest(state.index.root().to_string_lossy().as_bytes()))}),
    ))
}
async fn poll(State(state): State<LocalState>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    let window = state
        .ui_actions
        .window(&id)
        .map_err(|e| ApiError::unavailable(e.to_string()))?;
    *window.touched.lock().unwrap() = Instant::now();
    let mut receive = window
        .receive
        .try_lock()
        .map_err(|_| ApiError::conflict("A poll is already active for this window"))?;
    let request = tokio::time::timeout(Duration::from_secs(20), receive.recv())
        .await
        .ok()
        .flatten();
    // Cancelled/timed-out agent requests never start later from a queued message.
    let request = request.filter(|request| {
        (request.id.is_empty() && request.method == "ui_cancel")
            || window.pending.lock().unwrap().contains_key(&request.id)
    });
    Ok(Json(json!({"request":request})))
}
#[derive(Deserialize)]
struct Reply {
    id: String,
    result: Value,
}
async fn reply(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Json(body): Json<Reply>,
) -> ApiResult<Json<Value>> {
    let window = state
        .ui_actions
        .window(&id)
        .map_err(|e| ApiError::unavailable(e.to_string()))?;
    let tx = window
        .pending
        .lock()
        .unwrap()
        .remove(&body.id)
        .ok_or_else(|| {
            ApiError::conflict("UI request is no longer pending; no completion assumed")
        })?;
    tx.send(body.result)
        .map_err(|_| ApiError::conflict("UI caller disconnected; result not delivered"))?;
    Ok(Json(json!({"delivered":true})))
}
async fn receipt(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> ApiResult<Json<Value>> {
    state
        .ui_actions
        .window(&id)
        .map_err(|e| ApiError::unavailable(e.to_string()))?;
    let stage = body["stage"]
        .as_str()
        .filter(|s| matches!(*s, "requested" | "result"))
        .ok_or_else(|| ApiError::bad_request("Invalid receipt stage"))?;
    let workspace = format!(
        "{:x}",
        Sha256::digest(state.index.root().to_string_lossy().as_bytes())
    );
    if body["scope"]["window"] != id
        || body["scope"]["workspace"] != workspace
        || !body["request_id"]
            .as_str()
            .is_some_and(|s| !s.is_empty() && s.len() <= 128)
        || !matches!(body["actor"]["kind"].as_str(), Some("agent" | "human"))
    {
        return Err(ApiError::bad_request(
            "Receipt scope/request/actor mismatch",
        ));
    }
    let raw = serde_json::to_vec(&body).map_err(|e| ApiError::bad_request(e.to_string()))?;
    if raw.len() > 262_144 {
        return Err(ApiError::bad_request("Receipt exceeds 256 KiB"));
    }
    let directory = hickory_workspace::WorkspaceStore::for_project(state.index.root())?
        .dir()
        .join("ui-actions")
        .join(id);
    let _guard = state.ui_actions.receipts.lock().unwrap();
    std::fs::create_dir_all(&directory).map_err(|e| ApiError::internal(e.to_string()))?;
    let key = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&json!([body["actor"], body["request_id"]])).unwrap())
    );
    let path = directory.join(format!("{key}.{stage}.json"));
    if stage == "result" {
        let requested: Value = serde_json::from_slice(
            &std::fs::read(directory.join(format!("{key}.requested.json")))
                .map_err(|_| ApiError::conflict("Result has no retained intent"))?,
        )
        .map_err(|_| ApiError::conflict("Retained intent is unreadable"))?;
        if ["scope", "actor", "request_id", "action", "arguments"]
            .iter()
            .any(|key| requested[*key] != body[*key])
        {
            return Err(ApiError::conflict("Result does not match retained intent"));
        }
    }
    // Immutable request/result identities: a retry may retain identical bytes only.
    if path.exists() {
        if std::fs::read(&path).map_err(|e| ApiError::internal(e.to_string()))? != raw {
            return Err(ApiError::conflict(
                "Receipt identity already has different bytes",
            ));
        }
    } else {
        super::store::write_atomic(&path, &raw).map_err(|e| ApiError::internal(e.to_string()))?;
    }
    Ok(Json(json!({"retained":true})))
}
async fn close(State(state): State<LocalState>, Path(id): Path<String>) -> Json<Value> {
    if let Some(window) = state.ui_actions.windows.lock().unwrap().remove(&id) {
        for (_, sender) in window.pending.lock().unwrap().drain() {
            let _ = sender.send(json!({"status":"unavailable","reason":"Originating window closed; side effects may have occurred"}));
        }
    }
    state.fleet_pairings.close_window(&id);
    Json(json!({"closed":true}))
}
pub fn routes() -> Router<LocalState> {
    Router::new()
        .route("/v1/ui/windows", post(open))
        .route("/v1/ui/windows/{id}", axum::routing::delete(close))
        .route("/v1/ui/windows/{id}/requests", get(poll))
        .route("/v1/ui/windows/{id}/reply", post(reply))
        .route("/v1/ui/windows/{id}/receipts", post(receipt))
}

pub fn catalogue() -> Vec<Value> {
    vec![
        json!({"name":"ui_observe","description":"Read registered controls in the originating Hickory window. Start with parent root to discover disclosure groups. Scrolling never expands model context. Coverage is partial and explicitly reported.","inputSchema":{"type":"object","properties":{"parent":{"type":"string"},"limit":{"type":"integer","minimum":1,"maximum":64},"offset":{"type":"integer","minimum":0}},"additionalProperties":false}}),
        json!({"name":"ui_act","description":"Invoke a disclosed current semantic UI action as this agent. Use the same request_id when retrying. A UI handler returning is not proof of runtime verification. Human-only approvals are unavailable.","inputSchema":{"type":"object","properties":{"handle":{"type":"string"},"arguments":{"type":"object"},"request_id":{"type":"string","maxLength":128}},"required":["handle","arguments","request_id"],"additionalProperties":false}}),
        json!({"name":"ui_cancel","description":"Request cancellation of an owned in-flight UI invocation. This does not assert rollback or cancel an already returned host job.","inputSchema":{"type":"object","properties":{"request_id":{"type":"string"}},"required":["request_id"],"additionalProperties":false}}),
    ]
}

pub fn tool_result(mut result: Value) -> Value {
    let image = match super::ui_images::result_image(&result) {
        Ok(image) => image,
        Err(error) => {
            return json!({"content":[{"type":"text","text":json!({"status":"error","reason":format!("Image capture refused: {error:#}")}).to_string()}],"isError":true});
        }
    };
    if image.is_some() {
        result["outcome"]["value"]["ui_image"]
            .as_object_mut()
            .unwrap()
            .remove("data");
    }
    let error = matches!(
        result["status"].as_str(),
        Some("error" | "forbidden" | "invalid" | "conflict" | "unavailable" | "stale" | "disabled")
    ) || result["outcome"]["status"] == "unavailable";
    let mut content = vec![json!({"type":"text","text":result.to_string()})];
    if let Some(image) = image {
        content.push(image);
    }
    json!({"content":content,"isError":error})
}

/// Bounded, local-only retained debugger observations for coach reconciliation.
/// These are host receipt results, never statements parsed from model prose.
pub fn retained_debug_witnesses(state: &LocalState, source_hash: &str) -> Vec<Value> {
    let Ok(store) = hickory_workspace::WorkspaceStore::for_project(state.index.root()) else {
        return vec![];
    };
    let root = store.dir().join("ui-actions");
    if !root.exists() {
        return vec![];
    }
    if !root.canonicalize().is_ok_and(|path| {
        store
            .dir()
            .canonicalize()
            .is_ok_and(|store| path.starts_with(store))
    }) {
        return vec![];
    }
    let Ok(files) = hick_context_store::Files::new(&root, 262_144) else {
        return vec![];
    };
    let mut result = vec![];
    let mut examined = 0;
    let Ok(windows) = std::fs::read_dir(&root) else {
        return result;
    };
    for window in windows.flatten().take(32) {
        let name = window.file_name().to_string_lossy().into_owned();
        if name.len() != 32 || !name.bytes().all(|b| b.is_ascii_hexdigit()) {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(window.path()) else {
            continue;
        };
        for entry in entries.flatten().take(8192) {
            let file = entry.file_name().to_string_lossy().into_owned();
            if !file.ends_with(".result.json") {
                continue;
            }
            examined += 1;
            if examined > 1024 {
                return result;
            }
            let Ok(raw) = files.read(&format!("{name}/{file}")) else {
                continue;
            };
            let Ok(receipt) = serde_json::from_str::<Value>(&raw) else {
                continue;
            };
            if receipt["stage"] != "result"
                || receipt["action"]["label"] != "Read debugger state"
                || receipt["result"]["status"] != "result"
                || receipt["result"]["outcome"]["status"] != "completed"
            {
                continue;
            }
            let evidence = &receipt["result"]["outcome"]["value"]["evidence"];
            if !["run", "stop"].iter().all(|key| {
                evidence["scope"][key]
                    .as_str()
                    .is_some_and(|id| !id.is_empty())
            }) {
                continue;
            }
            if evidence["source"]["uri"]
                .as_str()
                .map(|s| s.trim_start_matches("hick:///"))
                != Some("deadline-spec.md")
                || evidence["source"]["revision"]
                    .as_str()
                    .map(|s| s.trim_start_matches("sha256:"))
                    != Some(source_hash)
            {
                continue;
            }
            let Some(annotations) = evidence["annotations"].as_array() else {
                continue;
            };
            if !annotations.iter().any(|a| a["payload"]["kind"] == "paused")
                || !annotations
                    .iter()
                    .any(|a| a["payload"]["kind"] == "breakpoint_bound")
            {
                continue;
            }
            result.push(json!({"run":evidence["scope"]["run"],"stop":evidence["scope"]["stop"],"evidence":annotations.iter().filter(|a|matches!(a["payload"]["kind"].as_str(),Some("paused"|"breakpoint_bound"))).take(64).map(|a|a["evidence"].clone()).collect::<Vec<_>>(),"source":evidence["source"],"request_id":receipt["request_id"],"actor":receipt["actor"],"coverage":"Retained debugger observation, not a currently paused session"}));
            if result.len() >= 16 {
                return result;
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn state() -> (tempfile::TempDir, LocalState) {
        let directory = tempfile::tempdir().unwrap();
        let prepared = super::super::prepare(super::super::ServeOptions {
            target: directory.path().to_path_buf(),
            port: 0,
            params: vec![],
            executor: crate::ExecutorChoice::Local,
            key_store_path: None,
            ui_settings_path: None,
        })
        .await
        .unwrap();
        (directory, prepared.state)
    }

    #[tokio::test]
    async fn retained_coach_debug_witness_requires_completed_matching_source_and_stop() {
        let (_directory, state) = state().await;
        let store = hickory_workspace::WorkspaceStore::for_project(state.index.root()).unwrap();
        let directory = store.dir().join("ui-actions").join("a".repeat(32));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("read.result.json");
        let mut receipt = json!({"stage":"result","request_id":"read","actor":{"id":"python:turn","kind":"agent"},"action":{"label":"Read debugger state"},"result":{"status":"result","outcome":{"status":"completed","value":{"evidence":{"source":{"uri":"deadline-spec.md","revision":"seed-hash"},"scope":{"run":"run","stop":"stop"},"annotations":[{"evidence":"pause","payload":{"kind":"paused"}},{"evidence":"bound","payload":{"kind":"breakpoint_bound"}}]}}}}});
        std::fs::write(&path, receipt.to_string()).unwrap();
        assert_eq!(retained_debug_witnesses(&state, "seed-hash").len(), 1);
        assert!(retained_debug_witnesses(&state, "different-source").is_empty());
        receipt["result"]["outcome"]["status"] = json!("unavailable");
        std::fs::write(&path, receipt.to_string()).unwrap();
        assert!(retained_debug_witnesses(&state, "seed-hash").is_empty());
        receipt["result"]["outcome"]["status"] = json!("completed");
        receipt["result"]["outcome"]["value"]["evidence"]["scope"]["stop"] = Value::Null;
        std::fs::write(&path, receipt.to_string()).unwrap();
        assert!(retained_debug_witnesses(&state, "seed-hash").is_empty());
    }

    // Guarantee: docs/guarantees/agent/ui-action-dispatch.md
    #[tokio::test]
    async fn window_broker_preserves_actor_and_only_accepts_the_matching_reply() {
        let (_directory, state) = state().await;
        let window = open(State(state.clone())).await.unwrap().0["window"]
            .as_str()
            .unwrap()
            .to_owned();
        let second = open(State(state.clone())).await.unwrap().0["window"]
            .as_str()
            .unwrap()
            .to_owned();
        let hub = state.ui_actions.clone();
        let id = window.clone();
        let task = tokio::spawn(async move {
            hub.call(
                &id,
                "python:actual-turn",
                "ui_observe",
                json!({"parent":"root"}),
            )
            .await
        });
        let frame = poll(State(state.clone()), Path(window.clone()))
            .await
            .unwrap()
            .0["request"]
            .clone();
        assert_eq!(
            frame["actor"],
            json!({"id":"python:actual-turn","kind":"agent"})
        );
        assert_eq!(frame["arguments"], json!({"parent":"root"}));
        let id = frame["id"].as_str().unwrap().to_owned();
        assert!(
            reply(
                State(state.clone()),
                Path(second),
                Json(Reply {
                    id: id.clone(),
                    result: json!("forged")
                })
            )
            .await
            .is_err()
        );
        let _ = reply(
            State(state.clone()),
            Path(window.clone()),
            Json(Reply {
                id: id.clone(),
                result: json!({"groups":["rail"],"coverage":"partial"}),
            }),
        )
        .await
        .unwrap();
        assert_eq!(task.await.unwrap().unwrap()["groups"], json!(["rail"]));
        assert!(
            reply(
                State(state),
                Path(window),
                Json(Reply {
                    id,
                    result: json!({})
                })
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn closed_window_cannot_be_retargeted_and_pending_calls_do_not_claim_completion() {
        let (_directory, state) = state().await;
        let window = open(State(state.clone())).await.unwrap().0["window"]
            .as_str()
            .unwrap()
            .to_owned();
        let hub = state.ui_actions.clone();
        let id = window.clone();
        let task = tokio::spawn(async move {
            hub.call(
                &id,
                "agent",
                "ui_act",
                json!({"handle":"action:x","request_id":"r","arguments":{}}),
            )
            .await
        });
        let _frame = poll(State(state.clone()), Path(window.clone()))
            .await
            .unwrap();
        let _ = close(State(state.clone()), Path(window.clone())).await;
        assert_eq!(task.await.unwrap().unwrap()["status"], "unavailable");
        assert!(
            state
                .ui_actions
                .call(&window, "agent", "ui_observe", json!({}))
                .await
                .is_err()
        );
        assert!(
            state
                .ui_actions
                .call("another", "agent", "ui_observe", json!({}))
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn cancelled_caller_cleans_pending_requests_and_bounds_arguments() {
        let (_directory, state) = state().await;
        let window = open(State(state.clone())).await.unwrap().0["window"]
            .as_str()
            .unwrap()
            .to_owned();
        let hub = state.ui_actions.clone();
        let id = window.clone();
        let task =
            tokio::spawn(async move { hub.call(&id, "agent", "ui_observe", json!({})).await });
        let frame = poll(State(state.clone()), Path(window.clone()))
            .await
            .unwrap()
            .0["request"]
            .clone();
        task.abort();
        let _ = task.await;
        assert!(
            state
                .ui_actions
                .window(&window)
                .unwrap()
                .pending
                .lock()
                .unwrap()
                .is_empty()
        );
        assert!(
            reply(
                State(state.clone()),
                Path(window.clone()),
                Json(Reply {
                    id: frame["id"].as_str().unwrap().into(),
                    result: json!({})
                })
            )
            .await
            .is_err()
        );
        assert!(
            state
                .ui_actions
                .call(&window, "agent", "arbitrary_method", json!({}))
                .await
                .is_err()
        );
        assert!(
            state
                .ui_actions
                .call(
                    &window,
                    "agent",
                    "ui_act",
                    json!({"text":"x".repeat(70_000)})
                )
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn abandoned_action_requests_cancellation_for_the_original_actor() {
        let (_directory, state) = state().await;
        let window = open(State(state.clone())).await.unwrap().0["window"]
            .as_str()
            .unwrap()
            .to_owned();
        let hub = state.ui_actions.clone();
        let id = window.clone();
        let task = tokio::spawn(async move {
            hub.call(
                &id,
                "python:cancelled-turn",
                "ui_act",
                json!({"handle":"action:current","arguments":{},"request_id":"original-operation"}),
            )
            .await
        });
        let _frame = poll(State(state.clone()), Path(window.clone()))
            .await
            .unwrap();
        task.abort();
        let _ = task.await;
        let cancellation =
            poll(State(state.clone()), Path(window)).await.unwrap().0["request"].clone();
        assert_eq!(cancellation["method"], "ui_cancel");
        assert_eq!(cancellation["id"], "");
        assert_eq!(
            cancellation["actor"],
            json!({"id":"python:cancelled-turn","kind":"agent"})
        );
        assert_eq!(
            cancellation["arguments"],
            json!({"request_id":"original-operation"})
        );
    }

    #[tokio::test]
    async fn receipts_are_retained_immutably_and_scoped_to_the_window() {
        let (_directory, state) = state().await;
        let scope = open(State(state.clone())).await.unwrap().0;
        let window = scope["window"].as_str().unwrap().to_owned();
        let body = json!({"stage":"requested","scope":scope,"request_id":"r","actor":{"id":"python:turn","kind":"agent"},"arguments":{},"action":{"label":"Run"}});
        let _ = receipt(
            State(state.clone()),
            Path(window.clone()),
            Json(body.clone()),
        )
        .await
        .unwrap();
        let _ = receipt(
            State(state.clone()),
            Path(window.clone()),
            Json(body.clone()),
        )
        .await
        .unwrap();
        let mut changed = body.clone();
        changed["arguments"] = json!({"different":true});
        assert!(
            receipt(State(state.clone()), Path(window.clone()), Json(changed))
                .await
                .is_err()
        );
        let mut changed = body;
        changed["scope"]["window"] = json!("other");
        assert!(
            receipt(State(state.clone()), Path(window.clone()), Json(changed))
                .await
                .is_err()
        );
        let directory = hickory_workspace::WorkspaceStore::for_project(state.index.root())
            .unwrap()
            .dir()
            .join("ui-actions")
            .join(window);
        assert_eq!(std::fs::read_dir(directory).unwrap().count(), 1);
    }
}
