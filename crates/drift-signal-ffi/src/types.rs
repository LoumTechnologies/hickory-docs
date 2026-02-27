//! FFI-safe record types mirroring drift-signal-core query types.

use drift_signal_core::query;

#[derive(uniffi::Record)]
pub struct FfiConversation {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<query::ConversationRow> for FfiConversation {
    fn from(r: query::ConversationRow) -> Self {
        Self {
            id: r.id,
            title: r.title,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(uniffi::Record)]
pub struct FfiMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

impl From<query::MessageRow> for FfiMessage {
    fn from(r: query::MessageRow) -> Self {
        Self {
            id: r.id,
            conversation_id: r.conversation_id,
            role: r.role,
            content: r.content,
            created_at: r.created_at,
        }
    }
}

#[derive(uniffi::Record)]
pub struct FfiMemory {
    pub id: String,
    pub key: String,
    pub content: String,
    pub conversation_id: Option<String>,
    pub created_at: String,
}

impl From<query::MemoryRow> for FfiMemory {
    fn from(r: query::MemoryRow) -> Self {
        Self {
            id: r.id,
            key: r.key,
            content: r.content,
            conversation_id: r.conversation_id,
            created_at: r.created_at,
        }
    }
}

#[derive(uniffi::Record)]
pub struct FfiMemorySearchResult {
    pub id: String,
    pub key: String,
    pub content: String,
    pub rank: f64,
}

impl From<query::MemorySearchResult> for FfiMemorySearchResult {
    fn from(r: query::MemorySearchResult) -> Self {
        Self {
            id: r.id,
            key: r.key,
            content: r.content,
            rank: r.rank,
        }
    }
}

#[derive(uniffi::Record)]
pub struct FfiTask {
    pub id: String,
    pub description: String,
    pub status: String,
    pub due_date: Option<String>,
    pub conversation_id: Option<String>,
    pub created_at: String,
}

impl From<query::TaskRow> for FfiTask {
    fn from(r: query::TaskRow) -> Self {
        Self {
            id: r.id,
            description: r.description,
            status: r.status,
            due_date: r.due_date,
            conversation_id: r.conversation_id,
            created_at: r.created_at,
        }
    }
}

#[derive(uniffi::Record)]
pub struct FfiDeviceCapability {
    pub id: String,
    pub device_id: String,
    pub cap_type: String,
    pub description: String,
}

impl From<drift_signal_core::capability::types::AppDeviceCapability> for FfiDeviceCapability {
    fn from(c: drift_signal_core::capability::types::AppDeviceCapability) -> Self {
        Self {
            id: c.id,
            device_id: c.device_id,
            cap_type: c.cap_type,
            description: c.description,
        }
    }
}

#[derive(uniffi::Record)]
pub struct FfiRegisteredCapability {
    pub id: String,
    pub device_id: String,
    pub cap_type: String,
    pub description: String,
    pub registered_at: String,
}

impl From<query::RegisteredCapabilityRow> for FfiRegisteredCapability {
    fn from(r: query::RegisteredCapabilityRow) -> Self {
        Self {
            id: r.id,
            device_id: r.device_id,
            cap_type: r.cap_type,
            description: r.description,
            registered_at: r.registered_at,
        }
    }
}

#[derive(uniffi::Record)]
pub struct FfiInvocation {
    pub id: String,
    pub cap_type: String,
    pub target_device: Option<String>,
    pub params: Option<String>,
    pub status: String,
    pub result: Option<String>,
    pub error: Option<String>,
    pub invoked_at: String,
    pub completed_at: Option<String>,
}

impl From<query::InvocationRow> for FfiInvocation {
    fn from(r: query::InvocationRow) -> Self {
        Self {
            id: r.id,
            cap_type: r.cap_type,
            target_device: r.target_device,
            params: r.params,
            status: r.status,
            result: r.result,
            error: r.error,
            invoked_at: r.invoked_at,
            completed_at: r.completed_at,
        }
    }
}
