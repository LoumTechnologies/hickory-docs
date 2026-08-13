//! The Hickory Docs desktop shell.
//!
//! A native window around the local document server — the same engine `hick
//! up` is built from, running in this process on loopback. The app and the CLI
//! are two front doors onto one implementation, which is the point of
//! `docs/specs/freeform/local-only.md`: there is no second code path for
//! weaving, lineage, or carrying an output edit back into a document.
//!
//! Nothing here reaches the network. There is no account, no telemetry, and no
//! update check.
//!
//! ## Why the launch sequence looks like this
//!
//! The window's URL is the address the server bound to, so the engine has to
//! start before there is a window to show. But choosing a folder needs a modal
//! dialog, which needs an event loop already running. The resolution is to
//! start Tauri with no window at all and do the whole sequence — pick, start,
//! open — on a worker thread, which is also where the blocking dialogs belong:
//! calling them on the main thread deadlocks against the event loop they are
//! waiting on.

pub mod server;

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager as _, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_dialog::{DialogExt as _, MessageDialogKind};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();
            // Off the main thread: everything below either blocks on a dialog
            // or blocks on the runtime, and both would wedge the event loop.
            std::thread::spawn(move || launch(handle));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Hickory Docs");
}

/// Choose a folder, start the engine in it, and open the window.
///
/// Loops rather than failing once: the two ways this goes wrong — a folder
/// with no documents in it, and a folder another Hickory Docs process already
/// holds — are both things the user fixes by choosing a different folder, so
/// the app says what happened and asks again.
fn launch(handle: AppHandle) {
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => return fail(&handle, &format!("Could not start the runtime.\n\n{e}")),
    };

    let config_dir = handle.path().app_config_dir().ok();

    // A folder named on the command line or in the environment is a deliberate
    // act, and is tried once: someone who typed a path wants that path, and
    // falling back to a picker would quietly hide their typo.
    if let Some(dir) = server::named_dir() {
        match runtime.block_on(server::start(&dir)) {
            Ok(session) => open(&handle, runtime, session, config_dir.as_deref(), &dir),
            Err(e) => fail(
                &handle,
                &format!("Could not open {}\n\n{e:#}", dir.display()),
            ),
        }
        return;
    }

    // Otherwise: the folder from last time, then ask.
    let mut candidate = config_dir.as_deref().and_then(server::last_opened);

    loop {
        let dir = match candidate.take() {
            Some(dir) => dir,
            None => match ask_for_folder(&handle) {
                Some(dir) => dir,
                // Cancelling the picker with nothing open is a decision not to
                // use the app right now, not an error to report.
                None => {
                    handle.exit(0);
                    return;
                }
            },
        };

        match runtime.block_on(server::start(&dir)) {
            Ok(session) => {
                open(&handle, runtime, session, config_dir.as_deref(), &dir);
                return;
            }
            Err(e) => {
                handle
                    .dialog()
                    .message(format!("Could not open {}\n\n{e:#}", dir.display()))
                    .kind(MessageDialogKind::Warning)
                    .title("Choose another folder")
                    .blocking_show();
            }
        }
    }
}

/// Show the native folder chooser.
fn ask_for_folder(handle: &AppHandle) -> Option<PathBuf> {
    handle
        .dialog()
        .file()
        .set_title("Open a folder of .hick documents")
        .blocking_pick_folder()
        .and_then(|p| p.into_path().ok())
}

/// Open the window on a running session, and remember the folder.
///
/// `runtime` and `session` are moved into the app's managed state: the runtime
/// owns the server task and the session holds the directory lock, so both have
/// to live exactly as long as the app does.
fn open(
    handle: &AppHandle,
    runtime: tokio::runtime::Runtime,
    session: server::Session,
    config_dir: Option<&Path>,
    dir: &Path,
) {
    if let Some(config_dir) = config_dir {
        server::remember(config_dir, dir);
    }

    let url = session.url.clone();
    handle.manage(session);
    handle.manage(runtime);

    let built = url
        .parse::<tauri::Url>()
        .map_err(|e| e.to_string())
        .and_then(|url| {
            WebviewWindowBuilder::new(handle, "main", WebviewUrl::External(url))
                .title(format!("Hickory Docs — {}", folder_name(dir)))
                .inner_size(1100.0, 780.0)
                .min_inner_size(360.0, 480.0)
                .build()
                .map_err(|e| e.to_string())
        });

    if let Err(e) = built {
        fail(handle, &format!("Could not open the window.\n\n{e}"));
    }
}

/// The folder's own name, for the title bar. A full path makes a long and
/// mostly useless title; the name is what the user calls the project.
fn folder_name(dir: &Path) -> String {
    dir.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| dir.display().to_string())
}

/// Report a fatal startup failure, then exit.
///
/// A dialog rather than stderr: this process is normally launched by
/// double-clicking an icon, so an error that exists only in a terminal does
/// not exist. It is printed as well, for anyone who did start it from one.
fn fail(handle: &AppHandle, message: &str) {
    eprintln!("hickory-desktop: {message}");
    handle
        .dialog()
        .message(message)
        .kind(MessageDialogKind::Error)
        .title("Hickory Docs could not start")
        .blocking_show();
    handle.exit(1);
}
