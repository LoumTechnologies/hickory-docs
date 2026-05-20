use crate::record::EvalRecord;
use anyhow::Result;

/// Abstraction over the eval record store transport.
///
/// `GitBackend` is the real implementation. `NoopBackend` is provided for tests.
/// A future server-backed implementation can be added without changing the record
/// format or the call sites.
pub trait StorageBackend: Send + Sync {
    /// Append a single record to this machine's persistent store.
    fn append(&self, record: &EvalRecord) -> Result<()>;

    /// Read all records from all machines (aggregated view).
    fn read_all(&self) -> Result<Vec<EvalRecord>>;
}
