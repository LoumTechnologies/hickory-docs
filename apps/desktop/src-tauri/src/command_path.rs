//! User-level installation of the bundled CLI. No arbitrary paths over HTTP.
use anyhow::{Context, Result, bail};
use axum::{
    Json, Router,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[cfg(unix)]
#[path = "command_path_unix.rs"]
mod platform;
#[cfg(windows)]
#[path = "command_path_windows.rs"]
mod platform;

#[derive(Clone)]
pub struct Installer {
    pub source: PathBuf,
    #[cfg(unix)]
    pub home: PathBuf,
    #[cfg(unix)]
    pub shell: String,
    #[cfg(unix)]
    pub appimage: Option<PathBuf>,
    pub search_path: std::ffi::OsString,
}

#[derive(Serialize)]
pub struct Status {
    pub available: bool,
    pub installed: bool,
    pub can_remove: bool,
    pub command: PathBuf,
    pub source: PathBuf,
    pub conflict: Option<PathBuf>,
    pub message: String,
}

impl Installer {
    fn discover() -> Result<Self> {
        let executable = std::env::current_exe()?;
        let source = executable
            .parent()
            .context("No application directory")?
            .join(if cfg!(windows) { "hick.exe" } else { "hick" });
        #[cfg(unix)]
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .context("No user home directory; cannot install the command")?;
        Ok(Self {
            source,
            #[cfg(unix)]
            home,
            #[cfg(unix)]
            shell: std::env::var("SHELL").unwrap_or_default(),
            #[cfg(unix)]
            appimage: std::env::var_os("APPIMAGE").map(PathBuf::from),
            search_path: std::env::var_os("PATH").unwrap_or_default(),
        })
    }

    pub fn status(&self) -> Result<Status> {
        platform::status(self)
    }
    pub fn install(&self) -> Result<Status> {
        platform::install(self).context("Cannot install the hick command. Check permissions on your user PATH directory and shell profiles, then retry")?;
        self.status()
    }
    pub fn remove(&self) -> Result<Status> {
        platform::remove(self).context("Cannot remove the hick command. Check permissions on your user PATH directory and shell profiles, then retry")?;
        self.status()
    }

    fn conflict(&self, destination: &Path) -> Option<PathBuf> {
        let path = &self.search_path;
        std::env::split_paths(path)
            .map(|dir| dir.join(if cfg!(windows) { "hick.exe" } else { "hick" }))
            .find(|path| {
                path.is_file()
                    && path != destination
                    && path.canonicalize().ok() != self.source.canonicalize().ok()
            })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Action {
    Install,
    Remove,
}
#[derive(Deserialize)]
struct Request {
    action: Action,
}

/// Shared by folderless and workspace windows, at the desktop's own origin.
pub fn routes(router: Router, ui_origin: Option<String>) -> Router {
    let installer = Arc::new(Mutex::new(Installer::discover()));
    let read = installer.clone();
    router.route(
        "/api/settings/command-path",
        get(move || {
            let read = read.clone();
            async move { perform(read, None).await }
        })
        .post(move |headers: HeaderMap, Json(request): Json<Request>| {
            let installer = installer.clone();
            let ui_origin = ui_origin.clone();
            async move {
                let own = format!(
                    "http://{}",
                    headers
                        .get("host")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or_default()
                );
                if let Some(origin) = headers.get("origin").and_then(|v| v.to_str().ok())
                    && origin != own
                    && ui_origin.as_deref() != Some(origin)
                {
                    return StatusCode::FORBIDDEN.into_response();
                }
                perform(installer, Some(request.action)).await
            }
        }),
    )
}

async fn perform(installer: Arc<Mutex<Result<Installer>>>, action: Option<Action>) -> Response {
    let outcome = tokio::task::spawn_blocking(move || -> Result<Status> {
        let guard = installer
            .lock()
            .map_err(|_| anyhow::anyhow!("Command installer is busy; retry"))?;
        let installer = guard
            .as_ref()
            .map_err(|error| anyhow::anyhow!("{error:#}"))?;
        match action {
            None => installer.status(),
            Some(Action::Install) => installer.install(),
            Some(Action::Remove) => installer.remove(),
        }
    })
    .await;
    match outcome {
        Ok(Ok(status)) => Json(status).into_response(),
        other => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": match other {
                Ok(Err(error)) => format!("{error:#}"),
                Err(error) => format!("Command setup did not finish: {error}"),
                _ => unreachable!(),
            }})),
        )
            .into_response(),
    }
}

fn require_available(status: &Status) -> Result<()> {
    if !status.available {
        bail!(
            "This build has no bundled hick CLI at {}. Install a packaged desktop release.",
            status.source.display()
        );
    }
    if let Some(conflict) = &status.conflict {
        bail!(
            "Another hick command is installed at {}. Remove that installation or adjust PATH before installing this one.",
            conflict.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    // Guarantee: docs/guarantees/release/the-app-installs-its-command-for-the-user.md
    #[tokio::test]
    async fn desktop_status_is_readable_and_other_origins_cannot_install() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            axum::serve(listener, routes(Router::new(), None))
                .await
                .unwrap();
        });
        let client = reqwest::Client::new();
        let endpoint = format!("{url}/api/settings/command-path");
        let status: serde_json::Value = client
            .get(&endpoint)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert!(status["available"].is_boolean());
        assert!(status["command"].is_string());
        let blocked = client
            .post(&endpoint)
            .header("Origin", "https://other.example")
            .json(&serde_json::json!({"action":"install"}))
            .send()
            .await
            .unwrap();
        assert_eq!(blocked.status(), StatusCode::FORBIDDEN);
        let invalid = client
            .post(&endpoint)
            .json(&serde_json::json!({"action":"replace"}))
            .send()
            .await
            .unwrap();
        assert_eq!(invalid.status(), StatusCode::UNPROCESSABLE_ENTITY);
        server.abort();
    }
}
