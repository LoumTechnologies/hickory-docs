<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-grove/src/command.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6a412c6b4da0"></a>
## `crates/hick-grove/src/command.rs`

<a id="f-6a412c6b4da0-GroveCommand"></a>
### `GroveCommand`

<storycode:copy id="f-6a412c6b4da0-GroveCommand">
use crate::namespace::DocId;

/// Commands that can be applied to the engine.
pub enum GroveCommand {
    /// Apply a structured XML message to a document.
    StructuredMessage { doc_id: DocId, xml: String },
    /// Apply a raw Yrs binary update to a document.
    BinaryUpdate { doc_id: DocId, update: Vec<u8> },
}

</storycode:copy>
<a id="f-6a412c6b4da0-CommandResult"></a>
### `CommandResult`

<storycode:copy id="f-6a412c6b4da0-CommandResult">
/// Result of processing a command.
pub struct CommandResult {
    pub doc_id: DocId,
    pub events_dispatched: usize,
}
</storycode:copy>

### Assemble `crates/hick-grove/src/command.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-grove/src/command.rs"><storycode:paste select="#f-6a412c6b4da0-GroveCommand"/><storycode:paste select="#f-6a412c6b4da0-CommandResult"/></storycode:file>

</storycode:doc>
