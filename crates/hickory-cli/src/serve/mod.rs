//! The local document server — the engine behind the desktop app's window.
//!
//! One process on the user's own machine answers the subset of the API a
//! document view needs (from files, not a database) and runs the rooms
//! `hickory-collab` defines. What that buys:
//!
//! - the lineage ribbons for a `.hick` file on your own disk, which nothing
//!   else can show you;
//! - an editor whose durable state is your working tree, so your other editor
//!   and `git diff` see every keystroke;
//! - execution on your hardware, under your executor.
//!
//! ## This is a library, not a command
//!
//! There is no `hick serve`. The desktop app links this module and runs it
//! in-process on loopback, loading the React client Tauri bundles with it;
//! `hick up` is headless and never starts it. That is why nothing here serves
//! static files, and why there is no `--web-dist`.
//!
//! ## Why there is no authorization
//!
//! This server binds `127.0.0.1` and answers exactly one person: the one whose
//! machine it is. There is no second principal to distinguish from the first,
//! so there is no token, no scope, and no capability link. A check that can
//! only ever pass is worse than no check, because it reads like protection.
//!
//! Sharing is not a feature this product has — not disabled, not deferred.
//! See `docs/specs/freeform/local-only.md`.

pub mod api;
pub mod debug_bridge;
pub mod lsp_bridge;
pub mod socket;
pub mod store;

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{Context as _, Result};
use axum::Router;
use axum::routing::{get, post};
use hickory_collab::RoomRegistry;
use serde_json::{Value, json};

use crate::{ExecutorChoice, RunMode};
use api::{ApiError, ApiResult};
use store::{DocIndex, FileDocStore};

/// How a local session was asked to run.
///
/// There is no `lan`, `scope`, `public`, or `web_dist` field, and there will
/// not be: this server binds loopback, answers one user — the one whose
/// machine it is — and has no UI of its own. The desktop app bundles the
/// client; the CLI does not serve HTML. See
/// `docs/specs/freeform/local-only.md`.
pub struct ServeOptions {
    /// The document, or the directory of documents, to serve.
    pub target: PathBuf,
    /// Port to listen on. 0 lets the kernel choose.
    pub port: u16,
    /// Parameter overrides passed to every weave and run.
    pub params: Vec<(String, String)>,
    /// Executor for runs (`HICKORY_EXECUTOR`).
    pub executor: ExecutorChoice,
}

/// One recorded run, in the shape `GET /api/runs/:id` returns.
#[derive(Clone)]
pub struct RunRecord {
    pub status: String,
    pub started_at: String,
    pub blocks: Value,
}

/// Everything a request handler needs.
#[derive(Clone)]
pub struct LocalState {
    pub index: Arc<DocIndex>,
    pub rooms: Arc<RoomRegistry>,
    pub runs: Arc<Mutex<HashMap<String, RunRecord>>>,
    pub params: Arc<Vec<(String, String)>>,
    pub executor: ExecutorChoice,
}

impl LocalState {
    pub fn executor_kind(&self) -> &'static str {
        self.executor.as_str()
    }

    pub fn read_source(&self, id: &str) -> ApiResult<String> {
        let path = self
            .index
            .absolute(id)
            .ok_or_else(|| ApiError::not_found(format!("no document {id} in this session")))?;
        std::fs::read_to_string(&path)
            .map_err(|e| ApiError::internal(format!("reading {}: {e}", path.display())))
    }

    pub fn write_source(&self, id: &str, source: &str) -> ApiResult<()> {
        let path = self
            .index
            .absolute(id)
            .ok_or_else(|| ApiError::not_found(format!("no document {id} in this session")))?;
        store::write_atomic(&path, source.as_bytes())
            .map_err(|e| ApiError::internal(format!("writing {}: {e:#}", path.display())))
    }

    /// Resolve a document path as provenance names it.
    ///
    /// Provenance carries the path the weave used, which for the primary
    /// document is what the pipeline was handed. Both the absolute form and
    /// the root-relative form are accepted, and anything that escapes the
    /// served root is refused — a guest must not be able to steer an edit into
    /// the host's home directory.
    fn resolve_doc_path(&self, doc_path: &str) -> ApiResult<PathBuf> {
        let candidate = Path::new(doc_path);
        let joined = if candidate.is_absolute() {
            candidate.to_path_buf()
        } else {
            self.index.root().join(candidate)
        };
        let root = self
            .index
            .root()
            .canonicalize()
            .unwrap_or_else(|_| self.index.root().to_path_buf());
        let resolved = joined.canonicalize().unwrap_or(joined);
        if !resolved.starts_with(&root) {
            return Err(ApiError::forbidden(format!(
                "{doc_path} is outside the directory being served"
            )));
        }
        Ok(resolved)
    }

    pub fn read_source_by_doc_path(&self, doc_path: &str) -> ApiResult<String> {
        let path = self.resolve_doc_path(doc_path)?;
        std::fs::read_to_string(&path)
            .map_err(|e| ApiError::internal(format!("reading {}: {e}", path.display())))
    }

    pub fn write_source_by_doc_path(&self, doc_path: &str, source: &str) -> ApiResult<()> {
        let path = self.resolve_doc_path(doc_path)?;
        store::write_atomic(&path, source.as_bytes())
            .map_err(|e| ApiError::internal(format!("writing {}: {e:#}", path.display())))
    }

    /// The room id for a document named the way provenance names it.
    pub fn id_for_doc_path(&self, doc_path: &str) -> String {
        let rel = self
            .resolve_doc_path(doc_path)
            .ok()
            .and_then(|p| {
                p.strip_prefix(self.index.root().canonicalize().ok()?)
                    .ok()
                    .map(|r| r.to_string_lossy().replace('\\', "/"))
            })
            .unwrap_or_else(|| doc_path.to_string());
        self.index.id_for_path(&rel)
    }

    /// Weave (never execute) a document, from the text the room is serving if
    /// one is live, else from disk.
    ///
    /// Reading through the room matters: a collaborator's unsaved keystrokes
    /// are in the CRDT for up to the persist debounce, and rendering the file
    /// instead would show everyone a document that is a moment out of date.
    pub async fn weave(&self, id: &str) -> ApiResult<crate::DocRun> {
        let path = self
            .index
            .absolute(id)
            .ok_or_else(|| ApiError::not_found(format!("no document {id} in this session")))?;

        // Flush the live text to disk first: the pipeline reads files (and
        // upstream documents) from the filesystem, so a weave of in-memory
        // text alone would silently ignore the room's newest edits.
        if let Some(room) = self.rooms.get(id).await {
            let live = room.text().await;
            if live != std::fs::read_to_string(&path).unwrap_or_default() {
                store::write_atomic(&path, live.as_bytes())
                    .map_err(|e| ApiError::internal(format!("{e:#}")))?;
            }
        }

        crate::run_doc(&path, &self.params, RunMode::Weave, ExecutorChoice::Local)
            .await
            .map_err(|e| ApiError::unprocessable(format!("{e:#}")))
    }

    /// Execute a document, publishing transcript events and the final status
    /// on the run channel. Returns the run id immediately.
    pub async fn start_run(&self, id: &str, check_only: bool) -> ApiResult<String> {
        let path = self
            .index
            .absolute(id)
            .ok_or_else(|| ApiError::not_found(format!("no document {id} in this session")))?;
        let run_id = format!("{:016x}", rand_id());
        let started_at = now_rfc3339();

        self.runs.lock().unwrap().insert(
            run_id.clone(),
            RunRecord {
                status: "running".into(),
                started_at: started_at.clone(),
                blocks: json!([]),
            },
        );

        let state = self.clone();
        let doc_key = id.to_string();
        let run_key = run_id.clone();
        tokio::spawn(async move {
            let mode = if check_only {
                RunMode::Verify
            } else {
                RunMode::Execute
            };
            let outcome = crate::run_doc(&path, &state.params, mode, state.executor).await;

            let (status, blocks) = match outcome {
                Ok(run) => {
                    // Replay each cell's transcript on the run channel, the
                    // same shape the hosted server publishes.
                    for (exec_id, entries) in &run.result.transcripts {
                        for event in entries.iter().flat_map(|entry| &entry.events) {
                            state
                                .rooms
                                .publish_run_event(
                                    &doc_key,
                                    &json!({
                                        "run_id": run_key,
                                        "exec_id": exec_id,
                                        "event": event,
                                    }),
                                )
                                .await;
                        }
                    }
                    if !check_only && let Err(e) = crate::write_outputs(&run, None) {
                        log::error!("writing outputs failed: {e:#}");
                    }
                    let failed = run.result.expectations.iter().any(|o| !o.passed);
                    let blocks = crate::block_model_json(&run)
                        .map(|v| v.get("blocks").cloned().unwrap_or(json!([])))
                        .unwrap_or(json!([]));
                    (if failed { "failed" } else { "ok" }, blocks)
                }
                Err(e) => {
                    log::warn!("run {run_key} failed: {e:#}");
                    ("failed", json!([]))
                }
            };

            if let Some(record) = state.runs.lock().unwrap().get_mut(&run_key) {
                record.status = status.to_string();
                record.blocks = blocks;
            }
            state
                .rooms
                .publish_run_event(&doc_key, &json!({ "run_id": run_key, "status": status }))
                .await;
        });

        Ok(run_id)
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// The routes a document view needs, and nothing else.
///
/// No authorization layer: this server binds loopback and answers the person
/// whose machine it is. There is no second principal to distinguish from the
/// first, so there is no token to check — and a check that can only ever pass
/// is worse than none, because it reads like protection.
///
/// No `/me`, `/billing/plans`, or `/analytics/capture` either. Those existed
/// because the React client was shared with a hosted product and asked for
/// them on load. It is not shared any more.
fn router(state: LocalState) -> Router {
    let api = Router::new()
        .route("/projects", get(api::projects))
        .route("/projects/{id}/docs", get(api::project_docs))
        .route("/docs/{id}", get(api::get_doc).put(api::put_doc))
        .route("/docs/{id}/render", get(api::render_doc))
        .route("/docs/{id}/outputs", get(api::list_outputs))
        .route("/docs/{id}/outputs/file", get(api::get_output_file))
        .route("/docs/{id}/outputs/edit", post(api::edit_outputs))
        .route("/docs/{id}/run", post(api::run_doc))
        .route("/docs/{id}/check", post(api::check_doc))
        .route("/docs/{id}/agent", post(api::agent_unavailable))
        .route("/docs/{id}/agent/turns", get(api::agent_turns))
        .route("/runs/{id}", get(api::get_run))
        .route("/structure", get(api::structure))
        .route("/executor", get(api::executor))
        .route("/health", get(api::health))
        .route("/ws", get(socket::ws_handler));

    Router::new().nest("/api", api).with_state(state)
}

/// A prepared session: the router, and the state a caller that wants its own
/// listener needs to talk to it.
pub struct Prepared {
    pub router: Router,
    pub state: LocalState,
}

/// Build a session without binding a socket.
///
/// Split out from [`serve`] so the collaboration guarantees can be driven by a
/// test over a real socket on an ephemeral port, and so the desktop app can
/// mount this router on a listener it owns.
pub async fn prepare(opts: ServeOptions) -> Result<Prepared> {
    let target = opts
        .target
        .canonicalize()
        .with_context(|| format!("no such path: {}", opts.target.display()))?;
    let root = if target.is_dir() {
        target.clone()
    } else {
        target
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    };

    // Scan the target to decide whether there is anything to serve, then index
    // the directory that contains it: rooms and path resolution work from the
    // root, so a single-file session can still follow `hick:upstream` beside
    // it.
    let found = DocIndex::scan(&target)
        .map(|index| index.entries().len())
        .unwrap_or(0);
    if found == 0 {
        anyhow::bail!(
            "no .hick documents to open under {}\n\
             Point this at a document, or at a directory containing one.",
            target.display()
        );
    }
    let index = Arc::new(DocIndex::scan(&root)?);

    let store = FileDocStore::new(index.clone());
    let state = LocalState {
        index: index.clone(),
        rooms: Arc::new(RoomRegistry::new(store)),
        runs: Arc::new(Mutex::new(HashMap::new())),
        params: Arc::new(opts.params),
        executor: opts.executor,
    };

    Ok(Prepared {
        router: router(state.clone()),
        state,
    })
}

/// Start the session on loopback and serve until interrupted.
///
/// Loopback is not a default that a flag can change. A process that starts
/// listening on `0.0.0.0` has made a decision about who may reach the user's
/// files, and this product does not make that decision at all — see
/// `docs/specs/freeform/local-only.md`.
pub async fn serve(opts: ServeOptions) -> Result<()> {
    let port = opts.port;
    let prepared = prepare(opts).await?;

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("binding {addr}"))?;
    let bound = listener.local_addr()?;
    log::info!("local session on http://{bound}");

    axum::serve(
        listener,
        prepared
            .router
            .into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Small helpers
// ---------------------------------------------------------------------------

fn now_rfc3339() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let rest = secs % 86_400;
    let (y, m, d) = api::civil_from_days_public(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        (rest % 3600) / 60,
        rest % 60
    )
}

/// A run id. Uniqueness within one process is all this needs.
fn rand_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    now.rotate_left(17) ^ n
}
