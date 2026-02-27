uniffi::setup_scaffolding!();

mod error;
mod logging;
mod types;

use std::sync::Mutex;

use drift_signal_core::{AppDriftSignal, DriftSignalConfig, SyncConfig};

use crate::error::DriftSignalError;
use crate::types::*;

// MARK: - VSLICE: DriftSignalFFI @FFI

/// UniFFI-exported engine wrapping `AppDriftSignal`.
///
/// All methods acquire a `std::sync::Mutex` lock (blocking, FFI-safe — no async).
/// Async operations use a dedicated tokio runtime.
#[derive(uniffi::Object)]
pub struct FfiDriftSignalEngine {
    inner: Mutex<AppDriftSignal>,
    runtime: tokio::runtime::Runtime,
}

#[uniffi::export]
impl FfiDriftSignalEngine {
    /// Open a new Drift Signal instance.
    #[uniffi::constructor]
    pub fn open(
        db_path: String,
        user_id: String,
        device_id: String,
    ) -> Result<Self, DriftSignalError> {
        let config = DriftSignalConfig {
            user_id,
            device_id,
            db_path,
        };
        let core = AppDriftSignal::open(config)?;
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| DriftSignalError::EngineError { message: e.to_string() })?;
        Ok(Self {
            inner: Mutex::new(core),
            runtime,
        })
    }

    /// Device ID for this instance.
    pub fn device_id(&self) -> Result<String, DriftSignalError> {
        let core = self.inner.lock()?;
        Ok(core.device_id().to_string())
    }

    // ── Sync ─────────────────────────────────────────────────────────

    /// Start P2P sync. Optionally provide a relay URL for NAT traversal.
    pub fn start_sync(&self, relay_url: Option<String>) -> Result<(), DriftSignalError> {
        let mut core = self.inner.lock()?;
        let mut config = SyncConfig::default();
        if let Some(url) = relay_url {
            config = config.with_cloud(url);
        }
        self.runtime.block_on(core.start_sync(config))?;
        Ok(())
    }

    /// Stop P2P sync and persist final state.
    pub fn stop_sync(&self) -> Result<(), DriftSignalError> {
        let mut core = self.inner.lock()?;
        self.runtime.block_on(core.stop_sync())?;
        Ok(())
    }

    /// Get the iroh node ID, if sync is running.
    pub fn node_id(&self) -> Result<Option<String>, DriftSignalError> {
        let core = self.inner.lock()?;
        Ok(core.node_id())
    }

    /// Get the current connection status.
    ///
    /// Returns one of: "offline", "connected".
    pub fn connection_status(&self) -> Result<String, DriftSignalError> {
        let core = self.inner.lock()?;
        Ok(core.connection_status())
    }

    /// Generate a device link token for pairing a new device.
    ///
    /// Display the returned string as a QR code. The other device
    /// scans it and calls `accept_device_link` with the same string.
    pub fn generate_link_token(&self) -> Result<String, DriftSignalError> {
        let core = self.inner.lock()?;
        Ok(core.generate_link_token())
    }

    // ── Chat ────────────────────────────────────────────────────────

    /// Create a new conversation, returning its ID.
    pub fn create_conversation(&self, title: String) -> Result<String, DriftSignalError> {
        let mut core = self.inner.lock()?;
        Ok(core.create_conversation(&title)?)
    }

    /// List all conversations, newest first.
    pub fn list_conversations(&self) -> Result<Vec<FfiConversation>, DriftSignalError> {
        let core = self.inner.lock()?;
        Ok(core
            .list_conversations()?
            .into_iter()
            .map(FfiConversation::from)
            .collect())
    }

    /// Get a single conversation by ID.
    pub fn get_conversation(
        &self,
        id: String,
    ) -> Result<Option<FfiConversation>, DriftSignalError> {
        let core = self.inner.lock()?;
        Ok(core.get_conversation(&id)?.map(FfiConversation::from))
    }

    /// Add a message to a conversation, returning the message ID.
    pub fn send_message(
        &self,
        conversation_id: String,
        role: String,
        content: String,
    ) -> Result<String, DriftSignalError> {
        let mut core = self.inner.lock()?;
        Ok(core.send_message(&conversation_id, &role, &content)?)
    }

    /// List all messages in a conversation, oldest first.
    pub fn list_messages(
        &self,
        conversation_id: String,
    ) -> Result<Vec<FfiMessage>, DriftSignalError> {
        let core = self.inner.lock()?;
        Ok(core
            .list_messages(&conversation_id)?
            .into_iter()
            .map(FfiMessage::from)
            .collect())
    }

    /// List recent messages in a conversation.
    pub fn list_recent_messages(
        &self,
        conversation_id: String,
        limit: u32,
    ) -> Result<Vec<FfiMessage>, DriftSignalError> {
        let core = self.inner.lock()?;
        Ok(core
            .list_recent_messages(&conversation_id, limit as usize)?
            .into_iter()
            .map(FfiMessage::from)
            .collect())
    }

    /// Delete a conversation and its messages.
    pub fn delete_conversation(&self, id: String) -> Result<(), DriftSignalError> {
        let core = self.inner.lock()?;
        Ok(core.delete_conversation(&id)?)
    }

    // ── Memory ──────────────────────────────────────────────────────

    /// Store a memory fact, returning its ID.
    pub fn create_fact(
        &self,
        key: String,
        content: String,
        conversation_id: Option<String>,
    ) -> Result<String, DriftSignalError> {
        let mut core = self.inner.lock()?;
        Ok(core.create_fact(&key, &content, conversation_id.as_deref())?)
    }

    /// List all memories, newest first.
    pub fn list_memories(&self) -> Result<Vec<FfiMemory>, DriftSignalError> {
        let core = self.inner.lock()?;
        Ok(core
            .list_memories()?
            .into_iter()
            .map(FfiMemory::from)
            .collect())
    }

    /// Full-text search over memories.
    pub fn search_memories(
        &self,
        query: String,
    ) -> Result<Vec<FfiMemorySearchResult>, DriftSignalError> {
        let core = self.inner.lock()?;
        Ok(core
            .search_memories(&query)?
            .into_iter()
            .map(FfiMemorySearchResult::from)
            .collect())
    }

    /// Forget memories by key.
    pub fn forget(&self, key: String) -> Result<(), DriftSignalError> {
        let mut core = self.inner.lock()?;
        core.forget(&key)?;
        Ok(())
    }

    /// Delete a single memory by ID.
    pub fn delete_memory(&self, id: String) -> Result<(), DriftSignalError> {
        let core = self.inner.lock()?;
        Ok(core.delete_memory(&id)?)
    }

    // ── Tasks ───────────────────────────────────────────────────────

    /// Create a new task, returning its ID.
    pub fn create_task(
        &self,
        description: String,
        conversation_id: Option<String>,
    ) -> Result<String, DriftSignalError> {
        let mut core = self.inner.lock()?;
        Ok(core.create_task(&description, conversation_id.as_deref())?)
    }

    /// Mark a task as complete.
    pub fn complete_task(&self, task_id: String) -> Result<(), DriftSignalError> {
        let mut core = self.inner.lock()?;
        core.complete_task(&task_id)?;
        Ok(())
    }

    /// Delete a task.
    pub fn delete_task(&self, task_id: String) -> Result<(), DriftSignalError> {
        let mut core = self.inner.lock()?;
        core.delete_task(&task_id)?;
        Ok(())
    }

    /// List all tasks, newest first.
    pub fn list_tasks(&self) -> Result<Vec<FfiTask>, DriftSignalError> {
        let core = self.inner.lock()?;
        Ok(core
            .list_tasks()?
            .into_iter()
            .map(FfiTask::from)
            .collect())
    }

    /// Get a single task by ID.
    pub fn get_task(&self, id: String) -> Result<Option<FfiTask>, DriftSignalError> {
        let core = self.inner.lock()?;
        Ok(core.get_task(&id)?.map(FfiTask::from))
    }

    // ── Capabilities ────────────────────────────────────────────────

    /// List all locally registered capabilities.
    pub fn list_capabilities(&self) -> Result<Vec<FfiDeviceCapability>, DriftSignalError> {
        let core = self.inner.lock()?;
        Ok(core
            .list_capabilities()
            .into_iter()
            .map(FfiDeviceCapability::from)
            .collect())
    }

    /// List all capabilities registered across all synced devices.
    pub fn list_registered_capabilities(
        &self,
    ) -> Result<Vec<FfiRegisteredCapability>, DriftSignalError> {
        let core = self.inner.lock()?;
        Ok(core
            .list_registered_capabilities()?
            .into_iter()
            .map(FfiRegisteredCapability::from)
            .collect())
    }

    /// Find which devices offer a specific capability type.
    pub fn find_capability_providers(
        &self,
        cap_type: String,
    ) -> Result<Vec<FfiRegisteredCapability>, DriftSignalError> {
        let core = self.inner.lock()?;
        Ok(core
            .find_capability_providers(&cap_type)?
            .into_iter()
            .map(FfiRegisteredCapability::from)
            .collect())
    }

    /// Get the status of a capability invocation.
    pub fn get_invocation(
        &self,
        id: String,
    ) -> Result<Option<FfiInvocation>, DriftSignalError> {
        let core = self.inner.lock()?;
        Ok(core.get_invocation(&id)?.map(FfiInvocation::from))
    }

    /// List all pending capability invocations.
    pub fn list_pending_invocations(&self) -> Result<Vec<FfiInvocation>, DriftSignalError> {
        let core = self.inner.lock()?;
        Ok(core
            .list_pending_invocations()?
            .into_iter()
            .map(FfiInvocation::from)
            .collect())
    }
}
