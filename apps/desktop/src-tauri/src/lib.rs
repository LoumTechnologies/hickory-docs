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

pub mod dev;
mod file_open;
mod recent;
pub mod server;

use std::path::{Path, PathBuf};

use anyhow::Context as _;
use tauri::menu::{Menu, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Manager as _, WebviewUrl, WebviewWindowBuilder, WindowEvent, Wry};
use tauri_plugin_dialog::{DialogExt as _, MessageDialogKind};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .on_menu_event(|app, event| on_menu(app, event.id().as_ref()))
        .on_window_event(|window, event| {
            if let WindowEvent::Focused(true) = event {
                recent::refresh(window.app_handle());
            }
            if let WindowEvent::CloseRequested { api, .. } = event {
                let gate = window.state::<CloseGate>();
                if !gate.0.swap(false, std::sync::atomic::Ordering::SeqCst) {
                    api.prevent_close();
                    if let Some(webview) = window.app_handle().get_webview_window(window.label()) {
                        let _ = webview.eval(
                            "window.dispatchEvent(new Event('hickory-window-close-request'))",
                        );
                    }
                }
            }
        })
        .setup(|app| {
            app.manage(CloseGate(std::sync::atomic::AtomicBool::new(false)));
            let handle = app.handle().clone();
            // The menu is built here rather than through `.menu(...)`: it
            // reads the person's accelerators from ui.json, which needs the
            // app's config directory, and the path resolver exists only once
            // setup runs. Built through `.menu` it panicked before the first
            // window ("state() called before manage()").
            app.set_menu(app_menu(&handle)?)?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Hickory Docs")
        .run(file_open::handler());
}

struct CloseGate(std::sync::atomic::AtomicBool);

/// The native menu bar.
///
/// Two kinds of item: the ones the webview can answer (New, Save, Save As,
/// Settings — forwarded as a DOM event, see [`dispatch_to_ui`]) and the ones
/// only this process can (Open, which restarts the session on another folder,
/// and Quit). Edit and Window are the platform's own items, predefined so
/// cut/copy/paste behave exactly as the OS says they should.
/// The menu bar's accelerators, as the person configured them in Settings →
/// Keyboard: the page resolves its keymap to the shell's spelling and writes
/// `native_accelerators` into ui.json, and this reads them at launch. An item
/// the file does not name keeps its built-in key. Launch-time only, like the
/// window title: the page has no IPC back into this shell, so a change lands
/// at the next launch, and the Settings page says so.
struct Accelerators(std::collections::BTreeMap<String, String>);

impl Accelerators {
    fn load(handle: &AppHandle) -> Self {
        let map = handle
            .path()
            .app_config_dir()
            .ok()
            .map(|dir| server::ui_settings_file(&dir))
            .and_then(|path| hickory_cli::serve::UiStore::load(&path).ok())
            .map(|ui| ui.native_accelerators)
            .unwrap_or_default();
        Self(map)
    }

    /// The accelerator for a menu id, or the built-in one. `""` in the file
    /// means "no key", which an item may legitimately have in a profile.
    fn get<'a>(&'a self, id: &str, built_in: &'a str) -> &'a str {
        self.0.get(id).map(String::as_str).unwrap_or(built_in)
    }
}

/// A menu item with its accelerator, when it has one: Tauri refuses an empty
/// accelerator string, so "no key" means not calling `.accelerator` at all.
fn item(
    handle: &AppHandle,
    keys: &Accelerators,
    id: &str,
    label: &str,
    built_in: &str,
) -> tauri::Result<tauri::menu::MenuItem<Wry>> {
    let mut builder = MenuItemBuilder::with_id(id, label);
    let key = keys.get(id, built_in);
    if !key.is_empty() {
        builder = builder.accelerator(key);
    }
    builder.build(handle)
}

fn app_menu(handle: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let keys = Accelerators::load(handle);
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

    let recent = recent::Menus::new(handle)?;
    let file = SubmenuBuilder::new(handle, "File")
        .item(&item(
            handle,
            &keys,
            "new-window",
            "New Window",
            "CmdOrCtrl+Shift+N",
        )?)
        .separator()
        .item(&item(handle, &keys, "new", "New Document", "CmdOrCtrl+N")?)
        .item(
            // Not a mode of New Document: this one writes a real file and
            // runs a generator, and the page answers it with its own dialog.
            &item(
                handle,
                &keys,
                "new-project",
                "New Project…",
                "CmdOrCtrl+Alt+Shift+N",
            )?,
        )
        .item(&item(
            handle,
            &keys,
            "open-file",
            "Open File…",
            "CmdOrCtrl+O",
        )?)
        .item(&item(
            handle,
            &keys,
            "open-folder",
            "Open Folder…",
            "CmdOrCtrl+Shift+O",
        )?)
        .item(&recent.files)
        .item(&recent.folders)
        .separator()
        .item(&item(handle, &keys, "save", "Save", "CmdOrCtrl+S")?)
        .item(&item(
            handle,
            &keys,
            "save-as",
            "Save As…",
            "CmdOrCtrl+Shift+S",
        )?)
        .item(
            // Every open buffer at once. The app already saves as you type,
            // so this is "flush everything now" rather than "or else it is
            // lost" — which is why it has no urgent accelerator.
            &item(handle, &keys, "save-all", "Save All", "CmdOrCtrl+Alt+S")?,
        )
        .separator()
        .item(&item(handle, &keys, "print", "Print…", "CmdOrCtrl+P")?)
        .separator()
        .item(&item(
            handle,
            &keys,
            "terminal",
            "New Terminal",
            "CmdOrCtrl+Shift+T",
        )?)
        .item(
            // The one key that answers "what needs me?" — it walks the
            // attention queue in the order the server ranks it, and says so
            // when nothing is left.
            &item(
                handle,
                &keys,
                "attention",
                "Next Needing Attention",
                "CmdOrCtrl+J",
            )?,
        )
        .separator()
        .item(&item(
            handle,
            &keys,
            "files",
            "Show Files",
            "CmdOrCtrl+Shift+E",
        )?)
        .item(&item(
            handle,
            &keys,
            "show-agent",
            "Agent",
            "CmdOrCtrl+Shift+I",
        )?)
        .item(&item(
            handle,
            &keys,
            "settings",
            "Settings…",
            "CmdOrCtrl+,",
        )?)
        .separator()
        .quit()
        .build()?;

    handle.manage(recent);

    let edit = SubmenuBuilder::new(handle, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .separator()
        .item(&item(
            handle,
            &keys,
            "edit-element",
            "Edit Element…",
            "CmdOrCtrl+Alt+I",
        )?)
        .build()?;

    // Zoom, at two scopes. The whole window takes the chord everybody
    // already has in their fingers from browsers and editors; adding Alt
    // narrows it to the focused tab, which is the rarer request.
    let view = SubmenuBuilder::new(handle, "View")
        .item(&item(handle, &keys, "zoom-in", "Zoom In", "CmdOrCtrl+=")?)
        .item(&item(handle, &keys, "zoom-out", "Zoom Out", "CmdOrCtrl+-")?)
        .item(&item(
            handle,
            &keys,
            "zoom-reset",
            "Actual Size",
            "CmdOrCtrl+0",
        )?)
        .separator()
        .item(&item(
            handle,
            &keys,
            "zoom-tab-in",
            "Zoom In This Tab",
            "CmdOrCtrl+Alt+=",
        )?)
        .item(&item(
            handle,
            &keys,
            "zoom-tab-out",
            "Zoom Out This Tab",
            "CmdOrCtrl+Alt+-",
        )?)
        .item(&item(
            handle,
            &keys,
            "zoom-tab-reset",
            "Actual Size In This Tab",
            "CmdOrCtrl+Alt+0",
        )?)
        .separator()
        .item(
            // Off by default: a blame column is a permanent indent on every
            // line of every file, answering a question nobody asks most of
            // the time. It earns its place when you are asking it.
            &item(
                handle,
                &keys,
                "blame",
                "Show Blame Column",
                "CmdOrCtrl+Alt+B",
            )?,
        )
        .build()?;

    let insert = insert_menu(handle, &keys)?;

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
fn insert_menu(
    handle: &AppHandle,
    keys: &Accelerators,
) -> tauri::Result<tauri::menu::Submenu<Wry>> {
    let mut insert = SubmenuBuilder::new(handle, "Insert").item(&item(
        handle,
        keys,
        "insert",
        "Insert Element…",
        "CmdOrCtrl+I",
    )?);
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
        "new-window" => {
            if let Err(e) = open_blank_window() {
                fail(app, &format!("Could not open a new window.\n\n{e:#}"));
            }
        }
        "new" | "new-project" | "save" | "save-as" | "save-all" | "print" | "settings"
        | "files" | "show-agent" | "terminal" | "attention" | "edit-element" => {
            dispatch_to_ui(app, id)
        }
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
        _ if id.starts_with("recent-file:") || id.starts_with("recent-folder:") => {
            let (kind, path) = id.split_once(':').expect("recent path prefix");
            let file = kind == "recent-file";
            let path = PathBuf::from(path);
            let handle = app.clone();
            std::thread::spawn(move || {
                if (file && !path.is_file()) || (!file && !path.is_dir()) {
                    recent::refresh(&handle);
                    handle.dialog().message(format!(
                        "Could not open {}. It has moved or is unavailable.\n\nUse File → Open File or Open Folder to choose its current location.",
                        path.display()
                    )).kind(MessageDialogKind::Warning).title("Could not open recent path").blocking_show();
                } else if file {
                    open_file(&handle, &path);
                } else {
                    switch_to(&handle, &path);
                }
            });
        }
        "open-folder" => {
            let handle = app.clone();
            std::thread::spawn(move || {
                let picked = handle
                    .dialog()
                    .file()
                    .set_title("Open a folder of documents")
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
                    .add_filter("Markdown documents", &["md"])
                    .add_filter("All files", &["*"])
                    .blocking_pick_file()
                    .and_then(|p| p.into_path().ok());
                let Some(file) = picked else { return };
                open_file(&handle, &file);
            });
        }
        // Predefined items (Quit, Edit, Window) are handled by the OS.
        _ => {}
    }
}

fn open_file(handle: &AppHandle, file: &Path) {
    let file = file.canonicalize().unwrap_or_else(|_| file.to_path_buf());
    let inside = handle
        .try_state::<OpenedDir>()
        .map(|d| file.starts_with(&d.0))
        .unwrap_or(false);
    if inside {
        open_path_in_ui(handle, &file);
        recent::record(handle, &file);
    } else {
        switch_to(handle, &file);
    }
}

/// The folder this session serves, for "is that file already open here".
struct OpenedDir(PathBuf);

/// Switch by launching the selected target explicitly, then exiting this
/// window. This leaves any original path or `--blank-window` flag behind.
fn switch_to(handle: &AppHandle, dir: &Path) {
    if let Ok(config_dir) = handle.path().app_config_dir() {
        server::remember(&config_dir, dir);
    }
    // Relaunch with an explicit target. Restart would preserve the original
    // command-line path and reopen it instead of the newly selected folder.
    match open_folder_in_new_process(dir) {
        Ok(()) => handle.exit(0),
        Err(e) => fail(
            handle,
            &format!("Could not open {}\n\n{e:#}", dir.display()),
        ),
    }
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

/// Open an explicitly requested path, or an editor with no folder selected.
fn launch(handle: AppHandle, requested_file: Option<PathBuf>) {
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => return fail(&handle, &format!("Could not start the runtime.\n\n{e}")),
    };

    let config_dir = handle.path().app_config_dir().ok();

    if std::env::args().skip(1).any(|arg| arg == "--blank-window") {
        match runtime.block_on(server::start_blank_with_config(
            config_dir.as_deref(),
            Some(server::shell_hooks(&handle, config_dir.as_deref())),
        )) {
            Ok(session) => open_blank(&handle, runtime, session, config_dir.as_deref()),
            Err(e) => fail(&handle, &format!("Could not open a blank window.\n\n{e:#}")),
        }
        return;
    }

    // A folder named on the command line or in the environment is a deliberate
    // act, and is tried once: someone who typed a path wants that path, and
    // falling back to a picker would quietly hide their typo.
    if let Some(dir) = requested_file.or_else(server::named_dir) {
        match runtime.block_on(server::start_with_menu(
            &dir,
            config_dir.as_deref(),
            Some(server::shell_hooks(&handle, config_dir.as_deref())),
            Some(handle.clone()),
        )) {
            Ok(session) => open(&handle, runtime, session, config_dir.as_deref(), &dir),
            Err(e) => fail(
                &handle,
                &format!("Could not open {}\n\n{e:#}", dir.display()),
            ),
        }
        return;
    }

    // An ordinary launch starts an editor, not a remembered repository.
    // Recent paths remain available through File → Recent; only an explicit
    // path above opens a workspace and enables repository diagnostics.
    match runtime.block_on(server::start_blank_with_config(
        config_dir.as_deref(),
        Some(server::shell_hooks(&handle, config_dir.as_deref())),
    )) {
        Ok(session) => open_blank(&handle, runtime, session, config_dir.as_deref()),
        Err(e) => fail(&handle, &format!("Could not open the editor.\n\n{e:#}")),
    }
}

/// Launch another copy of this app. A session is a process here, so this is
/// also how File → New Project opens its completed project elsewhere.
fn open_blank_window() -> anyhow::Result<()> {
    let app = find_own_app()?;
    hickory_cli::open_app::open_blank(&app)
}

fn open_folder_in_new_process(folder: &Path) -> anyhow::Result<()> {
    hickory_cli::open_app::open(&find_own_app()?, folder)
}

fn find_own_app() -> anyhow::Result<hickory_cli::open_app::App> {
    hickory_cli::open_app::find(|name| std::env::var(name).ok(), |path| path.exists())
        .context("could not find this app's own executable to start a second copy of it")
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
    handle.manage(OpenedDir(
        dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf()),
    ));
    recent::record(handle, dir);
    if let Some(config_dir) = config_dir {
        server::remember(config_dir, dir);
    }

    // The UI's address, not the engine's: under `just dev` they differ, and
    // the window wants the one with hot reload on it.
    let mut url = session.ui_url.clone();
    if dir.is_file() {
        let path = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
        let mut address = tauri::Url::parse(&url).expect("local UI URL");
        let encoded: String = path
            .to_string_lossy()
            .as_bytes()
            .iter()
            .map(|byte| format!("%{byte:02X}"))
            .collect();
        address.set_fragment(Some(&format!("/file/{encoded}")));
        url = address.to_string();
    }
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
                // Tauri's own drag-drop handler swallows file drops before
                // the page sees them: the cursor tracks the drag, the drop
                // lands nowhere, and nothing here listens for the Tauri
                // event in its place. The editor handles an image dropped
                // onto it as an ordinary HTML5 drop (editor/mdPaste.ts), so
                // the page gets the events.
                .disable_drag_drop_handler()
                .build()
                .map_err(|e| e.to_string())
        });

    if let Err(e) = built {
        fail(handle, &format!("Could not open the window.\n\n{e}"));
    }
}

/// Open a window with editor sessions and no selected folder.
fn open_blank(
    handle: &AppHandle,
    runtime: tokio::runtime::Runtime,
    session: server::Session,
    config_dir: Option<&Path>,
) {
    let url = session.ui_url.clone();
    handle.manage(session);
    handle.manage(runtime);
    let built = url
        .parse::<tauri::Url>()
        .map_err(|e| e.to_string())
        .and_then(|url| {
            WebviewWindowBuilder::new(handle, "main", WebviewUrl::External(url))
                .title(blank_native_title(config_dir))
                .inner_size(1100.0, 780.0)
                .min_inner_size(360.0, 480.0)
                .disable_drag_drop_handler()
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

fn blank_native_title(config_dir: Option<&Path>) -> String {
    config_dir
        .map(server::ui_settings_file)
        .and_then(|path| hickory_cli::serve::UiStore::load(&path).ok())
        .and_then(|ui| ui.window_title)
        .unwrap_or_else(|| "Hickory Docs".to_string())
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
