<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-dap/src/stack_frame.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d9758adb5ff8"></a>
## `crates/hick-dap/src/stack_frame.rs`

<a id="f-d9758adb5ff8-Frame"></a>
### `Frame`

<storycode:copy id="f-d9758adb5ff8-Frame">
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
</storycode:copy>

### Assemble `crates/hick-dap/src/stack_frame.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-dap/src/stack_frame.rs"><storycode:paste select="#f-d9758adb5ff8-Frame"/></storycode:file>

</storycode:doc>
