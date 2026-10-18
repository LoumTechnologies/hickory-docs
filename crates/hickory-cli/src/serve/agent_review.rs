//! Built-in agents use the same review queue and UI as ACP document edits.
use super::{LocalState, acp::edits::Edits};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct ReviewQueue {
    pub edits: Arc<Edits>,
    pub session: Mutex<Option<std::path::PathBuf>>,
}

pub fn queue(state: &LocalState, doc: &str) -> Arc<ReviewQueue> {
    state
        .agent
        .reviews
        .lock()
        .unwrap()
        .entry(doc.into())
        .or_default()
        .clone()
}
