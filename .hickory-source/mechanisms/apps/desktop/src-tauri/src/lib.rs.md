<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/desktop/src-tauri/src/lib.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-42caba993099"></a>
## `apps/desktop/src-tauri/src/lib.rs`

Wrap the shared engine and web interface in the native desktop host; keep host lifecycle behavior at this boundary.

Related contracts: [A Provider Key Entered in Settings Stays on This Machine](../../../../../guarantees/agent/keys-are-stored-locally-and-never-leave/contract.md), [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [An Equation Renders And Shows Its Source](../../../../../guarantees/authoring/an-equation-renders-and-shows-its-source/contract.md), [An Image Dropped In Becomes A File And A Reference](../../../../../guarantees/authoring/an-image-dropped-in-becomes-a-file-and-a-reference/contract.md), [Inserting An Element Writes Hick You Could Have Typed Yourself](../../../../../guarantees/authoring/inserting-an-element-writes-hick-you-could-have-typed/contract.md), [The App Starts As A Lightweight Editor](../../../../../guarantees/authoring/the-app-starts-as-a-lightweight-editor/contract.md), [Every Shortcut Is A Setting](../../../../../guarantees/editor-intelligence/every-shortcut-is-a-setting/contract.md), [The Blame Column Is Optional And Honest](../../../../../guarantees/lineage/the-blame-column-is-optional-and-honest/contract.md), [A Local Install Is a Launchable macOS App](../../../../../guarantees/release/a-local-install-is-a-launchable-macos-app/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md).

<a id="f-42caba993099-command_path"></a>
### `command_path`

The Hickory Docs desktop shell.
A native window around the local document server — the same engine `hick
up` is built from, running in this process on loopback. The app and the CLI
are two front doors onto one implementation, which is the point of
`docs/specs/freeform/local-only.md`: there is no second code path for
weaving, lineage, or carrying an output edit back into a document.
Packaged Unstable Apple Silicon builds check GitHub for signed updates.
There is no account or telemetry.
## Why the launch sequence looks like this
The window's URL is the address the server bound to, so the engine has to
start before there is a window to show. But choosing a folder needs a modal
dialog, which needs an event loop already running. The resolution is to
start Tauri with no window at all and do the whole sequence — pick, start,
open — on a worker thread, which is also where the blocking dialogs belong:
calling them on the main thread deadlocks against the event loop they are
waiting on.

<storycode:copy id="f-42caba993099-command_path">
//! The Hickory Docs desktop shell.
//!
//! A native window around the local document server — the same engine `hick
//! up` is built from, running in this process on loopback. The app and the CLI
//! are two front doors onto one implementation, which is the point of
//! `docs/specs/freeform/local-only.md`: there is no second code path for
//! weaving, lineage, or carrying an output edit back into a document.
//!
//! Packaged Unstable Apple Silicon builds check GitHub for signed updates.
//! There is no account or telemetry.
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

mod command_path;
</storycode:copy>
<a id="f-42caba993099-dev"></a>
### `dev`

<storycode:copy id="f-42caba993099-dev">
pub mod dev;
</storycode:copy>
<a id="f-42caba993099-exit_gate"></a>
### `exit_gate`

<storycode:copy id="f-42caba993099-exit_gate">
mod exit_gate;
</storycode:copy>
<a id="f-42caba993099-file_open"></a>
### `file_open`

<storycode:copy id="f-42caba993099-file_open">
mod file_open;
</storycode:copy>
<a id="f-42caba993099-folder_session"></a>
### `folder_session`

<storycode:copy id="f-42caba993099-folder_session">
mod folder_session;
</storycode:copy>
<a id="f-42caba993099-recent"></a>
### `recent`

<storycode:copy id="f-42caba993099-recent">
mod recent;
</storycode:copy>
<a id="f-42caba993099-updater"></a>
### `updater`

<storycode:copy id="f-42caba993099-updater">
mod updater;
</storycode:copy>
<a id="f-42caba993099-server"></a>
### `server`

<storycode:copy id="f-42caba993099-server">
pub mod server;

</storycode:copy>
<a id="f-42caba993099-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-42caba993099-use_declaration">
use std::path::{Path, PathBuf};

</storycode:copy>
<a id="f-42caba993099-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-42caba993099-use_declaration-2">
use anyhow::Context as _;
</storycode:copy>
<a id="f-42caba993099-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-42caba993099-use_declaration-3">
use tauri::menu::{Menu, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
</storycode:copy>
<a id="f-42caba993099-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-42caba993099-use_declaration-4">
use tauri::{AppHandle, Manager as _, WebviewUrl, WebviewWindowBuilder, WindowEvent, Wry};
</storycode:copy>
<a id="f-42caba993099-use_declaration-5"></a>
### `use_declaration`

<storycode:copy id="f-42caba993099-use_declaration-5">
use tauri_plugin_dialog::{DialogExt as _, MessageDialogKind};

</storycode:copy>
<a id="f-42caba993099-QuitIntent"></a>
### `QuitIntent`

Records a macOS application Quit request while the page resolves its
save/discard decision. A regular window close must not terminate the app.

<storycode:copy id="f-42caba993099-QuitIntent">
/// Records a macOS application Quit request while the page resolves its
/// save/discard decision. A regular window close must not terminate the app.
pub(crate) struct QuitIntent(pub std::sync::atomic::AtomicBool);

</storycode:copy>
<a id="f-42caba993099-run"></a>
### `run`

<storycode:copy id="f-42caba993099-run">
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .on_menu_event(|app, event| on_menu(app, event.id().as_ref()))
        .on_window_event(|window, event| {
            if let WindowEvent::Focused(true) = event {
                recent::refresh(window.app_handle());
            }
            if let WindowEvent::Destroyed = event {
                let folder = window.state::<WindowFolder>();
                if let Some(folder) = folder.0.lock().unwrap().as_deref()
                    && let Ok(config) = window.app_handle().path().app_config_dir()
                {
                    server::remember(&config, folder);
                }
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
            app.manage(WindowFolder(std::sync::Mutex::new(None)));
            app.manage(CloseGate(std::sync::atomic::AtomicBool::new(false)));
            app.manage(QuitIntent(std::sync::atomic::AtomicBool::new(false)));
            let handle = app.handle().clone();
            // The menu is built here rather than through `.menu(...)`: it
            // reads the person's accelerators from ui.json, which needs the
            // app's config directory, and the path resolver exists only once
            // setup runs. Built through `.menu` it panicked before the first
            // window ("state() called before manage()").
            app.set_menu(app_menu(&handle)?)?;
            updater::start(handle);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Hickory Docs")
        .run(file_open::handler());
}

</storycode:copy>
<a id="f-42caba993099-CloseGate"></a>
### `CloseGate`

<storycode:copy id="f-42caba993099-CloseGate">
struct CloseGate(std::sync::atomic::AtomicBool);

</storycode:copy>
<a id="f-42caba993099-WindowFolder"></a>
### `WindowFolder`

Each folder window has its own process. Record it at close as well as open
so the last window closed determines what an ordinary launch restores.

<storycode:copy id="f-42caba993099-WindowFolder">
// Each folder window has its own process. Record it at close as well as open
// so the last window closed determines what an ordinary launch restores.
struct WindowFolder(std::sync::Mutex<Option<PathBuf>>);

</storycode:copy>
<a id="f-42caba993099-Accelerators"></a>
### `Accelerators`

The native menu bar.
Two kinds of item: the ones the webview can answer (New, Save, Save As,
Settings — forwarded as a DOM event, see [`dispatch_to_ui`]) and the ones
only this process can (Open, which restarts the session on another folder,
and Quit). Edit and Window are the platform's own items, predefined so
cut/copy/paste behave exactly as the OS says they should.
The menu bar's accelerators, as the person configured them in Settings →
Keyboard: the page resolves its keymap to the shell's spelling and writes
`native_accelerators` into ui.json, and this reads them at launch. An item
the file does not name keeps its built-in key. Launch-time only, like the
window title: the page has no IPC back into this shell, so a change lands
at the next launch, and the Settings page says so.

<storycode:copy id="f-42caba993099-Accelerators">
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

</storycode:copy>
<a id="f-42caba993099-impl-Accelerators"></a>
### `impl-Accelerators`

<storycode:copy id="f-42caba993099-impl-Accelerators">
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

</storycode:copy>
<a id="f-42caba993099-item"></a>
### `item`

A menu item with its accelerator, when it has one: Tauri refuses an empty
accelerator string, so "no key" means not calling `.accelerator` at all.

<storycode:copy id="f-42caba993099-item">
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

</storycode:copy>
<a id="f-42caba993099-native_window_operation"></a>
### `native_window_operation`

<storycode:copy id="f-42caba993099-native_window_operation">
fn native_window_operation(
    handle: &AppHandle,
    operation: hickory_cli::serve::WindowOperation,
) -> anyhow::Result<hickory_cli::serve::WindowStatus> {
    use hickory_cli::serve::{WindowOperation, WindowStatus};
    let window = handle
        .get_webview_window("main")
        .ok_or_else(|| anyhow::anyhow!("Current native window closed"))?;
    match operation {
        WindowOperation::Inspect => {}
        WindowOperation::Minimize => window.minimize()?,
        WindowOperation::Restore => {
            window.show()?;
            window.unminimize()?;
        }
    }
    Ok(WindowStatus {
        native: true,
        platform: std::env::consts::OS.to_owned(),
        minimized: window.is_minimized()?,
    })
}

</storycode:copy>
<a id="f-42caba993099-app_menu"></a>
### `app_menu`

<storycode:copy id="f-42caba993099-app_menu">
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
        .item(&item(
            handle,
            &keys,
            "minimize-window",
            "Minimize",
            if cfg!(target_os = "macos") {
                "Cmd+M"
            } else {
                ""
            },
        )?)
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

</storycode:copy>
<a id="f-42caba993099-InsertGroups"></a>
### `InsertGroups`

<storycode:copy id="f-42caba993099-InsertGroups">
type InsertGroups = &'static [(&'static str, &'static [(&'static str, &'static str)])];

</storycode:copy>
<a id="f-42caba993099-INSERT_GROUPS"></a>
### `INSERT_GROUPS`

Every element of the Insert menu, as `(menu id, label)` grouped exactly as
the page groups them.
This is the native half of `apps/web/src/lib/insertCatalog.ts`, which owns
the attributes, the hints, and the bytes each element writes — the menu
only needs to name one. The two lists are kept in step by
`tests/insert_menu_matches_catalogue.rs`; a drift there is a real bug,
because a menu offering an element the page has never heard of would open
the panel on the wrong thing.

<storycode:copy id="f-42caba993099-INSERT_GROUPS">
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

</storycode:copy>
<a id="f-42caba993099-insert_menu_elements"></a>
### `insert_menu_elements`

The menu's element names, for the parity test against the page's
catalogue (`tests/insert_menu_matches_catalogue.rs`).

<storycode:copy id="f-42caba993099-insert_menu_elements">
/// The menu's element names, for the parity test against the page's
/// catalogue (`tests/insert_menu_matches_catalogue.rs`).
pub fn insert_menu_elements() -> InsertGroups {
    INSERT_GROUPS
}

</storycode:copy>
<a id="f-42caba993099-insert_menu"></a>
### `insert_menu`

The Insert submenu: the hick vocabulary, one nested menu per group.
Every item forwards `insert:<element>` to the page, which opens the Insert
panel already on that element — the attributes still get filled in there,
because a menu item cannot ask for a container name. The first item opens
the panel with nothing chosen, which is what the accelerator is for.

<storycode:copy id="f-42caba993099-insert_menu">
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

</storycode:copy>
<a id="f-42caba993099-on_menu"></a>
### `on_menu`

Route a chosen menu item.
Everything the page can act on is forwarded to it; only Open is handled
here: the first folder attaches to this window, and another folder
starts a separate session.

<storycode:copy id="f-42caba993099-on_menu">
/// Route a chosen menu item.
///
/// Everything the page can act on is forwarded to it; only Open is handled
/// here: the first folder attaches to this window, and another folder
/// starts a separate session.
fn on_menu(app: &AppHandle, id: &str) {
    match id {
        "minimize-window" => {
            if let Err(error) =
                native_window_operation(app, hickory_cli::serve::WindowOperation::Minimize)
            {
                app.dialog()
                    .message(error.to_string())
                    .title("Could not minimize window")
                    .show(|_| {});
            }
        }
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
                    open_folder_window(&handle, &path);
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
                    open_folder_window(&handle, &dir);
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

</storycode:copy>
<a id="f-42caba993099-open_file"></a>
### `open_file`

<storycode:copy id="f-42caba993099-open_file">
fn open_file(handle: &AppHandle, file: &Path) {
    let file = file.canonicalize().unwrap_or_else(|_| file.to_path_buf());
    open_path_in_ui(handle, &file);
    recent::record(handle, &file);
}

</storycode:copy>
<a id="f-42caba993099-open_folder_window"></a>
### `open_folder_window`

Attach the first folder to this window; later folders get another window.

<storycode:copy id="f-42caba993099-open_folder_window">
/// Attach the first folder to this window; later folders get another window.
fn open_folder_window(handle: &AppHandle, dir: &Path) {
    let config = handle.path().app_config_dir().ok();
    let result = (|| -> anyhow::Result<()> {
        let attached = if let (Some(session), Some(runtime)) = (
            handle.try_state::<server::Session>(),
            handle.try_state::<tokio::runtime::Runtime>(),
        ) {
            runtime.block_on(session.attach_folder(
                dir,
                config.as_deref(),
                Some(server::shell_hooks(handle, config.as_deref())),
                Some(handle.clone()),
            ))?
        } else {
            false
        };
        if attached {
            *handle.state::<WindowFolder>().0.lock().unwrap() = Some(dir.to_path_buf());
            if let Some(window) = handle.get_webview_window("main") {
                let _ = window.set_title(&native_title(config.as_deref(), dir));
                let _ = window.eval("window.dispatchEvent(new Event('hickory:files-changed'))");
            }
            if let Some(config) = &config {
                server::remember(config, dir);
            }
            recent::record(handle, dir);
            return Ok(());
        }
        open_folder_in_new_process(dir)
    })();
    if let Err(error) = result {
        handle
            .dialog()
            .message(format!("Could not open {}\n\n{error:#}", dir.display()))
            .kind(MessageDialogKind::Error)
            .title("Could not open folder")
            .blocking_show();
    }
}

</storycode:copy>
<a id="f-42caba993099-open_path_in_ui"></a>
### `open_path_in_ui`

Ask the page to open one file of the current session, by absolute path.
JSON-encoded into the eval string: a path may hold quotes.

<storycode:copy id="f-42caba993099-open_path_in_ui">
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

</storycode:copy>
<a id="f-42caba993099-dispatch_to_ui"></a>
### `dispatch_to_ui`

Forward a menu choice into the page as a DOM `CustomEvent`.
`eval` rather than Tauri's event IPC on purpose: the webview loads an
`http://127.0.0.1` URL (see `server.rs`), which the IPC capability system
treats as a remote origin. A dispatched DOM event needs no capability, no
`@tauri-apps/api` dependency, and works identically in a plain browser —
the frontend stays ignorant of being inside an app, which is the deal the
one-origin design already made.

<storycode:copy id="f-42caba993099-dispatch_to_ui">
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

</storycode:copy>
<a id="f-42caba993099-launch"></a>
### `launch`

Open an explicitly requested path, the last folder, or a blank editor.

<storycode:copy id="f-42caba993099-launch">
/// Open an explicitly requested path, the last folder, or a blank editor.
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
    if let Some(dir) = requested_file.or_else(server::named_dir).or_else(|| {
        config_dir
            .as_deref()
            .and_then(server::last_opened)
            .filter(|path| path.is_dir())
    }) {
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

    // First launch, or a remembered folder that was moved or removed.
    match runtime.block_on(server::start_blank_with_config(
        config_dir.as_deref(),
        Some(server::shell_hooks(&handle, config_dir.as_deref())),
    )) {
        Ok(session) => open_blank(&handle, runtime, session, config_dir.as_deref()),
        Err(e) => fail(&handle, &format!("Could not open the editor.\n\n{e:#}")),
    }
}

</storycode:copy>
<a id="f-42caba993099-open_blank_window"></a>
### `open_blank_window`

Launch another copy of this app. A session is a process here, so this is
also how File → New Project opens its completed project elsewhere.

<storycode:copy id="f-42caba993099-open_blank_window">
/// Launch another copy of this app. A session is a process here, so this is
/// also how File → New Project opens its completed project elsewhere.
fn open_blank_window() -> anyhow::Result<()> {
    let app = find_own_app()?;
    hickory_cli::open_app::open_blank(&app)
}

</storycode:copy>
<a id="f-42caba993099-open_folder_in_new_process"></a>
### `open_folder_in_new_process`

<storycode:copy id="f-42caba993099-open_folder_in_new_process">
fn open_folder_in_new_process(folder: &Path) -> anyhow::Result<()> {
    hickory_cli::open_app::open(&find_own_app()?, folder)
}

</storycode:copy>
<a id="f-42caba993099-find_own_app"></a>
### `find_own_app`

<storycode:copy id="f-42caba993099-find_own_app">
fn find_own_app() -> anyhow::Result<hickory_cli::open_app::App> {
    hickory_cli::open_app::find(|name| std::env::var(name).ok(), |path| path.exists())
        .context("could not find this app's own executable to start a second copy of it")
}

</storycode:copy>
<a id="f-42caba993099-open"></a>
### `open`

Open the window on a running session, and remember the folder.
`runtime` and `session` are moved into the app's managed state: the runtime
owns the server task and the session holds the directory lock, so both have
to live exactly as long as the app does.

<storycode:copy id="f-42caba993099-open">
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
    *handle.state::<WindowFolder>().0.lock().unwrap() = dir.is_dir().then(|| dir.to_path_buf());
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

</storycode:copy>
<a id="f-42caba993099-open_blank"></a>
### `open_blank`

Open a window with editor sessions and no selected folder.

<storycode:copy id="f-42caba993099-open_blank">
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

</storycode:copy>
<a id="f-42caba993099-folder_name"></a>
### `folder_name`

The folder's own name, for the title bar. A full path makes a long and
mostly useless title; the name is what the user calls the project.

<storycode:copy id="f-42caba993099-folder_name">
/// The folder's own name, for the title bar. A full path makes a long and
/// mostly useless title; the name is what the user calls the project.
fn folder_name(dir: &Path) -> String {
    dir.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| dir.display().to_string())
}

</storycode:copy>
<a id="f-42caba993099-native_title"></a>
### `native_title`

The native window's title, read ONCE at launch: the custom title from the
Settings page (`ui.json`, the file behind `PUT /api/settings/ui`) when
one is set, else "Hickory Docs — <folder>".
Deliberately launch-time only: the page has no IPC back into this shell
(the one-origin design keeps the frontend ignorant of being in an app),
so a title saved in Settings names the native window at the NEXT launch.
The live half is the page's own `document.title` (lib/windowTitle.ts).

<storycode:copy id="f-42caba993099-native_title">
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

</storycode:copy>
<a id="f-42caba993099-blank_native_title"></a>
### `blank_native_title`

<storycode:copy id="f-42caba993099-blank_native_title">
fn blank_native_title(config_dir: Option<&Path>) -> String {
    config_dir
        .map(server::ui_settings_file)
        .and_then(|path| hickory_cli::serve::UiStore::load(&path).ok())
        .and_then(|ui| ui.window_title)
        .unwrap_or_else(|| "Hickory Docs".to_string())
}

</storycode:copy>
<a id="f-42caba993099-fail"></a>
### `fail`

Report a fatal startup failure, then exit.
A dialog rather than stderr: this process is normally launched by
double-clicking an icon, so an error that exists only in a terminal does
not exist. It is printed as well, for anyone who did start it from one.

<storycode:copy id="f-42caba993099-fail">
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
</storycode:copy>

### Assemble `apps/desktop/src-tauri/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/desktop/src-tauri/src/lib.rs"><storycode:paste select="#f-42caba993099-command_path"/><storycode:paste select="#f-42caba993099-dev"/><storycode:paste select="#f-42caba993099-exit_gate"/><storycode:paste select="#f-42caba993099-file_open"/><storycode:paste select="#f-42caba993099-folder_session"/><storycode:paste select="#f-42caba993099-recent"/><storycode:paste select="#f-42caba993099-updater"/><storycode:paste select="#f-42caba993099-server"/><storycode:paste select="#f-42caba993099-use_declaration"/><storycode:paste select="#f-42caba993099-use_declaration-2"/><storycode:paste select="#f-42caba993099-use_declaration-3"/><storycode:paste select="#f-42caba993099-use_declaration-4"/><storycode:paste select="#f-42caba993099-use_declaration-5"/><storycode:paste select="#f-42caba993099-QuitIntent"/><storycode:paste select="#f-42caba993099-run"/><storycode:paste select="#f-42caba993099-CloseGate"/><storycode:paste select="#f-42caba993099-WindowFolder"/><storycode:paste select="#f-42caba993099-Accelerators"/><storycode:paste select="#f-42caba993099-impl-Accelerators"/><storycode:paste select="#f-42caba993099-item"/><storycode:paste select="#f-42caba993099-native_window_operation"/><storycode:paste select="#f-42caba993099-app_menu"/><storycode:paste select="#f-42caba993099-InsertGroups"/><storycode:paste select="#f-42caba993099-INSERT_GROUPS"/><storycode:paste select="#f-42caba993099-insert_menu_elements"/><storycode:paste select="#f-42caba993099-insert_menu"/><storycode:paste select="#f-42caba993099-on_menu"/><storycode:paste select="#f-42caba993099-open_file"/><storycode:paste select="#f-42caba993099-open_folder_window"/><storycode:paste select="#f-42caba993099-open_path_in_ui"/><storycode:paste select="#f-42caba993099-dispatch_to_ui"/><storycode:paste select="#f-42caba993099-launch"/><storycode:paste select="#f-42caba993099-open_blank_window"/><storycode:paste select="#f-42caba993099-open_folder_in_new_process"/><storycode:paste select="#f-42caba993099-find_own_app"/><storycode:paste select="#f-42caba993099-open"/><storycode:paste select="#f-42caba993099-open_blank"/><storycode:paste select="#f-42caba993099-folder_name"/><storycode:paste select="#f-42caba993099-native_title"/><storycode:paste select="#f-42caba993099-blank_native_title"/><storycode:paste select="#f-42caba993099-fail"/></storycode:file>

</storycode:doc>
