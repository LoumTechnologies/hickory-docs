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
use hickory_cli::serve::{ServeOptions, prepare};
use hickory_cli::up::DirectoryLock;

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
    pub url: String,
    /// Held for the process's lifetime. Dropping it releases the directory.
    _lock: DirectoryLock,
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

/// Start the engine on loopback and return the address to load.
///
/// Binds port 0: the app has no reason to want a particular port, and asking
/// for one means failing to start because something unrelated already holds
/// it.
pub async fn start(target: &Path) -> Result<Session> {
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
    })
    .await?;

    // The UI is the fallback, so every `/api` route the CLI defined wins and
    // anything else is a page request.
    let router = prepared.router.fallback(ui_handler);

    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 0));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("binding {addr}"))?;
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
        url: format!("http://{bound}"),
        _lock: lock,
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
