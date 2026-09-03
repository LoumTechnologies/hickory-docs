//! The local engine, hosted inside the desktop app.
//!
//! The app is a window around the same server `hick up` is built from. This
//! module starts it on an ephemeral loopback port, layers the built UI on top
//! of its API routes, and hands back the address for the window to load.
//!
//! ## Why the window loads a URL rather than bundled assets
//!
//! Tauri's default is to serve the frontend from the bundle under a `tauri://`
//! origin. That would put the UI on one origin and the API on another, which
//! costs a CORS policy, a way to tell the frontend which port the server
//! landed on, and a WebSocket URL that can no longer be derived from
//! `location.host`. Serving both from one origin costs none of those: relative
//! `fetch` and `location.host` work exactly as they do in a browser, and the
//! frontend needs no knowledge that it is inside an app at all.
//!
//! The static-file serving lives *here*, not in `hickory-cli`. The CLI has no
//! UI and never serves HTML; the desktop crate composes assets onto the router
//! the CLI hands it. That is why `serve::prepare` returns a `Router`.

use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use axum::body::Body;
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{OpenWhere, ServeOptions, Shell, prepare};
use hickory_cli::up::DirectoryLock;
use tauri::Manager as _;
use tauri_plugin_dialog::DialogExt as _;

/// The built UI (`apps/web/dist`), compiled into this binary.
///
/// Embedded rather than shipped beside the executable: a downloaded app has no
/// checkout to find `dist/` in, and a loose directory next to the binary is a
/// thing a user can separate from it by moving one and not the other.
#[derive(rust_embed::Embed)]
#[folder = "../../web/dist/"]
struct Ui;

/// A running local session: where to point the window, and the lock that keeps
/// anything else from writing the same files while it runs.
pub struct Session {
    /// The engine's own address — what `/api` is on, and what a dev proxy is
    /// pointed at.
    pub url: String,
    /// What the window loads. The same as `url` in anything anybody
    /// downloads; Vite's address under `just dev`, so a frontend change is a
    /// hot reload rather than a rebuild. See dev.rs.
    pub ui_url: String,
    /// Held for the process's lifetime. Dropping it releases the directory.
    _lock: DirectoryLock,
    /// The in-app up-loop (`local-only.md`: "the desktop app runs the
    /// up-loop and the rooms in one process"). Dropping it asks the loop to
    /// stop and clear its read-only marks; if the process dies before that
    /// lands, `write_outputs` clears marks defensively on the next run.
    _watch: hickory_cli::serve::watch::WatchGuard,
}

/// A folder named explicitly by whoever started the app.
///
/// `HICKORY_PROJECT_DIR` first, then the first command-line argument. Both are
/// deliberate acts, so either one skips the picker entirely — that is what
/// makes `hickory-desktop ~/notes` and `cargo tauri dev` work without a modal
/// in the way.
///
/// **The working directory is not consulted.** An app launched from a dock,
/// Finder, or a Start menu inherits `/`, and opening `/` because nobody said
/// otherwise is never what anyone meant. When nothing is named, the app asks.
pub fn named_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("HICKORY_PROJECT_DIR")
        && !dir.trim().is_empty()
    {
        return Some(PathBuf::from(dir));
    }
    std::env::args().nth(1).map(PathBuf::from)
}

/// Where the last opened folder is remembered.
fn recent_file(config_dir: &Path) -> PathBuf {
    config_dir.join("last-folder.txt")
}

/// Where the UI settings (`{"window_title": …}`) persist — the file behind
/// `GET/PUT /api/settings/ui`, and what the launch sequence reads for the
/// native window title.
pub fn ui_settings_file(config_dir: &Path) -> PathBuf {
    config_dir.join("ui.json")
}

/// The folder this app last opened successfully, if it still exists.
///
/// Checked for existence rather than trusted: a remembered path routinely
/// points at a folder that has since been moved, renamed, or was on a drive
/// that is not mounted. Silently reopening the picker is better than an error
/// about a folder the user has forgotten they ever opened.
pub fn last_opened(config_dir: &Path) -> Option<PathBuf> {
    let raw = std::fs::read_to_string(recent_file(config_dir)).ok()?;
    let path = PathBuf::from(raw.trim());
    (path.is_dir()).then_some(path)
}

/// Remember a folder as the one to reopen next launch.
///
/// Never fails the launch: not being able to write this costs the user one
/// trip through the picker next time, which is not worth refusing to start
/// over.
pub fn remember(config_dir: &Path, dir: &Path) {
    if std::fs::create_dir_all(config_dir).is_err() {
        return;
    }
    let _ = std::fs::write(recent_file(config_dir), dir.to_string_lossy().as_bytes());
}

/// The shell's own powers, handed to the engine.
///
/// Both halves are process-level acts, which is the whole reason they cannot
/// live in the server:
///
/// * **A new window is a new process.** The directory lock and the file
///   watcher are per-process, so two folders open at once means two copies of
///   the app — exactly what `hick open` already launches, through the same
///   finder, so there is one answer to "where is the app" rather than two.
/// * **This window is a restart.** Same reason, from the other side: this
///   process holds *this* folder, and it cannot hold another. So the new
///   folder is remembered and the process restarts into it — which is what
///   `switch_to` has always done for File → Open Folder.
///
/// The restart is deferred by a moment so the HTTP response that asked for it
/// gets out first. Exiting mid-response would leave the page reporting a
/// dropped connection as if something had gone wrong, when what happened is
/// precisely what was asked for.
pub fn shell_hooks(handle: &tauri::AppHandle, config_dir: Option<&Path>) -> Shell {
    let handle = handle.clone();
    let config_dir = config_dir.map(Path::to_path_buf);
    let picker = handle.clone();
    Shell {
        // The same picker File → Open Folder uses, reached from the page
        // rather than from the menu bar. `blocking_pick_folder` blocks the
        // thread it is called on, and the route calls this on a blocking one:
        // the menu handler spawns a worker for exactly this reason, since a
        // native modal on the main thread deadlocks the app.
        pick_folder: std::sync::Arc::new(move |start: &Path| {
            let mut builder = picker
                .dialog()
                .file()
                .set_title("Choose where the project goes")
                .set_directory(start);
            // Parented to the window, unlike the menu's pickers: this one is
            // opened from a button inside the page, so a chooser that came up
            // behind the window would read as the button having done nothing.
            if let Some(window) = picker.get_webview_window("main") {
                builder = builder.set_parent(&window);
            }
            Ok(builder
                .blocking_pick_folder()
                .and_then(|p| p.into_path().ok()))
        }),
        open_folder: std::sync::Arc::new(move |folder: &Path, where_: OpenWhere| match where_ {
            OpenWhere::None => Ok(()),
            OpenWhere::NewWindow => {
                let app = hickory_cli::open_app::find(
                    |name| std::env::var(name).ok(),
                    |path| path.exists(),
                )
                .context("could not find this app's own executable to start a second copy of it")?;
                hickory_cli::open_app::open(&app, folder)
            }
            OpenWhere::ThisWindow => {
                if let Some(dir) = &config_dir {
                    remember(dir, folder);
                }
                let handle = handle.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(250));
                    handle.restart();
                });
                Ok(())
            }
        }),
    }
}

/// Start the engine on loopback and return the address to load.
///
/// Binds port 0: the app has no reason to want a particular port, and asking
/// for one means failing to start because something unrelated already holds
/// it.
///
/// `config_dir` is where the app keeps its own state; when present, the LLM
/// provider keys the Settings page manages persist there as
/// `llm-keys.json`. A desktop app is launched from a dock, where no shell
/// profile exports anything, so its keys cannot live in environment
/// variables the way the CLI's do. `None` (no config dir on this platform)
/// degrades to env-only.
pub async fn start(
    target: &Path,
    config_dir: Option<&Path>,
    shell: Option<Shell>,
) -> Result<Session> {
    // Development may name the port, so that whatever is proxying to the
    // engine can be pointed at it before the engine exists. Unset — which is
    // every downloaded copy — leaves the ephemeral port below. See dev.rs.
    let dev = crate::dev::from_env()?;

    // The lock protects a WORKING DIRECTORY: two processes weaving the same
    // folder would each read the other's writes as the user's edits. A single
    // document's working directory is the folder it sits in — locking the
    // file itself would try to create `document.hick/.hick-cache`, which is
    // not a thing.
    let lock_root = if target.is_file() {
        target.parent().unwrap_or_else(|| Path::new("."))
    } else {
        target
    };
    let lock = DirectoryLock::acquire(lock_root)?;

    let prepared = prepare(ServeOptions {
        target: target.to_path_buf(),
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::from_env()?,
        key_store_path: config_dir.map(|dir| dir.join("llm-keys.json")),
        // UI settings (the custom window title) persist beside the keys;
        // ui_settings_file() is the same path the launch sequence reads to
        // name the native window before any page exists.
        ui_settings_path: config_dir.map(ui_settings_file),
    })
    .await?;

    // The powers the engine does not have on its own. It is an axum router:
    // it can commit a scaffold, and it cannot open a window. This is where the
    // window comes from — passed in rather than built here, so a test can
    // start the engine without an `AppHandle` and get a program that honestly
    // has no windows. See `hickory_cli::serve::Shell`.
    if let Some(shell) = shell {
        prepared.state.set_shell(shell);
    }

    // The up-loop runs beside the rooms: external edits (vim, formatters,
    // coding agents) reconcile into the live editor, and edits saved in
    // generated files carry back into their documents while the app is open.
    let watch = hickory_cli::serve::watch::spawn(prepared.state.clone())?;

    // The UI is the fallback, so every `/api` route the CLI defined wins and
    // anything else is a page request.
    let router = prepared.router.fallback(ui_handler);

    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, dev.serve_port.unwrap_or(0)));
    let listener = tokio::net::TcpListener::bind(addr).await.with_context(|| {
        match dev.serve_port {
            // A named port can be held by something else, and the something
            // else is nearly always a previous run of this app. Say which
            // command releases it rather than leaving a bare "address in use".
            Some(port) => format!(
                "binding {addr}: something already holds port {port}. That is usually a                  `just dev` still running — `just dev-stop` releases it."
            ),
            None => format!("binding {addr}"),
        }
    })?;
    let bound = listener.local_addr()?;

    tokio::spawn(async move {
        if let Err(e) = axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        {
            log::error!("local server stopped: {e:#}");
        }
    });

    Ok(Session {
        // Where the ENGINE is, always. Where the window points is a separate
        // question the caller answers, because in development it is Vite.
        url: format!("http://{bound}"),
        ui_url: dev
            .ui_origin
            .clone()
            .unwrap_or_else(|| format!("http://{bound}")),
        _lock: lock,
        _watch: watch,
    })
}

/// Serve an embedded asset, falling back to `index.html`.
///
/// The fallback is what makes client-side routing work: the app's routes are
/// hash-based today, but a deep link that hits the server as a path must still
/// return the shell rather than a 404.
async fn ui_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Ui::get(path).or_else(|| Ui::get("index.html")) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                [(header::CONTENT_TYPE, mime.as_ref())],
                Body::from(file.data.into_owned()),
            )
                .into_response()
        }
        // Only reachable when the binary was built without a UI, which is a
        // build mistake rather than a user's: say so instead of showing a
        // blank window.
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "This build carries no user interface. Build it with \
             `npm --prefix apps/web run build` before `cargo tauri build`.",
        )
            .into_response(),
    }
}
