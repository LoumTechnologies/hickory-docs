//! `hickory serve` — the same collaborative editor, hosted by whoever is
//! working on the document.
//!
//! One process on a contributor's machine serves the React client, answers the
//! subset of the API a document view needs (from files, not a database), and
//! runs the rooms `hickory-collab` defines. What that buys, in order of how
//! much it matters:
//!
//! - the lineage ribbons for a `.hick` file on your own disk, which nothing
//!   else can show you;
//! - collaborative editing whose durable state is your working tree, so your
//!   editor and `git diff` see every keystroke your collaborator makes;
//! - execution on your hardware, under your executor, subject to your policy.
//!
//! It is the same client and the same protocol as hickorydocs.com, because the
//! whole design of `local-collaboration.md` is that the server *role* moves,
//! not the product.
//!
//! ## The one rule worth reading before changing this module
//!
//! A capability link is a credential that anyone it is forwarded to can use.
//! Scopes are therefore enforced in one place ([`LocalState::require_scope_*`])
//! and `run` is never granted implicitly. A shared session that can execute
//! refuses to start on the unsandboxed local executor — see [`ShareGuard`].

pub mod api;
pub mod relay;
pub mod share;
pub mod socket;
pub mod store;
pub mod tunnel;

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
use share::{Caller, Scope, ShareGuard};
use store::{DocIndex, FileDocStore};

/// How a session was asked to run.
pub struct ServeOptions {
    /// The document, or the directory of documents, to serve.
    pub target: PathBuf,
    /// Port to listen on. 0 lets the kernel choose.
    pub port: u16,
    /// Bind to every interface (LAN sharing) rather than loopback only.
    ///
    /// Off by default: a process that starts listening on 0.0.0.0 because
    /// someone typed a two-word command has made a decision the person did
    /// not.
    pub lan: bool,
    /// What a holder of the share link may do.
    pub scope: Scope,
    /// Where the built web client lives.
    pub web_dist: Option<PathBuf>,
    /// Parameter overrides passed to every weave and run.
    pub params: Vec<(String, String)>,
    /// Executor for runs (`HICKORY_EXECUTOR`).
    pub executor: ExecutorChoice,
    /// Ask for an address that reaches beyond this network (`--public`).
    pub public: bool,
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
    /// The host's own credential, printed on the console. Always full
    /// capability: this is the person whose machine it is.
    pub host_token: Arc<String>,
    /// The credential in the share link, carrying `guest_scope`. Minted even
    /// for an unshared session so nothing has to branch on its absence.
    pub guest_token: Arc<String>,
    pub guest_scope: Scope,
    pub params: Arc<Vec<(String, String)>>,
    pub executor: ExecutorChoice,
}

impl LocalState {
    /// Resolve a presented token into what its holder may do.
    ///
    /// Unknown tokens resolve to `None` — the caller answers 403 without
    /// saying which token was wrong, because "close" is information.
    pub fn caller_for(&self, token: &str) -> Option<Caller> {
        if token == *self.host_token {
            return Some(Caller::host());
        }
        if token == *self.guest_token {
            return Some(Caller::guest(self.guest_scope));
        }
        None
    }

    /// What `GET /api/me` calls this session.
    pub fn identity(&self) -> String {
        format!("{}@local", whoami())
    }

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

fn router(state: LocalState, web_dist: Option<PathBuf>) -> Router {
    let api = Router::new()
        .route("/me", get(api::me))
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
        .route("/billing/plans", get(api::plans))
        .route("/analytics/capture", post(api::capture))
        .route("/executor", get(api::executor))
        .route("/health", get(api::health))
        .route("/ws", get(socket::ws_handler));

    // Every /api route requires a valid session token and carries the
    // resolved capability into the handler. Static assets do not: the browser
    // fetches the bundle before it has a token, and the bundle is the same
    // public artifact hickorydocs.com serves.
    let api = api.layer(axum::middleware::from_fn_with_state(
        state.clone(),
        share::authorize,
    ));
    let mut router = Router::new().nest("/api", api);
    if let Some(dist) = web_dist {
        router = router.fallback(share::web_fallback(dist));
    }
    router.with_state(state)
}

/// A prepared session: the router, and the state a caller that wants its own
/// listener needs to talk to it.
pub struct Prepared {
    pub router: Router,
    pub state: LocalState,
    pub guard: ShareGuard,
    pub relay: relay::Relay,
}

/// Build a session without binding a socket.
///
/// Split out from [`serve`] so the collaboration guarantees can be driven by a
/// test over a real socket on an ephemeral port, rather than asserted about
/// code that only ever runs inside `main`.
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
    // `expand_docs` reports an empty directory in the vocabulary of `hickory
    // run`; a session that failed to start needs to hear about `hickory serve`.
    let found = DocIndex::scan(&target)
        .map(|index| index.entries().len())
        .unwrap_or(0);
    if found == 0 {
        anyhow::bail!(
            "no .hick documents to serve under {}\n\
             Point `hickory serve` at a document, or at a directory containing one.",
            target.display()
        );
    }
    let index = Arc::new(DocIndex::scan(&root)?);

    // Refuse a shared, runnable session on an executor that would run a
    // guest's code as the host.
    let guard = ShareGuard::evaluate(opts.lan, opts.scope, opts.executor);
    guard.enforce()?;

    // Resolve reachability before binding: a session that advertises a public
    // link it cannot actually provide is worse than one that refused to start.
    let relay = relay::discover(&relay::RelayEnv {
        requested: opts.public,
        shared: opts.lan,
        explicit_url: std::env::var("HICKORY_PUBLIC_URL").ok(),
        pz_tunnel: std::env::var("PZ_TUNNEL").ok(),
        relay_base: std::env::var("HICKORY_RELAY_URL")
            .unwrap_or_else(|_| relay::DEFAULT_RELAY_URL.to_string()),
        hickory_token: crate::login::load().map(|c| c.access_token),
        resolve_pz: relay::resolve_portzero,
    })?;

    let store = FileDocStore::new(index.clone());
    let state = LocalState {
        index: index.clone(),
        rooms: Arc::new(RoomRegistry::new(store)),
        runs: Arc::new(Mutex::new(HashMap::new())),
        host_token: Arc::new(share::mint_token()),
        guest_token: Arc::new(share::mint_token()),
        guest_scope: opts.scope,
        params: Arc::new(opts.params),
        executor: opts.executor,
    };

    let web_dist = match opts.web_dist.or_else(share::discover_web_dist) {
        Some(dir) => Some(dir),
        None => {
            log::warn!(
                "no built web client found; serving the API only. Pass --web-dist <dir> (or set HICKORY_WEB_DIST) to open the editor."
            );
            None
        }
    };

    Ok(Prepared {
        router: router(state.clone(), web_dist),
        state,
        guard,
        relay,
    })
}

/// Start the session and serve until interrupted.
pub async fn serve(opts: ServeOptions) -> Result<()> {
    let lan = opts.lan;
    let port = opts.port;
    let prepared = prepare(opts).await?;

    let addr = SocketAddr::new(
        if lan {
            IpAddr::V4(Ipv4Addr::UNSPECIFIED)
        } else {
            IpAddr::V4(Ipv4Addr::LOCALHOST)
        },
        port,
    );
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("binding {addr}"))?;
    let bound = listener.local_addr()?;

    // Our relay needs the router and the listener to exist before it can hand
    // out an address, so it is the one provider resolved here rather than in
    // `prepare`.
    let mut public_url = prepared.relay.base_url().map(str::to_string);
    if let relay::Relay::Hickory { base, token } = &prepared.relay {
        // Guest WebSockets arriving through the tunnel are bridged back into
        // this process's own listener, so the local handler — capability check
        // included — is the only door there is.
        let handle = tunnel::open(
            base,
            token,
            None,
            prepared.router.clone(),
            &format!("ws://127.0.0.1:{}", bound.port()),
        )
        .await
        .context("opening a tunnel on the relay")?;
        log::info!(
            "tunnel open as {} for {} (up to {}h)",
            handle.slug,
            handle.account,
            handle.expires_in_secs / 3600
        );
        public_url = Some(handle.url);
    }

    let index = prepared.state.index.clone();
    share::print_banner(
        &prepared.state,
        &index,
        bound,
        lan,
        &prepared.guard,
        &prepared.relay,
        public_url.as_deref(),
    );

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

fn whoami() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "you".to_string())
}

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
