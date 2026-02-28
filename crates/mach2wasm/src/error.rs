use std::path::PathBuf;

/// Errors produced during Mach-O → WASI transpilation.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unsupported architecture: {arch} (only x86_64 is supported)")]
    UnsupportedArch { arch: String },

    #[error("not a Mach-O binary: {path}")]
    NotMachO { path: PathBuf },

    #[error("no x86_64 slice found in universal binary: {path}")]
    NoX86Slice { path: PathBuf },

    #[error("no entry point found in Mach-O binary")]
    NoEntryPoint,

    #[error("no code sections found in Mach-O binary")]
    NoCodeSections,

    #[error("remill lifting failed: {0}")]
    LiftingFailed(String),

    #[error("tool not found: {tool} (install it or add to PATH)")]
    ToolNotFound { tool: String },

    #[error("llc compilation failed: {0}")]
    LlcFailed(String),

    #[error("wasm-ld linking failed: {0}")]
    LinkFailed(String),

    #[error("shim compilation failed: {0}")]
    ShimCompilationFailed(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
