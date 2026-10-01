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
    locks: Mutex<Vec<crate::up::DirectoryLock>>,
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
        locks: Mutex::new(Vec::new()),
    });
    let router = Router::new()
        .route(
            "/health",
            get(|| async { Json(json!({"protocol":PROTOCOL})) }),
        )
        .route("/attach", post(attach))
        .route("/command", post(command))
        .route("/tool", post(tool))
        .layer(axum::extract::DefaultBodyLimit::max(64 * 1024 * 1024))
        .route("/clients/{id}/lease", post(lease).delete(detach))
        .route("/clients/{id}/api", axum::routing::any(dispatch))
        .route("/clients/{id}/api/{*path}", axum::routing::any(dispatch))
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

async fn attach(
    State(engine): State<Arc<Engine>>,
    Json(mut opts): Json<Attach>,
) -> Api<Json<Attached>> {
    let _attach = engine.attaching.lock().await;
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
    let mut workspaces = engine.workspaces.lock().await;
    let mut state = if let Some(state) = workspaces.get(&root) {
        if state.executor != opts.executor || *state.params != opts.params {
            return Err(error(
                "This workspace is using different execution settings. Use the same executor and parameters as its existing session.",
            ));
        }
        state.clone()
    } else {
        engine
            .locks
            .lock()
            .await
            .push(crate::up::DirectoryLock::acquire(&root).map_err(error)?);
        let mut state = serve::prepare(ServeOptions {
            target: root.clone(),
            port: 0,
            params: opts.params.clone(),
            executor: opts.executor,
            key_store_path: opts.key_store_path.clone(),
            ui_settings_path: opts.ui_settings_path.clone(),
        })
        .await
        .map_err(error)?
        .state;
        state.index = Arc::new(DocIndex::shared(&root).map_err(error)?);
        state.store = engine.store.clone();
        state.store.register(&state.index);
        state.rooms = engine.rooms.clone();
        state.writes = engine.writes.clone();
        state.engine_woven = Some(engine.watch.woven.clone());
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
    engine
        .subscriptions
        .write()
        .unwrap()
        .insert(opts.client.clone(), opts.clone());
    engine
        .watch
        .register(state.clone(), opts.clone())
        .await
        .map_err(error)?;
    let router = serve::router(state.clone());
    engine.clients.lock().await.insert(
        opts.client,
        Client {
            root,
            router,
            state,
            slot: slot.clone(),
            seen: Instant::now(),
        },
    );
    Ok(Json(Attached { slot, log_offset }))
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
    let id = req
        .uri()
        .path()
        .split('/')
        .nth(2)
        .unwrap_or_default()
        .to_string();
    let (router, state) = {
        let mut clients = engine.clients.lock().await;
        let Some(client) = clients.get_mut(&id) else {
            return StatusCode::NOT_FOUND.into_response();
        };
        client.seen = Instant::now();
        (client.router.clone(), client.state.clone())
    };
    let (mut parts, body) = req.into_parts();
    let uri = parts.uri.to_string();
    parts.uri = uri
        .strip_prefix(&format!("/clients/{id}"))
        .unwrap()
        .parse()
        .unwrap();
    let websocket = parts.headers.contains_key("upgrade");
    state.store.register(&state.index);
    let read = parts.method == axum::http::Method::GET || websocket;
    let request = Request::from_parts(parts, body);
    if read {
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
    engine
        .workers
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
