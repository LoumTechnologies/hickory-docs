//! Finder supplies document URLs through the event loop, not argv. Keep the
//! cold-launch request until Ready so it selects the workspace and URL fragment
//! before the webview loads. Later requests open their own session/window.
use std::path::PathBuf;
use tauri::{AppHandle, RunEvent};

pub(super) fn handler() -> impl FnMut(&AppHandle, RunEvent) {
    let mut pending = Vec::<PathBuf>::new();
    #[cfg(target_os = "macos")]
    let mut ready = false;
    move |handle, event| match event {
        #[cfg(target_os = "macos")]
        RunEvent::Opened { urls } => {
            let files = urls.into_iter().filter_map(|url| url.to_file_path().ok());
            for file in files {
                if ready {
                    open_separate(handle.clone(), file);
                } else {
                    pending.push(file);
                }
            }
        }
        RunEvent::Ready => {
            #[cfg(target_os = "macos")]
            {
                ready = true;
            }
            let mut files = std::mem::take(&mut pending).into_iter();
            let first = files.next();
            let app = handle.clone();
            // Engine startup and native dialogs must stay off the main thread.
            std::thread::spawn(move || super::launch(app, first));
            for file in files {
                open_separate(handle.clone(), file);
            }
        }
        _ => {}
    }
}

fn open_separate(handle: AppHandle, file: PathBuf) {
    std::thread::spawn(move || {
        if let Err(error) = super::open_folder_in_new_process(&file) {
            super::fail(
                &handle,
                &format!("Could not open {}\n\n{error:#}", file.display()),
            );
        }
    });
}
