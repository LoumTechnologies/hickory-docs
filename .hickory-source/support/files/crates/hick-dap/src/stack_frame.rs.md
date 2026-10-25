<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-dap/src/stack_frame.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `6ed47753a40db1bb6d795e5314fce43b7e02241348acac0175589f57142d61ff`.

<book:entry path="crates/hick-dap/src/stack_frame.rs">
use serde::{Deserialize, Serialize};

/// Where execution is, in the document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub id: i64,
    pub name: String,
    /// 0-based document line, when this frame is in a file the document
    /// generated. `None` for a library frame, which is shown read-only rather
    /// than pretended to be part of the document.
    pub line: Option<u32>,
    /// The file as the adapter named it, for frames outside the document.
    pub source: Option<String>,
    /// 0-based line in `source`, after mapping to a contributor document when
    /// possible, otherwise the adapter's raw source line.
    pub source_line: Option<u32>,
    pub in_document: bool,
    /// Raw adapter coordinates, before document mapping; private capture binds
    /// these independently from the human-facing document coordinates.
    #[serde(default)]
    pub adapter_source: Option<String>,
    #[serde(default)]
    pub adapter_line: Option<u32>,
}
</book:entry>

</book:doc>
