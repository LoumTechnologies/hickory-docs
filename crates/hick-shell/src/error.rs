/// Errors produced by the hick-shell interpreter.
#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    #[error("command not found: {0}")]
    CommandNotFound(String),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("WASI execution failed: {0}")]
    WasiExec(#[from] anyhow::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
