<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-agent/src/receipt.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-08b731d1e947"></a>
## `crates/hickory-agent/src/receipt.rs`

<a id="f-08b731d1e947-ScriptResult"></a>
### `ScriptResult`

Historical script receipt shape used by session/event readers. No execution.

<storycode:copy id="f-08b731d1e947-ScriptResult">
//! Historical script receipt shape used by session/event readers. No execution.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}
</storycode:copy>

### Assemble `crates/hickory-agent/src/receipt.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-agent/src/receipt.rs"><storycode:paste select="#f-08b731d1e947-ScriptResult"/></storycode:file>

</storycode:doc>
