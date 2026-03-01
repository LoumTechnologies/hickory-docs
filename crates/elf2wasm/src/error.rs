use std::path::PathBuf;

/// Errors produced during ELF → WASI transpilation.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unsupported architecture: {arch} (only x86_64 is supported)")]
    UnsupportedArch { arch: String },

    #[error("not an ELF binary: {path}")]
    NotELF { path: PathBuf },

    #[error("not an executable ELF (expected ET_EXEC or ET_DYN): {path}")]
    NotExecutable { path: PathBuf },

    #[error("no entry point found in ELF binary")]
    NoEntryPoint,

    #[error("no code sections found in ELF binary")]
    NoCodeSections,

    #[error("unsupported shared library dependency: {lib} (only static binaries fully supported)")]
    UnsupportedImport { lib: String },

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
