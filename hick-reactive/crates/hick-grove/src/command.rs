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
