<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/desktop/src-tauri/src/main.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-981e94b5aacd"></a>
## `apps/desktop/src-tauri/src/main.rs`

Wrap the shared engine and web interface in the native desktop host; keep host lifecycle behavior at this boundary.

Related contracts: [A Missing Merge Driver Is A Button, Not A Command To Go And Type](../../../../../guarantees/collaboration/a-missing-merge-driver-is-a-button/contract.md), [Find installed tools when the desktop starts outside a terminal](../../../../../guarantees/integrations/the-desktop-finds-tools-from-the-shell-path/contract.md).

<a id="f-981e94b5aacd-launch_path"></a>
### `launch_path`

<storycode:copy id="f-981e94b5aacd-launch_path">
// Desktop entry point.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "macos")]
mod launch_path;

</storycode:copy>
<a id="f-981e94b5aacd-main"></a>
### `main`

<storycode:copy id="f-981e94b5aacd-main">
fn main() {
    if std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("--hick-cli")) {
        let cli = std::env::current_exe()
            .expect("application executable")
            .parent()
            .expect("application directory")
            .join("hick");
        match std::process::Command::new(cli)
            .args(std::env::args_os().skip(2))
            .status()
        {
            Ok(status) => std::process::exit(status.code().unwrap_or(1)),
            Err(error) => {
                eprintln!("Cannot start the bundled hick command: {error}");
                std::process::exit(1);
            }
        }
    }
    // `hick init`, run from inside this app, defines git's merge drivers as
    // THIS executable — the app has the engine and the CLI is a separate
    // download that may not be here. So a merge git starts arrives at this
    // `main` with `merge-driver …` or `merge-generated …`, and is answered
    // without opening a window.
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(result) = hickory_cli::engine::run_argv(&args) {
        if let Err(error) = result {
            eprintln!("{error:#}");
            std::process::exit(2);
        }
        return;
    }
    if let Some(result) = hickory_cli::serve::workspace_fs::smoke_argv(&args) {
        if let Err(error) = result {
            eprintln!("{error:#}");
            std::process::exit(2);
        }
        return;
    }
    if let Some(result) = hickory_cli::serve::acp::proxy_argv(&args) {
        if let Err(error) = result {
            eprintln!("{error:#}");
            std::process::exit(2);
        }
        return;
    }
    if let Some(outcome) = hickory_cli::merge_driver::run_argv(&args) {
        match outcome {
            Ok(code) => std::process::exit(code),
            Err(e) => {
                eprintln!("hick merge: {e:#}");
                std::process::exit(2);
            }
        }
    }
    // Finder/Dock launches do not inherit the user's shell PATH. Do this
    // before Tauri or any worker thread exists. Engine/merge-driver children
    // inherit it; they must not run the user's profiles again.
    #[cfg(target_os = "macos")]
    match launch_path::load() {
        Ok(path) => {
            // SAFETY: main is still single-threaded; load spawns no threads.
            unsafe {
                std::env::set_var("PATH", &path);
                std::env::set_var("HICKORY_RESOLVED_PATH", path);
            };
        }
        Err(error) => eprintln!("Hickory Docs kept its inherited PATH: {error}"),
    }
    hickory_desktop_lib::run();
}
</storycode:copy>

### Assemble `apps/desktop/src-tauri/src/main.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/desktop/src-tauri/src/main.rs"><storycode:paste select="#f-981e94b5aacd-launch_path"/><storycode:paste select="#f-981e94b5aacd-main"/></storycode:file>

</storycode:doc>
