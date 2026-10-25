<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-grove/src/command.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `c621fbc0116978751bffd254072302a186d685696a7260fcf4dacbd03b63ac92`.

<book:entry path="crates/hick-grove/src/command.rs">
use crate::namespace::DocId;

/// Commands that can be applied to the engine.
pub enum GroveCommand {
    /// Apply a structured XML message to a document.
    StructuredMessage { doc_id: DocId, xml: String },
    /// Apply a raw Yrs binary update to a document.
    BinaryUpdate { doc_id: DocId, update: Vec<u8> },
}

/// Result of processing a command.
pub struct CommandResult {
    pub doc_id: DocId,
    pub events_dispatched: usize,
}
</book:entry>

</book:doc>
