//! `AppDriftSignal` — top-level facade wrapping waymark SyncApp + agent engine.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::Value;
use waymark_sdk::SyncApp;

use crate::capability::dispatcher::AppCapabilityDispatcher;
use crate::capability::types::{AppCapabilityHandler, AppDeviceCapability, new_pending_queue};
use crate::query::{
    self, ConversationRow, InvocationRow, MemoryRow, MemorySearchResult, MessageRow,
    RegisteredCapabilityRow, TaskRow,
};
use crate::sub_apps::{CapabilitiesSubApp, ChatSubApp, MemorySubApp, TaskSubApp};

/// Configuration for opening a Drift Signal instance.
pub struct DriftSignalConfig {
    pub user_id: String,
    pub device_id: String,
    pub db_path: String,
}

/// Top-level facade for the Drift Signal multi-frontend agent system.
///
/// Wraps a waymark `SyncApp` with registered domain sub-apps (chat, memory,
/// task, capabilities) and an optional embedded agent engine.
pub struct AppDriftSignal {
    sync_app: SyncApp,
    dispatcher: AppCapabilityDispatcher,
    device_id: String,
}

impl AppDriftSignal {
    /// Open a new Drift Signal instance, registering all sub-apps.
    pub fn open(config: DriftSignalConfig) -> anyhow::Result<Self> {
        let pending = new_pending_queue();

        let mut sync_app = SyncApp::open(Path::new(&config.db_path), &config.user_id)?;

        // Register domain sub-apps
        sync_app.register_sub_app(Box::new(ChatSubApp::new()))?;
        sync_app.register_sub_app(Box::new(MemorySubApp::new()))?;
        sync_app.register_sub_app(Box::new(TaskSubApp::new()))?;
        sync_app.register_sub_app(Box::new(CapabilitiesSubApp::new(
            config.device_id.clone(),
            pending.clone(),
        )))?;

        let dispatcher = AppCapabilityDispatcher::new(config.device_id.clone(), pending);

        Ok(Self {
            sync_app,
            dispatcher,
            device_id: config.device_id,
        })
    }

    /// Register a local capability handler and advertise it in the event stream.
    pub fn register_capability(
        &mut self,
        handler: Box<dyn AppCapabilityHandler>,
    ) -> anyhow::Result<()> {
        let cap_id = format!("cap-{}-{}", self.device_id, handler.cap_type());
        let cap_type = handler.cap_type().to_string();
        let description = handler.description().to_string();

        self.dispatcher.register(handler);

        // Advertise in the event stream so other devices can see it
        let mut attrs = BTreeMap::new();
        attrs.insert("id".into(), cap_id);
        attrs.insert("device_id".into(), self.device_id.clone());
        attrs.insert("cap_type".into(), cap_type);
        attrs.insert("description".into(), description);
        self.sync_app
            .execute_command("capabilities", "RegisterCapability", &attrs, None)?;

        Ok(())
    }

    /// Device ID for this instance.
    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    // ── Chat operations ──────────────────────────────────────────────

    /// Create a new chat conversation.
    pub fn create_conversation(&mut self, title: &str) -> anyhow::Result<String> {
        let id = format!("conv-{}", uuid::Uuid::now_v7());
        let mut attrs = BTreeMap::new();
        attrs.insert("id".into(), id.clone());
        attrs.insert("title".into(), title.to_string());
        self.sync_app
            .execute_command("chat", "CreateConversation", &attrs, None)?;
        Ok(id)
    }

    /// Add a message to a conversation.
    pub fn send_message(
        &mut self,
        conversation_id: &str,
        role: &str,
        content: &str,
    ) -> anyhow::Result<String> {
        let id = format!("msg-{}", uuid::Uuid::now_v7());
        let mut attrs = BTreeMap::new();
        attrs.insert("id".into(), id.clone());
        attrs.insert("conversation_id".into(), conversation_id.to_string());
        attrs.insert("role".into(), role.to_string());
        self.sync_app
            .execute_command("chat", "CreateMessage", &attrs, Some(content))?;
        Ok(id)
    }

    // ── Memory operations ────────────────────────────────────────────

    /// Store a memory fact.
    pub fn create_fact(
        &mut self,
        key: &str,
        content: &str,
        conversation_id: Option<&str>,
    ) -> anyhow::Result<String> {
        let id = format!("fact-{}", uuid::Uuid::now_v7());
        let mut attrs = BTreeMap::new();
        attrs.insert("id".into(), id.clone());
        attrs.insert("key".into(), key.to_string());
        if let Some(conv) = conversation_id {
            attrs.insert("conversation_id".into(), conv.to_string());
        }
        self.sync_app
            .execute_command("memory", "CreateFact", &attrs, Some(content))?;
        Ok(id)
    }

    /// Forget memories by key.
    pub fn forget(&mut self, key: &str) -> anyhow::Result<String> {
        let mut attrs = BTreeMap::new();
        attrs.insert("key".into(), key.to_string());
        let event_id = self
            .sync_app
            .execute_command("memory", "Forget", &attrs, None)?;
        Ok(event_id)
    }

    // ── Task operations ──────────────────────────────────────────────

    /// Create a new task.
    pub fn create_task(
        &mut self,
        description: &str,
        conversation_id: Option<&str>,
    ) -> anyhow::Result<String> {
        let id = format!("task-{}", uuid::Uuid::now_v7());
        let mut attrs = BTreeMap::new();
        attrs.insert("id".into(), id.clone());
        attrs.insert("description".into(), description.to_string());
        if let Some(conv) = conversation_id {
            attrs.insert("conversation_id".into(), conv.to_string());
        }
        self.sync_app
            .execute_command("task", "CreateTask", &attrs, None)?;
        Ok(id)
    }

    /// Complete a task.
    pub fn complete_task(&mut self, task_id: &str) -> anyhow::Result<String> {
        let mut attrs = BTreeMap::new();
        attrs.insert("id".into(), task_id.to_string());
        let event_id = self
            .sync_app
            .execute_command("task", "CompleteTask", &attrs, None)?;
        Ok(event_id)
    }

    /// Delete a task.
    pub fn delete_task(&mut self, task_id: &str) -> anyhow::Result<String> {
        let mut attrs = BTreeMap::new();
        attrs.insert("id".into(), task_id.to_string());
        let event_id = self
            .sync_app
            .execute_command("task", "DeleteTask", &attrs, None)?;
        Ok(event_id)
    }

    // ── Capability operations ────────────────────────────────────────

    /// List capabilities available on this device.
    pub fn list_capabilities(&self) -> Vec<AppDeviceCapability> {
        self.dispatcher.local_capabilities()
    }

    /// List all registered capabilities across all synced devices.
    pub fn list_registered_capabilities(&self) -> anyhow::Result<Vec<RegisteredCapabilityRow>> {
        query::list_registered_capabilities(self.sync_app.query_read_model())
    }

    /// Find which devices offer a specific capability type.
    pub fn find_capability_providers(
        &self,
        cap_type: &str,
    ) -> anyhow::Result<Vec<RegisteredCapabilityRow>> {
        query::list_registered_capabilities_by_type(self.sync_app.query_read_model(), cap_type)
    }

    /// Invoke a capability, dispatching locally or remotely.
    pub async fn invoke_capability(
        &mut self,
        cap_type: &str,
        target: Option<&str>,
        params: Value,
    ) -> anyhow::Result<String> {
        self.dispatcher
            .invoke(&mut self.sync_app, cap_type, target, params)
            .await
    }

    /// Process pending remote invocations queued by sync.
    ///
    /// Call this periodically or after receiving sync updates to execute
    /// capabilities requested by other devices.
    pub async fn process_pending_invocations(&mut self) -> anyhow::Result<usize> {
        self.dispatcher.process_pending(&mut self.sync_app).await
    }

    /// Get the status of a capability invocation.
    pub fn get_invocation(&self, id: &str) -> anyhow::Result<Option<InvocationRow>> {
        query::get_invocation(self.sync_app.query_read_model(), id)
    }

    /// List all pending capability invocations.
    pub fn list_pending_invocations(&self) -> anyhow::Result<Vec<InvocationRow>> {
        query::list_pending_invocations(self.sync_app.query_read_model())
    }

    /// Access the underlying SyncApp for advanced queries or raw commands.
    pub fn sync_app(&self) -> &SyncApp {
        &self.sync_app
    }

    /// Mutable access to the underlying SyncApp.
    pub fn sync_app_mut(&mut self) -> &mut SyncApp {
        &mut self.sync_app
    }

    // ── Read operations (query projected read model) ─────────────────

    pub fn list_conversations(&self) -> anyhow::Result<Vec<ConversationRow>> {
        query::list_conversations(self.sync_app.query_read_model())
    }

    pub fn get_conversation(&self, id: &str) -> anyhow::Result<Option<ConversationRow>> {
        query::get_conversation(self.sync_app.query_read_model(), id)
    }

    pub fn list_messages(&self, conversation_id: &str) -> anyhow::Result<Vec<MessageRow>> {
        query::list_messages(self.sync_app.query_read_model(), conversation_id)
    }

    pub fn list_recent_messages(
        &self,
        conversation_id: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<MessageRow>> {
        query::list_recent_messages(self.sync_app.query_read_model(), conversation_id, limit)
    }

    pub fn delete_conversation(&self, id: &str) -> anyhow::Result<()> {
        query::delete_conversation(self.sync_app.query_read_model(), id)
    }

    pub fn list_memories(&self) -> anyhow::Result<Vec<MemoryRow>> {
        query::list_memories(self.sync_app.query_read_model())
    }

    pub fn search_memories(&self, q: &str) -> anyhow::Result<Vec<MemorySearchResult>> {
        query::search_memories(self.sync_app.query_read_model(), q)
    }

    pub fn delete_memory(&self, id: &str) -> anyhow::Result<()> {
        query::delete_memory(self.sync_app.query_read_model(), id)
    }

    pub fn list_tasks(&self) -> anyhow::Result<Vec<TaskRow>> {
        query::list_tasks(self.sync_app.query_read_model())
    }

    pub fn get_task(&self, id: &str) -> anyhow::Result<Option<TaskRow>> {
        query::get_task(self.sync_app.query_read_model(), id)
    }
}
