//! Historical script receipt shape used by session/event readers. No execution.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}
