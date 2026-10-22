use super::{Attach, Attached, Endpoint, LEASE, PROTOCOL, WORKER, directory, watch::Coordinator};
use crate::serve::{
    self, LocalState, ServeOptions,
    store::{DocIndex, FileDocStore},
};
use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Path, Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Instant};
use tokio::sync::Mutex;
use tower::ServiceExt;

type Api<T> = Result<T, (StatusCode, Json<Value>)>;
fn error(e: impl std::fmt::Display) -> (StatusCode, Json<Value>) {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(json!({"error":e.to_string()})),
    )
}

struct Client {
    root: PathBuf,
    router: Router,
    state: LocalState,
    slot: String,
    seen: Instant,
    startup: super::startup::Startup,
}
struct Engine {
    token: String,
    clients: Mutex<HashMap<String, Client>>,
    workspaces: Mutex<HashMap<PathBuf, LocalState>>,
    attaching: Mutex<()>,
    writes: Arc<Mutex<()>>,
    store: Arc<FileDocStore>,
    rooms: Arc<hickory_collab::RoomRegistry>,
    watch: Coordinator,
    subscriptions: Arc<std::sync::RwLock<HashMap<String, Attach>>>,
    workers: std::sync::atomic::AtomicUsize,
    mcp: Mutex<HashMap<String, crate::mcp::Server>>,
    locks: Mutex<HashMap<PathBuf, crate::up::DirectoryLock>>,
    ended_clients: Mutex<std::collections::HashSet<String>>,
}

pub fn run_argv(args: &[String]) -> Option<Result<()>> {
    if args.first().map(String::as_str) != Some("__engine") {
        return None;
    }
    Some((|| {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_stack_size(16 * 1024 * 1024)
            .build()?;
        runtime.block_on(run())
    })())
}

async fn run() -> Result<()> {
    let dir = directory()?;
    let owner = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.join("owner.lock"))?;
    if owner.try_lock_exclusive().is_err() {
        return Ok(());
    }
    let token = super::secret();
    let writes = Arc::new(Mutex::new(()));
    let index = Arc::new(DocIndex::shared(&dir)?);
    let store = FileDocStore::new(index);
    let rooms = Arc::new(hickory_collab::RoomRegistry::with_writes(
        store.clone(),
        writes.clone(),
    ));
    let subscriptions = Arc::new(std::sync::RwLock::new(HashMap::new()));
    let watch = Coordinator::start(writes.clone(), subscriptions.clone())?;
    let engine = Arc::new(Engine {
        token: token.clone(),
        clients: Mutex::new(HashMap::new()),
        workspaces: Mutex::new(HashMap::new()),
        attaching: Mutex::new(()),
        writes,
        store,
        rooms,
        watch,
        subscriptions,
        workers: 0.into(),
        mcp: Mutex::new(HashMap::new()),
        locks: Mutex::new(HashMap::new()),
        ended_clients: Mutex::new(Default::default()),
    });
    let router = Router::new()
        .route(
            "/health",
            get(|| async {
                Json(json!({"protocol":PROTOCOL,"capabilities":[super::PREDICTION_PROTOCOL]}))
            }),
        )
        .route("/prediction-policy", get(prediction_policy))
        .route("/private/end", post(end_private))
        .route("/attach", post(attach))
        .route("/command", post(command))
        .route("/tool", post(tool))
        .layer(axum::extract::DefaultBodyLimit::max(64 * 1024 * 1024))
        .route("/clients/{id}/lease", post(lease).delete(detach))
        // Dispatch routes the stripped URI again. Named outer captures would
        // survive in request extensions and be appended to every inner Path.
        // A fallback adds no captures and preserves WebSocket upgrade state.
        .fallback(dispatch)
        .layer(axum::middleware::from_fn_with_state(
            engine.clone(),
            authenticate,
        ))
        .with_state(engine.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let endpoint = Endpoint {
        url: format!("http://{}", listener.local_addr()?),
        token,
        protocol: PROTOCOL,
        pid: std::process::id(),
    };
    serve::store::write_atomic(&dir.join("endpoint.json"), &serde_json::to_vec(&endpoint)?)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            dir.join("endpoint.json"),
            std::fs::Permissions::from_mode(0o600),
        )?;
    }
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let cleanup = engine.clone();
    tokio::spawn(async move {
        let mut idle_since = Instant::now();
        loop {
            tokio::time::sleep(super::HEARTBEAT).await;
            let mut clients = cleanup.clients.lock().await;
            clients.retain(|_, c| c.seen.elapsed() < LEASE);
            cleanup
                .subscriptions
                .write()
                .unwrap()
                .retain(|id, _| clients.contains_key(id));
            let busy = cleanup.workers.load(std::sync::atomic::Ordering::SeqCst) > 0
                || cleanup.workspaces.lock().await.values().any(|state| {
                    state
                        .private_prediction
                        .as_ref()
                        .is_some_and(|bound| bound.investigation_busy())
                        || state
                            .runs
                            .lock()
                            .unwrap()
                            .values()
                            .any(|r| r.status == "running")
                        || state
                            .terminals
                            .summaries()
                            .iter()
                            .any(|t| t.exit_code.is_none())
                });
            if clients.is_empty() && !busy {
                if idle_since.elapsed() > LEASE {
                    let _ = stop.send(());
                    break;
                }
            } else {
                idle_since = Instant::now();
            }
        }
    });
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        let _ = stopped.await;
    })
    .await?;
    for state in engine.workspaces.lock().await.values() {
        if let Some(bound) = &state.private_prediction {
            bound.shutdown_retained().await;
        }
    }
    engine.watch.woven.lock().await.release_read_only();
    // Keep owner.lock's inode: only the kernel decides when ownership ends.
    let _ = std::fs::remove_file(dir.join("endpoint.json"));
    Ok(())
}

async fn authenticate(
    State(engine): State<Arc<Engine>>,
    req: Request,
    next: axum::middleware::Next,
) -> Response {
    let expected = format!("Bearer {}", engine.token);
    if req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        != Some(expected.as_str())
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    next.run(req).await
}

async fn prediction_policy(State(engine): State<Arc<Engine>>) -> Json<Value> {
    let active = super::private_policy::active().unwrap_or(true)
        || engine
            .workspaces
            .lock()
            .await
            .values()
            .any(|state| state.private_prediction.is_some());
    Json(json!({"active":active,"scope":"engine-managed-context","version":1}))
}

async fn attach(
    State(engine): State<Arc<Engine>>,
    Json(mut opts): Json<Attach>,
) -> Api<Json<Attached>> {
    let _attach = engine.attaching.lock().await;
    if engine.ended_clients.lock().await.contains(&opts.client) {
        return Err(error(
            "This private client ended; open a new window for a fresh context",
        ));
    }
    opts.target = opts.target.canonicalize().map_err(error)?;
    let root = if opts.target.is_dir() {
        opts.target.clone()
    } else {
        opts.target.parent().unwrap().to_path_buf()
    };
    if let Some(callback) = &opts.callback {
        let url: reqwest::Url = callback.parse().map_err(error)?;
        if url.scheme() != "http" || url.host_str() != Some("127.0.0.1") || url.port().is_none() {
            return Err(error("window callback must use loopback"));
        }
    }
    // Restore only host-retained approved bytes, never adopt arbitrary disk state.
    // Existing live workspaces continue to inherit their current binding below.
    if opts.prediction.is_none() && !engine.workspaces.lock().await.contains_key(&root) {
        if let Some(recovery) = super::private_policy::binding(&root).map_err(error)? {
            if !opts.params.is_empty() {
                return Err(error("Private recovery refuses uncaptured parameters"));
            }
            let (path, profile) = recovery.restore(&root, opts.executor).map_err(error)?;
            opts.target = path;
            opts.prediction = Some(profile);
        }
    }
    let requested_prediction = opts
        .prediction
        .as_ref()
        .map(|profile| profile.fingerprint())
        .transpose()
        .map_err(error)?;
    if opts.prediction.is_some()
        && (!opts.target.is_file()
            || opts.run
            || engine.workers.load(std::sync::atomic::Ordering::SeqCst) > 0)
    {
        return Err(error(
            "Private prediction startup needs an exact source file, no ordinary run, and no active managed command",
        ));
    }
    let mut workspaces = engine.workspaces.lock().await;
    if opts.prediction.is_none()
        && super::private_policy::overlaps(&root).map_err(error)?
        && !workspaces
            .get(&root)
            .is_some_and(|state| state.private_prediction.is_some())
    {
        return Err(error(
            "This root retains private prediction quarantine after restart; a trusted host must restore its private binding",
        ));
    }
    if let Some(profile) = &opts.prediction {
        let store = profile.ledger_root.canonicalize().map_err(error)?;
        if workspaces.iter().any(|(other, state)| {
            state.private_prediction.is_none()
                && (store.starts_with(other) || other.starts_with(&store))
        }) {
            return Err(error(
                "Private evidence storage overlaps an ordinary workspace; choose isolated host storage",
            ));
        }
    }
    for (other, state) in workspaces.iter() {
        if other != &root
            && (root.starts_with(other) || other.starts_with(&root))
            && (state.private_prediction.is_some() || opts.prediction.is_some())
        {
            return Err(error(
                "Overlapping ordinary/private workspace attachments are unavailable",
            ));
        }
    }
    let mut state = if let Some(state) = workspaces.get(&root) {
        if let Some(bound) = &state.private_prediction {
            if bound.is_retired() {
                return Err(error(
                    "Private practice is ending; finish End before opening a fresh window",
                ));
            }
            if opts.run
                || requested_prediction
                    .as_ref()
                    .is_some_and(|requested| Some(requested) != bound.startup_fingerprint.get())
            {
                return Err(error(
                    "This workspace retains a different private prediction startup binding",
                ));
            }
        } else if requested_prediction.is_some() {
            return Err(error(
                "An ordinary workspace cannot be converted in place; start a fresh private workspace",
            ));
        }

        if state.executor != opts.executor || *state.params != opts.params {
            return Err(error(
                "This workspace is using different execution settings. Use the same executor and parameters as its existing session.",
            ));
        }
        state.clone()
    } else {
        let private_binding = opts
            .prediction
            .as_ref()
            .map(|profile| profile.binding(&root))
            .transpose()
            .map_err(error)?;
        let directory_lock = crate::up::DirectoryLock::acquire(&root).map_err(error)?;
        let options = ServeOptions {
            target: if opts.prediction.is_some() {
                opts.target.clone()
            } else {
                root.clone()
            },
            port: 0,
            params: opts.params.clone(),
            executor: opts.executor,
            key_store_path: opts.key_store_path.clone(),
            ui_settings_path: opts.ui_settings_path.clone(),
        };
        let mut state = match &opts.prediction {
            Some(_) => {
                serve::prepare_with_prediction(
                    options,
                    private_binding.expect("validated private binding"),
                )
                .await
            }
            None => serve::prepare(options).await,
        }
        .map_err(error)?
        .state;
        if let Some(bound) = &state.private_prediction {
            super::private_policy::retain(&root, &opts.prediction.as_ref().unwrap().ledger_root)
                .map_err(error)?;
            bound
                .install_recovery(
                    opts.prediction.clone().expect("private startup"),
                    opts.executor,
                )
                .map_err(error)?;
            let admission=async {
                super::private_policy::drain().await?;
                for ordinary in workspaces.values().filter(|state|state.private_prediction.is_none()) {
                    anyhow::ensure!(!ordinary.runs.lock().unwrap().values().any(|run|run.status=="running") && !ordinary.terminals.summaries().iter().any(|terminal|terminal.exit_code.is_none()) && !ordinary.agent.ordinary_activity().await,"Stop ordinary executions, terminals and agent connections before opening private practice");
                }
                Ok::<(),anyhow::Error>(())
            }.await;
            if let Err(failure) = admission {
                super::private_policy::end(&root).map_err(error)?;
                return Err(error(failure));
            }
            bound
                .startup_fingerprint
                .set(requested_prediction.clone().unwrap())
                .map_err(|_| error("Private startup binding already installed"))?;
        } else {
            state.index = Arc::new(DocIndex::shared_from(&state.index));
            state.store = engine.store.clone();
            state.store.register(&state.index);
            state.rooms = engine.rooms.clone();
            state.engine_woven = Some(engine.watch.woven.clone());
        }
        state.writes = engine.writes.clone();
        engine
            .locks
            .lock()
            .await
            .insert(root.clone(), directory_lock);
        workspaces.insert(root.clone(), state.clone());
        state
    };
    drop(workspaces);
    // Settings supplied by a desktop client also apply if a CLI started first.
    if let Some(path) = &opts.key_store_path {
        *state.keys.store.write().unwrap() = crate::engine::load_keys(path).map_err(error)?;
        // Replace on every view of this workspace, including future attaches.
        let mut workspaces = engine.workspaces.lock().await;
        let cached = workspaces.get_mut(&root).unwrap();
        let keys = Arc::new(serve::KeySettings {
            store: std::sync::RwLock::new(crate::engine::load_keys(path).map_err(error)?),
            path: Some(path.clone()),
        });
        cached.keys = keys.clone();
        state.keys = keys;
    }
    if let Some(path) = &opts.ui_settings_path {
        let ui = Arc::new(serve::UiSettings {
            store: std::sync::RwLock::new(serve::UiStore::load(path).map_err(error)?),
            path: Some(path.clone()),
        });
        engine.workspaces.lock().await.get_mut(&root).unwrap().ui = ui.clone();
        state.ui = ui;
    }
    // A runtime bundled by the attaching app overrides a runtime this engine
    // discovered on its own (or did not), so an app that opens a workspace
    // served by a CLI-started engine still gets its own Python agent.
    if let Some(path) = &opts.python_runtime_path {
        engine
            .workspaces
            .lock()
            .await
            .get_mut(&root)
            .unwrap()
            .python_runtime_path = Some(path.clone());
        state.python_runtime_path = Some(path.clone());
    }
    let clients = engine.clients.lock().await;
    let slot = if let Some(client) = clients.get(&opts.client) {
        client.slot.clone()
    } else {
        (0..)
            .find(|slot| {
                !clients
                    .values()
                    .any(|c| c.root == root && c.slot == slot.to_string())
            })
            .unwrap()
            .to_string()
    };
    state.window_slot = Some(slot.clone());
    state.shell = Arc::new(std::sync::Mutex::new(opts.callback.as_ref().map(|url| {
        super::proxy::remote_shell(url.clone(), opts.callback_token.clone())
    })));
    drop(clients);
    let log_offset = std::fs::metadata(directory().map_err(error)?.join("engine.log"))
        .map(|m| m.len())
        .unwrap_or(0);
    let startup = if state.private_prediction.is_some() {
        super::startup::Startup::complete()
    } else {
        engine
            .subscriptions
            .write()
            .unwrap()
            .insert(opts.client.clone(), opts.clone());
        engine
            .watch
            .register(state.clone(), opts.clone())
            .await
            .map_err(error)?
    };
    // This is a window property, even when file and folder views share an engine.
    state.folder_open = opts.target.is_dir();
    let router = serve::router(state.clone());
    let state_prediction = state
        .private_prediction
        .as_ref()
        .and_then(|bound| bound.startup_fingerprint.get())
        .cloned();
    engine.clients.lock().await.insert(
        opts.client,
        Client {
            root,
            router,
            state,
            slot: slot.clone(),
            seen: Instant::now(),
            startup,
        },
    );
    Ok(Json(Attached {
        slot,
        log_offset,
        prediction: requested_prediction.or_else(|| state_prediction),
    }))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EndPrivate {
    client: String,
}
/// Trusted host lifecycle transport, absent from model/MCP/tool catalogues.
async fn end_private(
    State(engine): State<Arc<Engine>>,
    Json(body): Json<EndPrivate>,
) -> Api<Json<Value>> {
    let _attach = engine.attaching.lock().await;
    if engine.ended_clients.lock().await.contains(&body.client) {
        return Ok(Json(
            json!({"ended":true,"history_retained":true,"root_still_private":true,"context_reusable":false}),
        ));
    }
    let clients = engine.clients.lock().await;
    let client = clients
        .get(&body.client)
        .ok_or_else(|| error("Private client unavailable"))?;
    let root = client.root.clone();
    if clients.iter().any(|(id, client)| {
        id != &body.client
            && client.root == root
            && !client
                .state
                .private_prediction
                .as_ref()
                .is_some_and(|bound| bound.is_retired())
    }) {
        return Err(error(
            "Close the other windows and clients of this private practice before ending",
        ));
    }
    let state = client.state.clone();
    let bound = state
        .private_prediction
        .as_ref()
        .ok_or_else(|| error("This is not a private practice"))?;
    bound.retire(&state).await.map_err(error)?;
    // A failed durable write retains global quarantine. This client is retired
    // and can retry End; no ordinary fallback or loss of history is allowed.
    super::private_policy::end(&root).map_err(error)?;
    // Keep this retired router through its ordinary lease solely for the final
    // immutable UI action receipt and window cleanup. Its admission is closed.
    engine.ended_clients.lock().await.insert(body.client);
    drop(clients);
    engine.workspaces.lock().await.remove(&root);
    engine.locks.lock().await.remove(&root);
    Ok(Json(
        json!({"ended":true,"history_retained":true,"root_still_private":true,"context_reusable":false}),
    ))
}

async fn lease(State(engine): State<Arc<Engine>>, Path(id): Path<String>) -> StatusCode {
    if let Some(client) = engine.clients.lock().await.get_mut(&id) {
        client.seen = Instant::now();
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}
async fn detach(State(engine): State<Arc<Engine>>, Path(id): Path<String>) -> StatusCode {
    engine.clients.lock().await.remove(&id);
    engine.subscriptions.write().unwrap().remove(&id);
    StatusCode::NO_CONTENT
}

async fn dispatch(State(engine): State<Arc<Engine>>, req: Request) -> Response {
    let Some((id, path)) = req
        .uri()
        .path()
        .strip_prefix("/clients/")
        .and_then(|path| path.split_once('/'))
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if path != "api" && !path.starts_with("api/") {
        return StatusCode::NOT_FOUND.into_response();
    }
    let id = id.to_owned();
    let (router, state, startup) = {
        let mut clients = engine.clients.lock().await;
        let Some(client) = clients.get_mut(&id) else {
            return StatusCode::NOT_FOUND.into_response();
        };
        client.seen = Instant::now();
        (
            client.router.clone(),
            client.state.clone(),
            client.startup.clone(),
        )
    };
    let (mut parts, body) = req.into_parts();
    let uri = parts.uri.to_string();
    parts.uri = uri
        .strip_prefix(&format!("/clients/{id}"))
        .unwrap()
        .parse()
        .unwrap();
    let websocket = parts.headers.contains_key("upgrade");
    if parts.method == axum::http::Method::GET && parts.uri.path() == "/api/startup" {
        return match startup.result() {
            None => Json(json!({ "ready": false })).into_response(),
            Some(Ok(())) => Json(json!({ "ready": true })).into_response(),
            Some(Err(error)) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "error": error })),
            )
                .into_response(),
        };
    }
    let bootstrap = !websocket && super::startup::editor_bootstrap(parts.uri.path(), &parts.method);
    // Free the browser's HTTP connection slots while preparation is pending.
    // The marker means this request has NOT reached a handler and is safe
    // for the client to submit once initialization completes, including PUTs.
    if !bootstrap && !websocket && startup.result().is_none() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            [("x-hickory-startup", "pending")],
            Json(json!({ "error": "Preparing generated files…" })),
        )
            .into_response();
    }
    if !bootstrap && let Err(error) = startup.wait().await {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": error.to_string() })),
        )
            .into_response();
    }
    state.store.register(&state.index);
    let read = parts.method == axum::http::Method::GET || websocket;
    let request = Request::from_parts(parts, body);
    if read || bootstrap {
        router.oneshot(request).await.unwrap()
    } else {
        super::writes::during(&engine.writes, async {
            router.oneshot(request).await.unwrap()
        })
        .await
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct CommandRequest {
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub env: Vec<(String, String)>,
    pub input: Vec<u8>,
}
#[derive(Serialize, Deserialize)]
pub(super) struct CommandResult {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

async fn command(
    State(engine): State<Arc<Engine>>,
    Json(opts): Json<CommandRequest>,
) -> Api<Json<CommandResult>> {
    let attaching = engine.attaching.lock().await;
    if super::private_policy::active().map_err(error)?
        || engine
            .workspaces
            .lock()
            .await
            .values()
            .any(|state| state.private_prediction.is_some())
    {
        return Err(error(
            "Managed command execution is unavailable while a private prediction workspace is active; use sealed Verify",
        ));
    }
    if super::private_policy::overlaps(&opts.cwd.canonicalize().map_err(error)?).map_err(error)? {
        return Err(error(
            "Ordinary commands cannot execute in a retained private root; reopen its private practice",
        ));
    }
    engine
        .workers
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    drop(attaching);
    let result = async {
        let _write = engine.writes.lock().await;
        // Flush live room text before the CLI reads the working tree.
        let states: Vec<_> = engine.workspaces.lock().await.values().cloned().collect();
        for state in &states {
            for (id, _) in state.index.entries() {
                if let Some(room) = state.rooms.get(&id).await {
                    let path = state.index.absolute(&id).unwrap();
                    serve::store::write_atomic(&path, room.text().await.as_bytes())
                        .map_err(error)?;
                }
            }
        }
        let before = engine.watch.woven.lock().await.snapshot();
        let mut child = tokio::process::Command::new(opts.executable)
            .args(opts.args)
            .current_dir(opts.cwd)
            .env_clear()
            .envs(opts.env)
            .env(WORKER, "1")
            .kill_on_drop(true)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(error)?;
        let mut stdin = child.stdin.take().unwrap();
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            let _ = stdin.write_all(&opts.input).await;
        });
        let output = child.wait_with_output().await.map_err(error)?;
        let mut woven = engine.watch.woven.lock().await;
        engine.watch.refresh(woven.adopt_changed(&before));
        for state in states {
            state.store.register(&state.index);
            serve::watch::reconcile_rooms(&state, &state.index.root().to_path_buf(), &woven).await;
        }
        Ok(Json(CommandResult {
            code: output.status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }))
    }
    .await;
    engine
        .workers
        .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    result
}

async fn tool(
    State(engine): State<Arc<Engine>>,
    Json(call): Json<super::mcp::Call>,
) -> Api<Json<Value>> {
    let state = engine
        .clients
        .lock()
        .await
        .get(&call.client)
        .map(|c| c.state.clone())
        .ok_or_else(|| error("MCP client disconnected; reconnect before editing"))?;
    if let Some(bound) = &state.private_prediction {
        return Ok(Json(bound.mcp_tool(state.executor, &call.params).await));
    }
    if super::private_policy::active().map_err(error)?
        || engine
            .workspaces
            .lock()
            .await
            .values()
            .any(|state| state.private_prediction.is_some())
    {
        return Err(error(
            "MCP tools for another workspace are unavailable while the engine retains a private prediction workspace",
        ));
    }
    if call.options.root.canonicalize().map_err(error)?
        != state.index.root().canonicalize().map_err(error)?
    {
        return Err(error("MCP options do not match the attached workspace"));
    }
    super::writes::during(&engine.writes, async {
        let before = engine.watch.woven.lock().await.snapshot();
        for (id, _) in state.index.entries() {
            if let Some(room) = state.rooms.get(&id).await {
                serve::store::write_atomic(
                    &state.index.absolute(&id).unwrap(),
                    room.text().await.as_bytes(),
                )
                .map_err(error)?;
            }
        }
        let mut servers = engine.mcp.lock().await;
        if !servers.contains_key(&call.client) {
            servers.insert(
                call.client.clone(),
                crate::mcp::engine_server(call.options)
                    .await
                    .map_err(error)?,
            );
        }
        let result = servers
            .get_mut(&call.client)
            .unwrap()
            .handle("tools/call", &call.params)
            .await
            .map_err(|(_, message)| error(message))?;
        let mut woven = engine.watch.woven.lock().await;
        engine.watch.refresh(woven.adopt_changed(&before));
        serve::watch::reconcile_rooms(&state, &state.index.root().to_path_buf(), &woven).await;
        Ok(Json(result))
    })
    .await
}
