use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "mach2wasm",
    about = "AOT transpiler: macOS Mach-O x86_64 → WASI WebAssembly",
    version = env!("CARGO_PKG_VERSION"),
)]
pub struct Cli {
    /// Path to the Mach-O x86_64 input binary
    pub input: PathBuf,

    /// Output .wasm file path
    #[arg(short, long, default_value = "output.wasm")]
    pub output: PathBuf,

    /// Optimization level (0-3) passed to llc
    #[arg(short = 'O', long = "opt-level", default_value = "2")]
    pub opt_level: u8,

    /// Emit LLVM IR (.ll) instead of .wasm — useful for debugging
    #[arg(long)]
    pub emit_llvm: bool,

    /// Path to Remill semantics directory (contains lifted bitcode)
    #[arg(long, env = "REMILL_SEMANTICS_DIR")]
    pub semantics_dir: Option<PathBuf>,

    /// Path to wasi-sysroot for libc headers/libraries
    #[arg(long, env = "WASI_SYSROOT")]
    pub wasi_sysroot: Option<PathBuf>,

    /// Verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}
