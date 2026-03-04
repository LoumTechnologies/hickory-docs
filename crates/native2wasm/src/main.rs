use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use native2wasm::detect;
use native2wasm::strategy::{Strategy, select_strategy};

#[derive(Parser, Debug)]
#[command(
    name = "native2wasm",
    about = "Any native binary → WASI WebAssembly",
    long_about = "Transpile native binaries (ELF, Mach-O, PE) for x86_64 or ARM64 \
                  to WebAssembly modules targeting WASI.\n\n\
                  Strategies:\n  \
                  aot  — AOT lift via Remill (fastest, static binaries)\n  \
                  emu  — User-mode emulator (dynamic binaries)\n  \
                  vm   — Full Linux VM in WASM (universal fallback)\n  \
                  auto — Automatically select the best strategy",
    version = env!("CARGO_PKG_VERSION"),
)]
struct Cli {
    /// Path to the input native binary
    input: PathBuf,

    /// Output .wasm file path
    #[arg(short, long, default_value = "output.wasm")]
    output: PathBuf,

    /// Transpilation strategy
    #[arg(short, long, value_enum, default_value = "auto")]
    strategy: Strategy,

    /// Optimization level (0-3) passed to llc
    #[arg(short = 'O', long = "opt-level", default_value = "2")]
    opt_level: u8,

    /// Emit LLVM IR (.ll) instead of .wasm — useful for debugging
    #[arg(long)]
    emit_llvm: bool,

    /// Path to Remill semantics directory
    #[arg(long, env = "REMILL_SEMANTICS_DIR")]
    semantics_dir: Option<PathBuf>,

    /// Path to wasi-sysroot for libc headers/libraries
    #[arg(long, env = "WASI_SYSROOT")]
    wasi_sysroot: Option<PathBuf>,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(if cli.verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .format_timestamp(None)
        .init();

    // 1. Detect binary format, architecture, and OS.
    log::info!("detecting: {}", cli.input.display());
    let detected = detect::detect(&cli.input)?;
    log::info!("detected: {detected}");

    // 2. Select strategy.
    let strategy = select_strategy(&detected, cli.strategy);

    // 3. Dispatch.
    match strategy {
        Strategy::Aot => native2wasm::run_aot(
            &cli.input,
            &detected,
            &cli.output,
            cli.opt_level,
            cli.emit_llvm,
            cli.semantics_dir.as_deref(),
            cli.wasi_sysroot.as_deref(),
        ),
        Strategy::Emu => native2wasm::run_emu(&cli.input, &detected, &cli.output),
        Strategy::Vm => native2wasm::run_vm(&cli.input, &detected, &cli.output),
        Strategy::Auto => unreachable!("auto should have been resolved"),
    }
}
