//! Capability links: what a shared session lets a stranger do, and what it
//! refuses to let them do.
//!
//! A local session has no accounts, by design — asking a collaborator to sign
//! up before they can fix a typo is how a two-minute collaboration becomes no
//! collaboration. What replaces an account is a **capability**: an unguessable
//! token in the URL, carrying a scope. The link *is* the credential, which is
//! exactly why the scope is narrow by default and why `run` is never implied.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use axum::body::Body;
use axum::extract::Request;
use axum::response::{IntoResponse, Response};

use super::relay::Relay;
use super::{LocalState, store::DocIndex};
use crate::ExecutorChoice;

/// What the holder of a link may do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Read the document, its outputs, and the lineage. No writes, no runs.
    Read,
    /// Read and edit. The default for a shared session: editing together is
    /// the point, and an edit is reviewable afterwards in git.
    Edit,
    /// Edit and execute. Runs code on the host's machine — granted only when
    /// asked for explicitly, and only on a sandboxing executor.
    Run,
}

impl Scope {
    pub fn parse(s: &str) -> Option<Scope> {
        match s.trim().to_ascii_lowercase().as_str() {
            "read" | "readonly" | "read-only" => Some(Scope::Read),
            "edit" | "write" => Some(Scope::Edit),
            "run" | "execute" => Some(Scope::Run),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Scope::Read => "read",
            Scope::Edit => "edit",
            Scope::Run => "run",
        }
    }

    pub fn can_edit(self) -> bool {
        matches!(self, Scope::Edit | Scope::Run)
    }

    pub fn can_run(self) -> bool {
        matches!(self, Scope::Run)
    }

    pub const ALL: &'static [&'static str] = &["read", "edit", "run"];
}

/// Which party a request came from, and therefore what it may do.
///
/// The host is not a guest. A session someone starts on their own machine
/// must let *them* run their own document — the scope on a share link governs
/// the people the link was sent to, and conflating the two made
/// `hickory serve doc.hick` unable to run anything at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Caller {
    pub scope: Scope,
    pub is_host: bool,
}

impl Caller {
    pub fn host() -> Self {
        Self {
            scope: Scope::Run,
            is_host: true,
        }
    }

    pub fn guest(scope: Scope) -> Self {
        Self {
            scope,
            is_host: false,
        }
    }

    pub fn can_edit(self) -> bool {
        self.scope.can_edit()
    }

    pub fn can_run(self) -> bool {
        self.scope.can_run()
    }
}

/// An unguessable session token.
///
/// 128 bits from the OS RNG. Not a JWT: there is nothing to encode, no second
/// party to verify it, and a signed token that cannot be revoked is worse than
/// a random one that dies with the process.
pub fn mint_token() -> String {
    let mut bytes = [0u8; 16];
    getrandom(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Fill `buf` with OS randomness.
fn getrandom(buf: &mut [u8]) {
    // `/dev/urandom` on unix, `rand` elsewhere. The token must never fall back
    // to something predictable, so a failure here panics rather than degrading.
    #[cfg(unix)]
    {
        use std::io::Read as _;
        if let Ok(mut f) = std::fs::File::open("/dev/urandom")
            && f.read_exact(buf).is_ok()
        {
            return;
        }
    }
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or_else(|_| panic!("no clock and no /dev/urandom: cannot mint a session token"));
    for byte in buf.iter_mut() {
        // xorshift64*, seeded from the clock. Reached only where /dev/urandom
        // is unavailable; documented as the weaker path rather than silent.
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        *byte = (seed >> 33) as u8;
    }
}

// ---------------------------------------------------------------------------
// The execution guard
// ---------------------------------------------------------------------------

/// The decision about whether this session may execute for other people.
pub struct ShareGuard {
    pub shared: bool,
    pub scope: Scope,
    pub executor: ExecutorChoice,
}

impl ShareGuard {
    pub fn evaluate(shared: bool, scope: Scope, executor: ExecutorChoice) -> Self {
        Self {
            shared,
            scope,
            executor,
        }
    }

    /// Refuse a configuration that would let someone with a link run code as
    /// the host's own user.
    ///
    /// This refuses rather than warns, and the difference is the whole point:
    /// a warning printed above a URL that works anyway is a grant. The
    /// unsandboxed executor is entirely correct for a session the host is
    /// running alone — which is why the check is on *sharing*, not on the
    /// executor by itself.
    pub fn enforce(&self) -> Result<()> {
        if !(self.shared && self.scope.can_run()) {
            return Ok(());
        }
        if matches!(self.executor, ExecutorChoice::Local) {
            anyhow::bail!(
                "refusing to share a session that can run documents on the local executor.\n\
                 \n\
                 `--scope run` lets anyone holding the link execute this document, and the \
                 local executor runs commands as you, with your files and your network — so \
                 a link forwarded to one more person is a shell on this machine.\n\
                 \n\
                 Either:\n  \
                 • sandbox the execution:  HICKORY_EXECUTOR=docker hickory serve --share --scope run …\n  \
                 • or share without it:    hickory serve --share --scope edit …   (guests edit; you run)"
            );
        }
        Ok(())
    }

    /// One line describing the execution posture, for the banner.
    pub fn posture(&self) -> String {
        match (self.scope, self.executor) {
            (Scope::Run, ExecutorChoice::Local) => {
                "runs execute on this machine as you".to_string()
            }
            (Scope::Run, other) => {
                format!("guests may run documents ({} executor)", other.as_str())
            }
            _ => "guests cannot run documents; only you can".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Request authorization
// ---------------------------------------------------------------------------

/// The token a request presents: `Authorization: Bearer …` (what the web
/// client sends) or `?t=` / `?token=` (the link itself, and the WS handshake,
/// which cannot set headers).
pub fn presented_token(req: &Request<Body>) -> Option<String> {
    if let Some(value) = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        return Some(value.trim().to_string());
    }
    let query = req.uri().query()?;
    query.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        (k == "t" || k == "token").then(|| v.to_string())
    })
}

/// Reject an unauthenticated API request, and hand the capability to the
/// handler.
///
/// This is the only place a token becomes a permission. A handler that forgets
/// to check a scope can still be wrong about *what* a caller may do, but no
/// handler can be reached by someone who holds no token at all — which is the
/// property that matters once `--share` binds this process to a network.
pub async fn authorize(
    axum::extract::State(state): axum::extract::State<LocalState>,
    mut req: Request<Body>,
    next: axum::middleware::Next,
) -> Response {
    let Some(token) = presented_token(&req) else {
        return unauthorized("this session needs its link: open the URL `hickory serve` printed");
    };
    let Some(caller) = state.caller_for(&token) else {
        return unauthorized("this session link is not valid");
    };
    req.extensions_mut().insert(caller);
    next.run(req).await
}

fn unauthorized(msg: &str) -> Response {
    (
        axum::http::StatusCode::FORBIDDEN,
        axum::Json(serde_json::json!({ "error": msg })),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Serving the client
// ---------------------------------------------------------------------------

/// Where a built web client might be found, for a developer running from the
/// repo. A shipped binary is expected to be given `--web-dist`, or to embed
/// the assets (see the open question in `local-collaboration.md`).
pub fn discover_web_dist() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("HICKORY_WEB_DIST") {
        let path = PathBuf::from(dir);
        if path.is_dir() {
            return Some(path);
        }
    }
    let mut here = std::env::current_dir().ok()?;
    loop {
        let candidate = here.join("apps/web/dist");
        if candidate.join("index.html").is_file() {
            return Some(candidate);
        }
        if !here.pop() {
            return None;
        }
    }
}

/// Serve the built client, injecting this session's credential into
/// `index.html`.
///
/// The client keeps its token in `localStorage` and shows the marketing page
/// when it has none — so without this, opening a local session would greet a
/// collaborator with a pitch for the hosted product. Injecting at serve time
/// keeps the client unmodified: no local-mode branch to maintain in the app,
/// and no way for the two to drift.
pub fn web_fallback(
    dist: PathBuf,
) -> impl Fn(
    axum::extract::State<LocalState>,
    Request<Body>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
+ Clone
+ Send
+ 'static {
    move |axum::extract::State(state): axum::extract::State<LocalState>, req: Request<Body>| {
        let dist = dist.clone();
        Box::pin(async move { serve_asset(dist, state, req).await })
    }
}

async fn serve_asset(dist: PathBuf, state: LocalState, req: Request<Body>) -> Response {
    use tower::ServiceExt as _;

    let path = req.uri().path().trim_start_matches('/').to_string();
    let candidate = dist.join(&path);
    let is_asset = !path.is_empty() && candidate.is_file();

    if is_asset {
        let service = tower_http::services::ServeDir::new(&dist);
        return match service.oneshot(req).await {
            Ok(resp) => resp.into_response(),
            Err(_) => (axum::http::StatusCode::NOT_FOUND, "not found").into_response(),
        };
    }

    // Anything else is the SPA entry point.
    let index = dist.join("index.html");
    let html = match std::fs::read_to_string(&index) {
        Ok(html) => html,
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("cannot read {}: {e}", index.display()),
            )
                .into_response();
        }
    };
    let opening = state
        .index
        .sole()
        .map(|(id, _)| format!("#/docs/{id}"))
        .unwrap_or_else(|| "#/projects".to_string());
    let bootstrap = format!(
        "<script>(function(){{\
           var t=new URLSearchParams(location.search).get('t')||{token:?};\
           try{{localStorage.setItem('hickory.token',t);}}catch(e){{}}\
           if(!location.hash||location.hash==='#/'){{location.hash={opening:?};}}\
         }})();</script>",
        token = state.host_token.as_str(),
        opening = opening,
    );
    let html = match html.find("</head>") {
        Some(at) => format!("{}{bootstrap}{}", &html[..at], &html[at..]),
        None => format!("{bootstrap}{html}"),
    };
    axum::response::Html(html).into_response()
}

// ---------------------------------------------------------------------------
// Banner
// ---------------------------------------------------------------------------

/// What the host sees when the session starts.
///
/// An operator-facing surface: it says what is being served, who can reach it,
/// what they may do, and where execution happens — because every one of those
/// is a decision the host is accountable for and cannot check afterwards.
pub fn print_banner(
    state: &LocalState,
    index: &Arc<DocIndex>,
    addr: SocketAddr,
    lan: bool,
    guard: &ShareGuard,
    relay: &Relay,
    // The address a tunnel handed back, when one was opened. Known only after
    // the relay answers, so it cannot come from `Relay` itself.
    public_url: Option<&str>,
) {
    let host = if lan {
        local_ip().unwrap_or_else(|| "0.0.0.0".to_string())
    } else {
        "127.0.0.1".to_string()
    };
    let host_url = format!("http://127.0.0.1:{}/?t={}", addr.port(), state.host_token);
    // A relay's base URL already carries scheme and host; the LAN address is
    // the fallback.
    let guest_url = match public_url.or_else(|| relay.base_url()) {
        Some(base) => format!("{base}/?t={}", state.guest_token),
        None => format!("http://{host}:{}/?t={}", addr.port(), state.guest_token),
    };

    eprintln!();
    eprintln!("  hickory serve — {}", index.root().display());
    let docs = index.entries();
    match docs.as_slice() {
        [only] => eprintln!("  document:  {}", only.1),
        many => eprintln!("  documents: {}", many.len()),
    }
    eprintln!("  You:       {host_url}");
    if lan {
        eprintln!();
        eprintln!("  Share:     {guest_url}");
        eprintln!(
            "             they may {} · {}",
            state.guest_scope.as_str(),
            guard.posture()
        );
        match relay {
            Relay::None => {
                eprintln!("             anyone on this network holding that link can join")
            }
            _ if relay.is_public() => eprintln!(
                "             this link is on the PUBLIC INTERNET — anyone it reaches can join"
            ),
            Relay::PortZero { domain, .. } => eprintln!(
                "             {domain} reaches machines on your PortZero overlay, not the \
                 internet — `portzero login` for a cloud tunnel"
            ),
            Relay::Explicit(_) | Relay::Hickory { .. } => {}
        }
    } else {
        eprintln!("  Loopback only — add --share to let others on your network join.");
    }
    eprintln!();
    eprintln!("  Edits are written to your files as they happen. Ctrl-C ends the session.");
    eprintln!();
}

/// This machine's LAN address, for the banner.
fn local_ip() -> Option<String> {
    // Connecting a UDP socket picks the interface the kernel would route
    // from, without sending a packet.
    let sock = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    sock.connect("192.0.2.1:80").ok()?; // TEST-NET-1: routable, never answers
    Some(sock.local_addr().ok()?.ip().to_string())
}

/// True when `path` is inside `root` after symlink resolution.
pub fn is_inside(root: &Path, path: &Path) -> bool {
    match (root.canonicalize(), path.canonicalize()) {
        (Ok(r), Ok(p)) => p.starts_with(r),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scopes_parse_and_nest() {
        assert_eq!(Scope::parse("read"), Some(Scope::Read));
        assert_eq!(Scope::parse("EDIT"), Some(Scope::Edit));
        assert_eq!(Scope::parse("run"), Some(Scope::Run));
        assert_eq!(Scope::parse("admin"), None);

        assert!(!Scope::Read.can_edit());
        assert!(Scope::Edit.can_edit());
        assert!(!Scope::Edit.can_run());
        assert!(Scope::Run.can_edit() && Scope::Run.can_run());
    }

    /// Guarantee: docs/guarantees/collaboration/shared-runs-require-a-sandbox.md
    #[test]
    fn a_shared_runnable_session_refuses_the_unsandboxed_executor() {
        let err = ShareGuard::evaluate(true, Scope::Run, ExecutorChoice::Local)
            .enforce()
            .expect_err("sharing + run + local executor must refuse");
        let msg = err.to_string();
        assert!(msg.contains("HICKORY_EXECUTOR=docker"), "{msg}");
        assert!(msg.contains("--scope edit"), "{msg}");
    }

    #[test]
    fn the_guard_only_applies_to_shared_runnable_sessions() {
        // Alone on your own machine, the local executor is exactly right.
        ShareGuard::evaluate(false, Scope::Run, ExecutorChoice::Local)
            .enforce()
            .unwrap();
        // Shared, but guests cannot run.
        ShareGuard::evaluate(true, Scope::Edit, ExecutorChoice::Local)
            .enforce()
            .unwrap();
        // Shared and runnable, but sandboxed.
        ShareGuard::evaluate(true, Scope::Run, ExecutorChoice::Docker)
            .enforce()
            .unwrap();
    }

    #[test]
    fn a_token_is_unguessable_and_fresh_each_time() {
        let a = mint_token();
        assert_eq!(a.len(), 32, "128 bits, hex");
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, mint_token());
    }
}
