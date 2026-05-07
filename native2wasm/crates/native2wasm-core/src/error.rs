/// Errors produced during native → WASI transpilation.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("unsupported architecture: {arch} (supported: x86_64, AArch64)")]
    UnsupportedArch { arch: String },

    #[error("no entry point found in binary")]
    NoEntryPoint,

    #[error("no code sections found in binary")]
    NoCodeSections,

    #[error("remill lifting failed: {0}")]
    LiftingFailed(String),

    #[error("tool not found: {tool} (install it or add to PATH)")]
    ToolNotFound { tool: String },

    #[error("opt (bitcode optimization) failed: {0}")]
    OptFailed(String),

    #[error("llc compilation failed: {0}")]
    LlcFailed(String),

    #[error("wasm-ld linking failed: {0}")]
    LinkFailed(String),

    #[error("shim compilation failed: {0}")]
    ShimCompilationFailed(String),

    #[error("binary parse error: {0}")]
    Parse(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
