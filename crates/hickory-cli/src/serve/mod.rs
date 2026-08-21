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

pub mod agent;
pub mod api;
pub mod debug_bridge;
pub mod find;
pub mod formula;
pub mod git;
pub mod lsp_bridge;
pub mod plain_file;
pub mod refactor;
pub mod reveal;
pub mod socket;
pub mod store;
pub mod terminal;
pub mod watch;
pub mod workspace;

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{Context as _, Result};
use axum::Router;
use axum::routing::{delete, get, post, put};
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
    /// Where the desktop app's Settings page persists LLM provider keys
    /// (`<app config dir>/llm-keys.json`). `None` — the CLI and the tests —
    /// means no file: keys come from the environment alone, exactly as
    /// before this field existed.
    pub key_store_path: Option<PathBuf>,
    /// Where the desktop app's Settings page persists UI settings — today
    /// the custom window title (`<app config dir>/ui.json`, beside
    /// `llm-keys.json`). `None` — the CLI — means nothing persists: the
    /// routes still answer, with defaults.
    pub ui_settings_path: Option<PathBuf>,
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
    /// The project search engine, built on first use and reused: reopening
    /// per request would re-parse the whole embedding index every query.
    /// std Mutex, always locked inside `spawn_blocking`.
    pub search: Arc<std::sync::Mutex<Option<hick_search::SearchEngine>>>,
    /// The in-app agent's conversation trees and test seam. See
    /// [`agent::AgentHub`].
    pub agent: Arc<agent::AgentHub>,
    /// The store behind the rooms, kept here for its echo test
    /// ([`store::FileDocStore::was_own_write`]) — the in-app up-loop must
    /// not mistake the rooms' own persists for external edits.
    pub store: Arc<FileDocStore>,
    /// The LLM provider keys the Settings page manages, and where they
    /// persist. See [`KeySettings`].
    pub keys: Arc<KeySettings>,
    /// The UI settings the Settings page manages, and where they persist.
    /// See [`UiSettings`].
    pub ui: Arc<UiSettings>,
    /// Pinned refactor baselines, one per document under restructuring.
    /// Session state, deliberately in memory — see [`refactor`].
    pub refactors: Arc<Mutex<HashMap<String, refactor::RefactorBaseline>>>,
    /// Every terminal session in the window, and the attention queue across
    /// them. Sessions outlive their panes, so they belong to the session
    /// state rather than to any one client. See [`terminal`].
    pub terminals: Arc<hick_term::Terminals>,
}

/// The session's provider-key settings: the live store the agent route reads
/// (so a key saved in Settings works on the very next turn, no restart), and
/// the file it persists to (`None` = env-only, the CLI's mode).
pub struct KeySettings {
    pub store: std::sync::RwLock<hickory_agent::KeyStore>,
    pub path: Option<PathBuf>,
}

/// The session's UI settings — mirrors [`KeySettings`]: a live store the
/// routes read and write, and the file it persists to (`None` = in-memory
/// only, the CLI's mode).
pub struct UiSettings {
    pub store: std::sync::RwLock<UiStore>,
    pub path: Option<PathBuf>,
}

/// What `ui.json` holds: `{"window_title": string|null}`. The desktop shell
/// reads this file directly at launch to name the native window before any
/// page has loaded; the page reads it over `GET /api/settings/ui` for the
/// in-window `document.title`.
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct UiStore {
    /// Custom window title; `None` means the default (the folder's name).
    #[serde(default)]
    pub window_title: Option<String>,
}

impl UiStore {
    /// Read the file, tolerating its absence (a fresh install has none).
    /// A file that exists but does not parse is an error naming the file —
    /// silently discarding a setting the user made is worse than failing
    /// where the cause is visible.
    pub fn load(path: &Path) -> Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(raw) => serde_json::from_str(&raw)
                .with_context(|| format!("unreadable UI settings in {}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
        }
    }

    /// Persist, creating the parent directory on first save.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        let body = serde_json::to_string_pretty(self).context("encoding UI settings")?;
        store::write_atomic(path, body.as_bytes())
            .with_context(|| format!("writing {}", path.display()))
    }
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
        .route(
            "/projects/{id}/docs",
            get(api::project_docs).post(api::create_doc),
        )
        .route("/docs/{id}", get(api::get_doc).put(api::put_doc))
        .route("/docs/{id}/render", get(api::render_doc))
        .route("/docs/{id}/outputs", get(api::list_outputs))
        .route("/docs/{id}/outputs/file", get(api::get_output_file))
        .route("/docs/{id}/outputs/edit", post(api::edit_outputs))
        .route("/docs/{id}/refactor/begin", post(refactor::begin))
        .route("/docs/{id}/refactor/status", get(refactor::status))
        .route("/docs/{id}/refactor/end", post(refactor::end))
        .route("/adopt", post(refactor::adopt))
        .route("/docs/{id}/run", post(api::run_doc))
        .route("/docs/{id}/check", post(api::check_doc))
        .route("/docs/{id}/agent", post(agent::start_turn))
        .route("/docs/{id}/agent/turns", get(agent::list_turns))
        .route("/runs/{id}", get(api::get_run))
        .route(
            "/settings/keys",
            get(api::get_settings_keys).put(api::put_settings_keys),
        )
        .route(
            "/settings/ui",
            get(api::get_settings_ui).put(api::put_settings_ui),
        )
        .route("/files", get(api::files))
        .route("/reveal", post(reveal::reveal))
        .route("/open-external", post(reveal::open_external))
        .route("/file", get(plain_file::get_file).put(plain_file::put_file))
        .route("/scratchpad", post(plain_file::post_scratchpad))
        .route("/search", get(api::search))
        // What this project calls things — a different question from the
        // language server's, and shown beside it rather than instead of it.
        .route("/complete", get(api::complete))
        // Exhaustive, not ranked — see serve/find.rs for why that distinction
        // is the whole reason this is not `/search`.
        .route("/find", get(find::find))
        .route("/find/replace", post(find::replace))
        .route("/structure", get(api::structure))
        // Who last touched each line. Off by default in the editor, so this
        // is only ever asked for by someone who turned the column on.
        .route("/blame", get(api::blame))
        // History, read-only. Changing a repository is a thing people rightly
        // do deliberately, where the exact command is visible — and there is
        // a terminal on every row of the tree. See serve/git.rs.
        .route("/git/log", get(git::log))
        .route("/git/status", get(git::status))
        // Formulas: the host has already resolved references and worked out
        // the order by the time a backend sees anything. See serve/formula.rs.
        .route("/formula/evaluate", post(formula::evaluate))
        .route("/formula/languages", get(formula::languages))
        .route("/executor", get(api::executor))
        .route("/health", get(api::health))
        .route(
            "/workspace/ui",
            get(workspace::get_ui).put(workspace::put_ui),
        )
        .route(
            "/workspace/drafts",
            get(workspace::list_drafts)
                .put(workspace::put_draft)
                .delete(workspace::discard_draft),
        )
        .route("/terminals", get(terminal::list).post(terminal::open))
        .route("/terminals/turbo", put(terminal::set_turbo))
        .route("/terminals/ws", get(terminal::ws_handler))
        .route("/terminals/{id}", delete(terminal::close))
        .route("/terminals/{id}/input", post(terminal::input))
        .route("/terminals/{id}/resize", post(terminal::resize))
        .route("/terminals/{id}/interrupt", post(terminal::interrupt))
        .route("/terminals/{id}/answer", post(terminal::answer))
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
    // An empty folder is a valid session, not an error: it is the app's
    // first-run state (the default workspace starts with no documents, and
    // the UI lands on a fresh untitled one). A FILE target that does not
    // exist is still refused below by DocIndex::scan.
    let index = Arc::new(DocIndex::scan(&root)?);

    // Provider keys load before the first request so a bad file fails here,
    // with the file named, rather than as a mystery 500 on the first agent
    // turn. No path (the CLI) means an empty store: every store-aware code
    // path then degrades to exactly the env-only behavior.
    let key_store = match &opts.key_store_path {
        Some(path) => hickory_agent::KeyStore::load(path)?,
        None => hickory_agent::KeyStore::default(),
    };

    // Same story for the UI settings: a bad file fails here, named, rather
    // than as a mystery on the first Settings visit.
    let ui_store = match &opts.ui_settings_path {
        Some(path) => UiStore::load(path)?,
        None => UiStore::default(),
    };

    // A bad HICKORY_TERM_SCROLLBACK fails here, naming the variable, rather
    // than when a session happens to overflow its buffer an hour later.
    let term_config = hick_term::TermConfig::from_env()?;

    let store = FileDocStore::new(index.clone());
    let state = LocalState {
        store: store.clone(),
        index: index.clone(),
        rooms: Arc::new(RoomRegistry::new(store)),
        runs: Arc::new(Mutex::new(HashMap::new())),
        params: Arc::new(opts.params),
        executor: opts.executor,
        search: Arc::new(std::sync::Mutex::new(None)),
        agent: Arc::new(agent::AgentHub::default()),
        keys: Arc::new(KeySettings {
            store: std::sync::RwLock::new(key_store),
            path: opts.key_store_path,
        }),
        ui: Arc::new(UiSettings {
            store: std::sync::RwLock::new(ui_store),
            path: opts.ui_settings_path,
        }),
        refactors: Arc::new(Mutex::new(HashMap::new())),
        terminals: Arc::new(hick_term::Terminals::new(term_config)),
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

pub(crate) fn now_rfc3339() -> String {
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
pub(crate) fn rand_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    now.rotate_left(17) ^ n
}
