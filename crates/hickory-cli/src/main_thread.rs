use std::process::ExitCode;

/// What `hick --version` reports.
///
/// A downloaded binary should name the release it came from, not the
/// workspace's `Cargo.toml` number, which nobody bumps between releases and
/// which would make every unstable build claim to be `0.1.0`.
/// `scripts/dist.sh` sets `HICKORY_VERSION` when it builds an artifact; a
/// plain `cargo build` leaves it unset and falls back to the crate version
/// **plus the commit it was built from** (`build.rs`), because otherwise
/// every unstable build claims to be `0.1.0` — and a document's woven bytes
/// depend on which build wove them, so two indistinguishable versions produce
/// a drift failure that reads as a content change.
pub const VERSION: &str = match option_env!("HICKORY_VERSION") {
    Some(v) => v,
    None => match option_env!("HICKORY_BUILD_VERSION") {
        Some(v) => v,
        None => env!("CARGO_PKG_VERSION"),
    },
};

/// How much stack the work gets, on every platform.
///
/// Windows gives a process's main thread 1 MiB; Linux and macOS give 8. The
/// runtime's `block_on` runs on that thread, so the whole command — parse,
/// weave, execute, the async chain under all of it — lives inside whatever the
/// platform happened to choose. `hick up --run` overflowed 1 MiB in a debug
/// build and died with "thread 'main' has overflowed its stack", taking the
/// loop down mid-edit (#23).
///
const STACK_SIZE: usize = 16 * 1024 * 1024;

pub fn main_exit() -> ExitCode {
    if let Some(result) =
        hickory_cli::serve::acp::proxy_argv(&std::env::args().skip(1).collect::<Vec<_>>())
    {
        return match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e:#}");
                ExitCode::FAILURE
            }
        };
    }
    match std::thread::Builder::new()
        .name("hick".to_string())
        .stack_size(STACK_SIZE)
        .spawn(super::run)
        .expect("failed to start the main thread")
        .join()
    {
        Ok(code) => code,
        // Re-raise rather than turning it into an exit code: a panic should
        // still look like a panic, with the same status it has always had.
        Err(panic) => std::panic::resume_unwind(panic),
    }
}
