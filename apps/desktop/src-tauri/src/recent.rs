//! Machine-local recent paths, shared by desktop windows.
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tauri::{
    AppHandle, Manager, Wry,
    menu::{MenuItemBuilder, Submenu},
};

const LIMIT: usize = 10;

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct History {
    files: Vec<PathBuf>,
    folders: Vec<PathBuf>,
}

impl History {
    fn load(config: &Path) -> Self {
        let mut history: Self = fs::read(config.join("recent-paths.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        // Preserve the last path remembered by older releases.
        if history.files.is_empty()
            && history.folders.is_empty()
            && let Some(path) = crate::server::last_opened(config)
        {
            history.add(&path);
        }
        history.files.retain(|p| p.is_file());
        history.folders.retain(|p| p.is_dir());
        history.files.truncate(LIMIT);
        history.folders.truncate(LIMIT);
        history
    }

    fn add(&mut self, path: &Path) {
        let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let list = if path.is_file() {
            &mut self.files
        } else if path.is_dir() {
            &mut self.folders
        } else {
            return;
        };
        list.retain(|old| old != &path);
        list.insert(0, path);
        list.truncate(LIMIT);
    }
}

fn remember(config: &Path, path: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(config)?;
    // Multiple app processes share the list; lock the read/modify/write act.
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(config.join("recent-paths.lock"))?;
    lock.lock()?;
    let mut history = History::load(config);
    history.add(path);
    let mut temp = tempfile::NamedTempFile::new_in(config)?;
    temp.write_all(&serde_json::to_vec(&history)?)?;
    temp.as_file().sync_all()?;
    temp.persist(config.join("recent-paths.json"))?;
    Ok(())
}

pub struct Menus {
    pub files: Submenu<Wry>,
    pub folders: Submenu<Wry>,
}

impl Menus {
    pub fn new(handle: &AppHandle) -> tauri::Result<Self> {
        let menus = Self {
            files: Submenu::with_id(handle, "recent-files", "Recent Files", true)?,
            folders: Submenu::with_id(handle, "recent-folders", "Recent Folders", true)?,
        };
        menus.refresh(handle)?;
        Ok(menus)
    }

    fn refresh(&self, handle: &AppHandle) -> tauri::Result<()> {
        let history = handle
            .path()
            .app_config_dir()
            .ok()
            .map(|dir| History::load(&dir))
            .unwrap_or_default();
        for (menu, prefix, paths, empty) in [
            (
                &self.files,
                "recent-file:",
                history.files,
                "No Recent Files",
            ),
            (
                &self.folders,
                "recent-folder:",
                history.folders,
                "No Recent Folders",
            ),
        ] {
            for item in menu.items()? {
                menu.remove(&item)?;
            }
            if paths.is_empty() {
                menu.append(&MenuItemBuilder::new(empty).enabled(false).build(handle)?)?;
            }
            for path in paths {
                let text = path.to_string_lossy();
                menu.append(
                    &MenuItemBuilder::with_id(format!("{prefix}{text}"), text).build(handle)?,
                )?;
            }
        }
        Ok(())
    }
}

pub fn record(handle: &AppHandle, path: &Path) {
    if let Ok(config) = handle.path().app_config_dir()
        && let Err(error) = remember(&config, path)
    {
        log::warn!("could not remember recent path: {error:#}");
    }
    refresh(handle);
}

pub fn refresh(handle: &AppHandle) {
    // Apply a whole refresh on the event-loop thread so simultaneous HTTP
    // navigation and focus events cannot interleave removals and appends.
    let app = handle.clone();
    if let Err(error) = handle.run_on_main_thread(move || {
        if let Some(menus) = app.try_state::<Menus>()
            && let Err(error) = menus.refresh(&app)
        {
            log::warn!("could not refresh recent menus: {error}");
        }
    }) {
        log::warn!("could not schedule recent menu refresh: {error}");
    }
}

/// The page reports the active file, including tree/search navigation.
/// The route is desktop-only and accepts existing files inside this workspace.
pub fn routes(
    router: axum::Router,
    handle: AppHandle,
    target: &Path,
    ui_origin: Option<String>,
) -> axum::Router {
    use axum::{
        Json,
        http::{HeaderMap, StatusCode},
        routing::post,
    };
    #[derive(Deserialize)]
    struct Opened {
        path: PathBuf,
    }
    let root = if target.is_file() {
        target.parent().unwrap_or(target)
    } else {
        target
    };
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    router.route(
        "/api/recent-files",
        post(move |headers: HeaderMap, Json(opened): Json<Opened>| {
            let handle = handle.clone();
            let root = root.clone();
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
                    return StatusCode::FORBIDDEN;
                }
                let Ok(path) = root.join(opened.path).canonicalize() else {
                    return StatusCode::NOT_FOUND;
                };
                if !path.starts_with(&root) || !path.is_file() {
                    return StatusCode::BAD_REQUEST;
                }
                match tokio::task::spawn_blocking(move || record(&handle, &path)).await {
                    Ok(()) => StatusCode::NO_CONTENT,
                    Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
                }
            }
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_persist_separately_newest_first_without_duplicates_or_missing_entries() {
        let config = tempfile::tempdir().unwrap();
        let workspace = tempfile::tempdir().unwrap();
        let mut files = Vec::new();
        for i in 0..12 {
            let path = workspace.path().join(format!("{i}.md"));
            fs::write(&path, "note").unwrap();
            remember(config.path(), &path).unwrap();
            files.push(path.canonicalize().unwrap());
        }
        remember(config.path(), workspace.path()).unwrap();
        remember(config.path(), &files[5]).unwrap();
        let history = History::load(config.path());
        assert_eq!(history.files.len(), LIMIT);
        assert_eq!(history.files[0], files[5]);
        assert_eq!(history.files[1], files[11]);
        assert_eq!(history.files.iter().filter(|p| **p == files[5]).count(), 1);
        assert_eq!(
            history.folders,
            vec![workspace.path().canonicalize().unwrap()]
        );
        fs::remove_file(&files[5]).unwrap();
        assert!(!History::load(config.path()).files.contains(&files[5]));
    }

    #[test]
    fn legacy_last_path_is_preserved_and_damaged_history_recovers() {
        let config = tempfile::tempdir().unwrap();
        let workspace = tempfile::tempdir().unwrap();
        crate::server::remember(config.path(), workspace.path());
        fs::write(config.path().join("recent-paths.json"), "{broken").unwrap();
        assert_eq!(
            History::load(config.path()).folders,
            vec![workspace.path().canonicalize().unwrap()]
        );
        let file = workspace.path().join("note.md");
        fs::write(&file, "note").unwrap();
        remember(config.path(), &file).unwrap();
        assert_eq!(
            History::load(config.path()).files,
            vec![file.canonicalize().unwrap()]
        );
    }
}
