<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-agent/src/receipt.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `2d5aadb159cfff56e7862b41853da2eaf6b2ba9634694330807e31879cebfdb1`.

<book:entry path="crates/hickory-agent/src/receipt.rs">
//! Historical script receipt shape used by session/event readers. No execution.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}
</book:entry>

</book:doc>
