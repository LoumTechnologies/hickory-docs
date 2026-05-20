//! `HickEditAgent` — composable hick-XML behaviour layered on top of [`AgentHistory`].

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Mutex;

use hick_agent_sdk::AgentHistory;

use crate::file_io::load_hick_sources;
use crate::paste_slot_inserter::insert_paste_slots;
use crate::{HickEditService, PasteSlotRegistry};

/// Adds hick-XML edit capabilities to an [`AgentHistory`].
///
/// `HickEditAgent` holds an [`AgentHistory`] and layers on hick-specific
/// behaviour: discovering paste slots, composing `<hick:copy>` elements for
/// modifications, and composing `<hick:file>` elements for new files.
pub struct HickEditAgent {
    /// Conversation history (messages and taint classifications).
    pub history: AgentHistory,
    /// Hick edit service for pipeline execution and structural queries.
    edit_service: Arc<Mutex<HickEditService>>,
    /// Registry of paste slots discovered from the current pipeline.
    registry: PasteSlotRegistry,
    /// Project directory containing .hick files.
    project_dir: PathBuf,
}

impl HickEditAgent {
    /// Create a new `HickEditAgent` from an [`AgentHistory`].
    pub fn new(history: AgentHistory, project_dir: PathBuf) -> Self {
        Self {
            history,
            edit_service: Arc::new(Mutex::new(HickEditService::new())),
            registry: PasteSlotRegistry::new(),
            project_dir,
        }
    }

    /// Scan the current pipeline sources and populate the paste slot registry.
    ///
    /// Returns a reference to the populated registry.
    pub fn discover_paste_slots(&mut self) -> anyhow::Result<&PasteSlotRegistry> {
        let sources = load_hick_sources(&self.project_dir)?;
        self.registry = PasteSlotRegistry::from_sources(&sources);
        Ok(&self.registry)
    }

    /// Return the current paste slot registry.
    pub fn registry(&self) -> &PasteSlotRegistry {
        &self.registry
    }

    /// Compose a `<hick:copy select="…">` XML element for modifying existing content.
    ///
    /// Returns the XML fragment the agent should insert into a `.hick` source file.
    pub fn compose_copy_element(&self, selector: &str, new_content: &str) -> String {
        format!("<hick:copy select=\"{selector}\">{new_content}</hick:copy>")
    }

    /// Compose a `<hick:file path="…">` XML element for producing a new output file.
    ///
    /// Automatically inserts `<hick:paste name="…"/>` slots at structural boundaries
    /// (function ends, class bodies, import groups, module footer) so agents have
    /// stable, merge-safe composition points from the moment the file is created.
    ///
    /// Returns the XML fragment the agent should insert into a `.hick` source file.
    pub fn compose_file_element(&self, path: &str, content: &str) -> String {
        let content_with_slots = insert_paste_slots(path, content);
        format!("<hick:file path=\"{path}\">{content_with_slots}</hick:file>")
    }

    /// Access the edit service for direct pipeline operations.
    pub fn edit_service(&self) -> Arc<Mutex<HickEditService>> {
        Arc::clone(&self.edit_service)
    }
}
