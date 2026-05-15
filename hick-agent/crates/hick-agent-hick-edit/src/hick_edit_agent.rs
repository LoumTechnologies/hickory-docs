//! `HickEditAgent` — composable hick-XML behaviour layered on top of `AgentRuntime`.

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Mutex;

use hick_agent_sdk::AgentRuntime;

use crate::file_io::load_hick_sources;
use crate::{HickEditService, PasteSlotRegistry};

/// Adds hick-XML edit capabilities to an [`AgentRuntime`].
///
/// `HickEditAgent` wraps the generic SDK runtime and layers on hick-specific
/// behaviour: discovering paste slots, composing `<hick:copy>` elements for
/// modifications, and composing `<hick:file>` elements for new files.
pub struct HickEditAgent {
    /// Underlying generic agent runtime (LLM, history, sandbox).
    pub runtime: AgentRuntime,
    /// Hick edit service for pipeline execution and structural queries.
    edit_service: Arc<Mutex<HickEditService>>,
    /// Registry of paste slots discovered from the current pipeline.
    registry: PasteSlotRegistry,
    /// Project directory containing .hick files.
    project_dir: PathBuf,
}

impl HickEditAgent {
    /// Create a new `HickEditAgent` from an existing [`AgentRuntime`].
    pub fn new(runtime: AgentRuntime, project_dir: PathBuf) -> Self {
        Self {
            runtime,
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
    /// Returns the XML fragment the agent should insert into a `.hick` source file.
    pub fn compose_file_element(&self, path: &str, content: &str) -> String {
        format!("<hick:file path=\"{path}\">{content}</hick:file>")
    }

    /// Access the edit service for direct pipeline operations.
    pub fn edit_service(&self) -> Arc<Mutex<HickEditService>> {
        Arc::clone(&self.edit_service)
    }
}
