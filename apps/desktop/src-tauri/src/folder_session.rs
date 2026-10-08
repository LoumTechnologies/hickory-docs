//! Attach the first folder without replacing the window or its editor origin.
use crate::server::Session;
use axum::{
    Router,
    body::{Body, to_bytes},
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::any,
};
use hickory_cli::serve::{Shell, store::DocIndex};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::{Mutex, RwLock};
use tower::ServiceExt;

pub(crate) struct FolderSession {
    router: Arc<RwLock<Router>>,
    original: Router,
    index: Arc<DocIndex>,
    files: Arc<std::sync::RwLock<HashSet<PathBuf>>>,
    attachment: Mutex<Attachment>,
    folder_open: Arc<AtomicBool>,
}
struct Attachment {
    folder_open: bool,
    _session: Option<Box<Session>>,
}

impl FolderSession {
    pub(crate) fn host(
        router: Router,
        folder_open: bool,
        index: Arc<DocIndex>,
        files: Arc<std::sync::RwLock<HashSet<PathBuf>>>,
    ) -> (Router, Self) {
        let active = Arc::new(RwLock::new(router.clone()));
        let service = active.clone();
        let folder_flag = Arc::new(AtomicBool::new(folder_open));
        let tracking = folder_flag.clone();
        let tracked_index = index.clone();
        let tracked_files = files.clone();
        let hosted = Router::new().fallback(any(move |request: Request| {
            let service = service.clone();
            let index = tracked_index.clone();
            let files = tracked_files.clone();
            let track_open =
                !tracking.load(Ordering::SeqCst) && request.uri().path() == "/api/open-file";
            async move {
                let router = service.read().await.clone();
                let response = router.oneshot(request).await?;
                if !track_open || !response.status().is_success() {
                    return Ok::<_, std::convert::Infallible>(response);
                }
                // Record only selections the existing engine validated. This
                // also covers files opened after a single-file launch.
                let (parts, body) = response.into_parts();
                let bytes = match to_bytes(body, 64 * 1024).await {
                    Ok(bytes) => bytes,
                    Err(_) => return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
                };
                if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes)
                    && let Some(path) = value["path"].as_str()
                {
                    if value["doc_id"].is_string() {
                        index.add(path);
                    }
                    let absolute = index.root().join(path);
                    if let Ok(path) = absolute.canonicalize() {
                        files.write().unwrap().insert(path);
                    }
                }
                Ok(Response::from_parts(parts, Body::from(bytes)))
            }
        }));
        (
            hosted,
            Self {
                router: active,
                original: router,
                index,
                files,
                folder_open: folder_flag,
                attachment: Mutex::new(Attachment {
                    folder_open,
                    _session: None,
                }),
            },
        )
    }
}

impl Session {
    /// True when this attached the window's first folder; false means the
    /// window already has one and the shell should open a separate window.
    pub async fn attach_folder(
        &self,
        target: &Path,
        config: Option<&Path>,
        shell: Option<Shell>,
        handle: Option<tauri::AppHandle>,
    ) -> anyhow::Result<bool> {
        let routing = &self.folders;
        let mut attachment = routing.attachment.lock().await;
        if attachment.folder_open {
            return Ok(false);
        }
        anyhow::ensure!(target.is_dir(), "not a folder: {}", target.display());
        let session = crate::server::start_with_menu(target, config, shell, handle).await?;
        let original = routing.original.clone();
        let workspace = session.folders.router.read().await.clone();
        // Carry the exact selected-file grants into the folder engine before
        // switching routing. Plain-file reads and saves then share one engine.
        let selected: Vec<_> = routing.files.read().unwrap().iter().cloned().collect();
        for path in selected {
            let request = Request::builder()
                .method("POST")
                .uri("/api/open-file")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(
                    &serde_json::json!({ "path": path }),
                )?))?;
            let response = workspace.clone().oneshot(request).await?;
            anyhow::ensure!(
                response.status().is_success(),
                "could not carry the open file {} into the folder session",
                path.display()
            );
        }
        let index = routing.index.clone();
        let combined = Router::new().fallback(any(move |request: Request| {
            let original = original.clone();
            let workspace = workspace.clone();
            let index = index.clone();
            async move {
                let path = request.uri().path();
                let url = tauri::Url::parse(&format!("http://localhost{}", request.uri()))
                    .expect("request URI");
                let doc = path
                    .strip_prefix("/api/docs/")
                    .and_then(|rest| rest.split('/').next())
                    .map(str::to_owned)
                    .or_else(|| {
                        url.query_pairs()
                            .find(|(key, _)| key == "doc")
                            .map(|(_, value)| {
                                value.strip_prefix("doc:").unwrap_or(&value).to_owned()
                            })
                    });
                let existing_doc = doc.is_some_and(|id| index.path_of(&id).is_some());
                // Keep existing document rooms and window recovery
                // on their original session. Folder operations use the engine.
                let window_storage = path == "/api/workspace/ui" || path == "/api/workspace/drafts";
                if existing_doc || window_storage {
                    original.oneshot(request).await
                } else {
                    workspace.oneshot(request).await
                }
            }
        }));
        let mut active = routing.router.write().await;
        routing.folder_open.store(true, Ordering::SeqCst);
        *active = combined;
        attachment._session = Some(Box::new(session));
        attachment.folder_open = true;
        Ok(true)
    }
}
