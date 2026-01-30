//! Multi-document state for tracking element outputs, file outputs, and
//! cross-document references. Adapted from HickoryDocs `xml.rs`.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::node::{Context, InsertionPoint, Node, converge_to_string};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// MultiDocumentState
// ---------------------------------------------------------------------------

/// Global state shared across a hick execution run.
pub struct MultiDocumentState {
    /// Element-keyed insertion points.
    element_outputs: Mutex<HashMap<usize, Arc<InsertionPoint>>>,
    /// File-path-keyed insertion points (for `<hick:file>`).
    file_outputs: Mutex<HashMap<String, Arc<InsertionPoint>>>,
    /// Copy blocks: id → content.
    copy_blocks: Mutex<HashMap<String, String>>,
    /// Cut blocks: id → content (removed from output).
    cut_blocks: Mutex<HashMap<String, String>>,
    /// Counter for generating unique container IDs.
    container_id_counter: std::sync::atomic::AtomicU64,
}

impl Default for MultiDocumentState {
    fn default() -> Self {
        Self {
            element_outputs: Mutex::new(HashMap::new()),
            file_outputs: Mutex::new(HashMap::new()),
            copy_blocks: Mutex::new(HashMap::new()),
            cut_blocks: Mutex::new(HashMap::new()),
            container_id_counter: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

impl MultiDocumentState {
    pub fn add_file_output(&self, path: String, value: Arc<InsertionPoint>) {
        self.file_outputs.lock().unwrap().insert(path, value);
    }

    pub fn add_element_output(&self, key: usize, value: Arc<InsertionPoint>) {
        self.element_outputs.lock().unwrap().insert(key, value);
    }

    pub fn try_get_element_output(&self, key: usize) -> Option<Arc<InsertionPoint>> {
        self.element_outputs.lock().unwrap().get(&key).cloned()
    }

    /// Register a `<hick:copy>` block.
    pub fn register_copy(&self, id: String, content: String) {
        self.copy_blocks.lock().unwrap().insert(id, content);
    }

    /// Register a `<hick:cut>` block.
    pub fn register_cut(&self, id: String, content: String) {
        self.cut_blocks.lock().unwrap().insert(id, content);
    }

    /// Resolve a `<hick:paste select="...">` selector.
    /// Supports `#id` (CSS) and tries cut blocks first, then copy blocks.
    pub fn resolve_paste(&self, selector: &str) -> Option<String> {
        let id = selector.trim_start_matches('#');

        // Check cut blocks first (they're consumed but available via paste)
        if let Some(content) = self.cut_blocks.lock().unwrap().get(id) {
            return Some(content.clone());
        }

        // Then check copy blocks
        if let Some(content) = self.copy_blocks.lock().unwrap().get(id) {
            return Some(content.clone());
        }

        None
    }

    /// Get all file outputs, converging each to a string.
    pub async fn get_files(&self) -> HashMap<String, String> {
        let outputs = self.file_outputs.lock().unwrap().clone();
        let mut result = HashMap::new();

        for (path, insertion_point) in outputs {
            let contents =
                converge_to_string(insertion_point as Arc<dyn Node>, Context::default()).await;
            result.insert(path, contents.unwrap_or_default());
        }

        result
    }

    /// Generate a unique container ID for this session.
    pub fn next_container_id(&self) -> String {
        let id = self
            .container_id_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        format!("container-{id}")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::StringNode;

    #[test]
    fn copy_paste_roundtrip() {
        let state = MultiDocumentState::default();
        state.register_copy("version".to_string(), "3.12.0".to_string());

        let resolved = state.resolve_paste("#version");
        assert_eq!(resolved, Some("3.12.0".to_string()));
    }

    #[test]
    fn cut_available_via_paste() {
        let state = MultiDocumentState::default();
        state.register_cut("secret".to_string(), "hidden data".to_string());

        let resolved = state.resolve_paste("#secret");
        assert_eq!(resolved, Some("hidden data".to_string()));
    }

    #[test]
    fn paste_returns_none_for_missing() {
        let state = MultiDocumentState::default();
        assert_eq!(state.resolve_paste("#nonexistent"), None);
    }

    #[tokio::test]
    async fn file_output_converges() {
        let state = MultiDocumentState::default();
        let ip = Arc::new(InsertionPoint::new());
        ip.add(Arc::new(StringNode::new("hello world")));
        ip.close();
        state.add_file_output("test.txt".to_string(), ip);

        let files = state.get_files().await;
        assert_eq!(files.get("test.txt").unwrap(), "hello world");
    }

    #[test]
    fn container_id_increments() {
        let state = MultiDocumentState::default();
        assert_eq!(state.next_container_id(), "container-0");
        assert_eq!(state.next_container_id(), "container-1");
        assert_eq!(state.next_container_id(), "container-2");
    }
}
