// Desktop entry point.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // `hick init`, run from inside this app, defines git's merge drivers as
    // THIS executable — the app has the engine and the CLI is a separate
    // download that may not be here. So a merge git starts arrives at this
    // `main` with `merge-driver …` or `merge-generated …`, and is answered
    // without opening a window.
    let args: Vec<String> = std::env::args().skip(1).collect();
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
    hickory_desktop_lib::run();
}
