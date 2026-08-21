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

use tauri::menu::{Menu, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Manager as _, WebviewUrl, WebviewWindowBuilder, Wry};
use tauri_plugin_dialog::{DialogExt as _, MessageDialogKind};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .menu(app_menu)
        .on_menu_event(|app, event| on_menu(app, event.id().as_ref()))
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

/// The native menu bar.
///
/// Two kinds of item: the ones the webview can answer (New, Save, Save As,
/// Settings — forwarded as a DOM event, see [`dispatch_to_ui`]) and the ones
/// only this process can (Open, which restarts the session on another folder,
/// and Quit). Edit and Window are the platform's own items, predefined so
/// cut/copy/paste behave exactly as the OS says they should.
fn app_menu(handle: &AppHandle) -> tauri::Result<Menu<Wry>> {
    // On macOS the first submenu is the application menu; without it, "File"
    // would be renamed to the app and lose its own items.
    #[cfg(target_os = "macos")]
    let app_submenu = SubmenuBuilder::new(handle, "Hickory Docs")
        .about(None)
        .separator()
        .services()
        .separator()
        .hide()
        .hide_others()
        .show_all()
        .separator()
        .quit()
        .build()?;

    let file = SubmenuBuilder::new(handle, "File")
        .item(
            &MenuItemBuilder::with_id("new", "New Document")
                .accelerator("CmdOrCtrl+N")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("open-file", "Open File…")
                .accelerator("CmdOrCtrl+O")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("open-folder", "Open Folder…")
                .accelerator("CmdOrCtrl+Shift+O")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("save", "Save")
                .accelerator("CmdOrCtrl+S")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("save-as", "Save As…")
                .accelerator("CmdOrCtrl+Shift+S")
                .build(handle)?,
        )
        .item(
            // Every open buffer at once. The app already saves as you type,
            // so this is "flush everything now" rather than "or else it is
            // lost" — which is why it has no urgent accelerator.
            &MenuItemBuilder::with_id("save-all", "Save All")
                .accelerator("CmdOrCtrl+Alt+S")
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("print", "Print…")
                .accelerator("CmdOrCtrl+P")
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("terminal", "New Terminal")
                .accelerator("CmdOrCtrl+Shift+T")
                .build(handle)?,
        )
        .item(
            // The one key that answers "what needs me?" — it walks the
            // attention queue in the order the server ranks it, and says so
            // when nothing is left.
            &MenuItemBuilder::with_id("attention", "Next Needing Attention")
                .accelerator("CmdOrCtrl+J")
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("files", "Show Files")
                .accelerator("CmdOrCtrl+Shift+E")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("settings", "Settings…")
                .accelerator("CmdOrCtrl+,")
                .build(handle)?,
        )
        .separator()
        .quit()
        .build()?;

    let edit = SubmenuBuilder::new(handle, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    // Zoom, at two scopes. The whole window takes the chord everybody
    // already has in their fingers from browsers and editors; adding Alt
    // narrows it to the focused tab, which is the rarer request.
    let view = SubmenuBuilder::new(handle, "View")
        .item(
            &MenuItemBuilder::with_id("zoom-in", "Zoom In")
                .accelerator("CmdOrCtrl+=")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("zoom-out", "Zoom Out")
                .accelerator("CmdOrCtrl+-")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("zoom-reset", "Actual Size")
                .accelerator("CmdOrCtrl+0")
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("zoom-tab-in", "Zoom In This Tab")
                .accelerator("CmdOrCtrl+Alt+=")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("zoom-tab-out", "Zoom Out This Tab")
                .accelerator("CmdOrCtrl+Alt+-")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("zoom-tab-reset", "Actual Size In This Tab")
                .accelerator("CmdOrCtrl+Alt+0")
                .build(handle)?,
        )
        .separator()
        .item(
            // Off by default: a blame column is a permanent indent on every
            // line of every file, answering a question nobody asks most of
            // the time. It earns its place when you are asking it.
            &MenuItemBuilder::with_id("blame", "Show Blame Column")
                .accelerator("CmdOrCtrl+Alt+B")
                .build(handle)?,
        )
        .build()?;

    let insert = insert_menu(handle)?;

    let window = SubmenuBuilder::new(handle, "Window")
        .minimize()
        .close_window()
        .build()?;

    let menu = MenuBuilder::new(handle);
    #[cfg(target_os = "macos")]
    let menu = menu.item(&app_submenu);
    menu.item(&file)
        .item(&edit)
        .item(&view)
        .item(&insert)
        .item(&window)
        .build()
}

type InsertGroups = &'static [(&'static str, &'static [(&'static str, &'static str)])];

/// Every element of the Insert menu, as `(menu id, label)` grouped exactly as
/// the page groups them.
///
/// This is the native half of `apps/web/src/lib/insertCatalog.ts`, which owns
/// the attributes, the hints, and the bytes each element writes — the menu
/// only needs to name one. The two lists are kept in step by
/// `tests/insert_menu_matches_catalogue.rs`; a drift there is a real bug,
/// because a menu offering an element the page has never heard of would open
/// the panel on the wrong thing.
const INSERT_GROUPS: InsertGroups = &[
    (
        "Execution",
        &[
            ("exec", "Exec Cell"),
            ("expect", "Expectation"),
            ("container", "Container"),
            ("needs", "Needs A Program"),
            ("secret", "Secret"),
            ("volume", "Volume"),
            ("agent", "Agent Cell"),
            ("confirm", "Confirmation Gate"),
            ("verify", "Verify Command"),
            ("capture", "Capture"),
            ("feature", "Feature"),
        ],
    ),
    (
        "Capabilities",
        &[
            ("allow-network", "Allow Network"),
            ("deny-network", "Deny Network"),
            ("allow-file-read", "Allow File Read"),
            ("allow-file-write", "Allow File Write"),
            ("volume-access", "Volume Access Rule"),
            ("fork", "Fork A Container"),
            ("attenuate", "Attenuate A Container"),
        ],
    ),
    (
        "Reuse",
        &[("copy", "Copy"), ("cut", "Cut"), ("paste", "Paste")],
    ),
    (
        "Document",
        &[
            ("file", "Generated File"),
            ("var", "Variable"),
            ("val", "Variable Value"),
            ("when", "Conditional Block"),
            ("diagram", "Diagram"),
            ("table", "Table"),
            ("math", "Equation"),
            ("transform", "Transform"),
            ("include", "Include A File"),
            ("upstream", "Upstream Document"),
        ],
    ),
];

/// The menu's element names, for the parity test against the page's
/// catalogue (`tests/insert_menu_matches_catalogue.rs`).
pub fn insert_menu_elements() -> InsertGroups {
    INSERT_GROUPS
}

/// The Insert submenu: the hick vocabulary, one nested menu per group.
///
/// Every item forwards `insert:<element>` to the page, which opens the Insert
/// panel already on that element — the attributes still get filled in there,
/// because a menu item cannot ask for a container name. The first item opens
/// the panel with nothing chosen, which is what the accelerator is for.
fn insert_menu(handle: &AppHandle) -> tauri::Result<tauri::menu::Submenu<Wry>> {
    let mut insert = SubmenuBuilder::new(handle, "Insert").item(
        &MenuItemBuilder::with_id("insert", "Insert Element…")
            .accelerator("CmdOrCtrl+I")
            .build(handle)?,
    );
    insert = insert.separator();
    for (group, elements) in INSERT_GROUPS {
        let mut sub = SubmenuBuilder::new(handle, *group);
        for (id, label) in *elements {
            sub =
                sub.item(&MenuItemBuilder::with_id(format!("insert:{id}"), *label).build(handle)?);
        }
        insert = insert.item(&sub.build()?);
    }
    insert.build()
}

/// Route a chosen menu item.
///
/// Everything the page can act on is forwarded to it; only Open is handled
/// here, because switching folders is a process-level act (the directory lock
/// and the file watcher are per-process, so a new session is a restart).
fn on_menu(app: &AppHandle, id: &str) {
    match id {
        "new" | "save" | "save-as" | "save-all" | "print" | "settings" | "files" | "terminal"
        | "attention" => dispatch_to_ui(app, id),
        // Zoom, both scopes. Handled by the page rather than by the webview's
        // own zoom: this app sizes in `rem`, so moving the root font size
        // RE-LAYS-OUT at the new size, where a webview zoom scales rendered
        // pixels — blurry text, and hit targets that no longer line up with
        // what is drawn.
        _ if id.starts_with("zoom-") => dispatch_to_ui(app, id),
        "blame" => dispatch_to_ui(app, id),
        // Every item of the Insert submenu, which the page answers by opening
        // its panel on the named element.
        _ if id.starts_with("insert") => dispatch_to_ui(app, id),
        // No "file or folder?" question dialog: each verb goes straight to
        // its native picker. Cross-platform pickers cannot offer both in one
        // dialog, and a modal asking which picker you meant is worse than a
        // second menu item. Pickers block, and blocking dialogs deadlock the
        // main thread (see `run`) — so both run on a worker.
        "open-folder" => {
            let handle = app.clone();
            std::thread::spawn(move || {
                let picked = handle
                    .dialog()
                    .file()
                    .set_title("Open a folder of .hick documents")
                    .blocking_pick_folder()
                    .and_then(|p| p.into_path().ok());
                if let Some(dir) = picked {
                    switch_to(&handle, &dir);
                }
            });
        }
        "open-file" => {
            let handle = app.clone();
            std::thread::spawn(move || {
                let picked = handle
                    .dialog()
                    .file()
                    .set_title("Open a document")
                    .add_filter("Hickory documents", &["hick"])
                    .add_filter("All files", &["*"])
                    .blocking_pick_file()
                    .and_then(|p| p.into_path().ok());
                let Some(file) = picked else { return };
                // A file INSIDE the current session's folder opens in place:
                // the workspace is multi-document, so this is one more tab,
                // not a new session. Only a file elsewhere switches folders.
                let inside = handle
                    .try_state::<OpenedDir>()
                    .map(|d| file.starts_with(&d.0))
                    .unwrap_or(false);
                if inside {
                    open_path_in_ui(&handle, &file);
                } else {
                    let target = file
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| file.clone());
                    switch_to(&handle, &target);
                }
            });
        }
        // Predefined items (Quit, Edit, Window) are handled by the OS.
        _ => {}
    }
}

/// The folder this session serves, for "is that file already open here".
struct OpenedDir(PathBuf);

/// Switch the session to another folder: remember it and restart — the
/// directory lock and the watcher are per-process, so a new session IS a
/// restart.
fn switch_to(handle: &AppHandle, dir: &Path) {
    if let Ok(config_dir) = handle.path().app_config_dir() {
        server::remember(&config_dir, dir);
    }
    handle.restart();
}

/// Ask the page to open one file of the current session, by absolute path.
/// JSON-encoded into the eval string: a path may hold quotes.
fn open_path_in_ui(handle: &AppHandle, path: &Path) {
    if let Some(window) = handle.get_webview_window("main") {
        let payload = serde_json::to_string(&path.to_string_lossy()).unwrap_or_default();
        let _ = window.eval(format!(
            "window.dispatchEvent(new CustomEvent('hickory-open-path',{{detail:{payload}}}))"
        ));
    }
}

/// Forward a menu choice into the page as a DOM `CustomEvent`.
///
/// `eval` rather than Tauri's event IPC on purpose: the webview loads an
/// `http://127.0.0.1` URL (see `server.rs`), which the IPC capability system
/// treats as a remote origin. A dispatched DOM event needs no capability, no
/// `@tauri-apps/api` dependency, and works identically in a plain browser —
/// the frontend stays ignorant of being inside an app, which is the deal the
/// one-origin design already made.
fn dispatch_to_ui(app: &AppHandle, action: &str) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.eval(format!(
            "window.dispatchEvent(new CustomEvent('hickory-menu',{{detail:'{action}'}}))"
        ));
    }
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
        match runtime.block_on(server::start(&dir, config_dir.as_deref())) {
            Ok(session) => open(&handle, runtime, session, config_dir.as_deref(), &dir),
            Err(e) => fail(
                &handle,
                &format!("Could not open {}\n\n{e:#}", dir.display()),
            ),
        }
        return;
    }

    // Otherwise: the folder from last time, else the default workspace —
    // never a dialog. Launching the app is opening it; a picker standing
    // between the icon and a typeable page makes starting feel like a
    // commitment, and the default workspace is an ordinary folder of .hick
    // files (`hick up` runs on it, nothing about it is special), so there is
    // no decision worth interrupting for. The picker survives only as the
    // fallback for a machine where the workspace cannot be created.
    let mut candidate = config_dir
        .as_deref()
        .and_then(server::last_opened)
        .or_else(|| default_workspace(&handle));

    loop {
        let dir = match candidate.take() {
            Some(dir) => dir,
            None => match pick_folder(&handle) {
                Some(dir) => dir,
                // Cancelling the picker with nothing open is a decision not to
                // use the app right now, not an error to report.
                None => {
                    handle.exit(0);
                    return;
                }
            },
        };

        match runtime.block_on(server::start(&dir, config_dir.as_deref())) {
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

/// The zero-ceremony workspace: `Documents/HickoryDocs` (home as fallback),
/// created on demand.
///
/// Under Documents rather than an app-data directory because these are the
/// user's files, not the app's: visible in a file manager, syncable,
/// greppable, and exactly what `hick up` or a git repo would be pointed at
/// later. Nothing about the folder is special — it is simply the answer to
/// "where", so the app never has to ask.
fn default_workspace(handle: &AppHandle) -> Option<PathBuf> {
    let base = handle
        .path()
        .document_dir()
        .ok()
        .or_else(|| handle.path().home_dir().ok())?;
    let dir = base.join("HickoryDocs");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

/// The launch fallback's folder picker — reached only when the default
/// workspace cannot be created. No "file or folder?" question: the fallback
/// needs a working directory, and a single file's directory is one.
fn pick_folder(handle: &AppHandle) -> Option<PathBuf> {
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
    handle.manage(OpenedDir(dir.to_path_buf()));
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
                .title(native_title(config_dir, dir))
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

/// The native window's title, read ONCE at launch: the custom title from the
/// Settings page (`ui.json`, the file behind `PUT /api/settings/ui`) when
/// one is set, else "Hickory Docs — <folder>".
///
/// Deliberately launch-time only: the page has no IPC back into this shell
/// (the one-origin design keeps the frontend ignorant of being in an app),
/// so a title saved in Settings names the native window at the NEXT launch.
/// The live half is the page's own `document.title` (lib/windowTitle.ts).
fn native_title(config_dir: Option<&Path>, dir: &Path) -> String {
    config_dir
        .map(server::ui_settings_file)
        .and_then(|path| hickory_cli::serve::UiStore::load(&path).ok())
        .and_then(|ui| ui.window_title)
        .unwrap_or_else(|| format!("Hickory Docs — {}", folder_name(dir)))
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
