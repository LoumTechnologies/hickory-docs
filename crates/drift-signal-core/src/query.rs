//! Read-model query helpers for drift-signal-core.
//!
//! These functions query the projected SQLite tables populated by the
//! sub-app projections.

use serde::{Deserialize, Serialize};
use waymark_storage::database::Database;

// ── Response types ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationRow {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRow {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRow {
    pub id: String,
    pub key: String,
    pub content: String,
    pub conversation_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    pub id: String,
    pub key: String,
    pub content: String,
    pub rank: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRow {
    pub id: String,
    pub description: String,
    pub status: String,
    pub due_date: Option<String>,
    pub conversation_id: Option<String>,
    pub created_at: String,
}

// ── Query functions ──────────────────────────────────────────────────

pub fn list_conversations(db: &dyn Database) -> anyhow::Result<Vec<ConversationRow>> {
    let rows = db.query_all(
        "SELECT id, title, created_at, updated_at FROM chat_conversations ORDER BY updated_at DESC",
        &[],
    )?;
    Ok(rows
        .iter()
        .filter_map(|r| {
            Some(ConversationRow {
                id: r.get_string(0).ok()?,
                title: r.get_string(1).ok()?,
                created_at: r.get_string(2).ok()?,
                updated_at: r.get_string(3).ok()?,
            })
        })
        .collect())
}

pub fn get_conversation(db: &dyn Database, id: &str) -> anyhow::Result<Option<ConversationRow>> {
    let row = db.query_opt(
        "SELECT id, title, created_at, updated_at FROM chat_conversations WHERE id = ?1",
        &[id.into()],
    )?;
    Ok(row.and_then(|r| {
        Some(ConversationRow {
            id: r.get_string(0).ok()?,
            title: r.get_string(1).ok()?,
            created_at: r.get_string(2).ok()?,
            updated_at: r.get_string(3).ok()?,
        })
    }))
}

pub fn list_messages(db: &dyn Database, conversation_id: &str) -> anyhow::Result<Vec<MessageRow>> {
    let rows = db.query_all(
        "SELECT id, conversation_id, role, content, created_at FROM chat_messages WHERE conversation_id = ?1 ORDER BY created_at ASC",
        &[conversation_id.into()],
    )?;
    Ok(rows
        .iter()
        .filter_map(|r| {
            Some(MessageRow {
                id: r.get_string(0).ok()?,
                conversation_id: r.get_string(1).ok()?,
                role: r.get_string(2).ok()?,
                content: r.get_string(3).ok()?,
                created_at: r.get_string(4).ok()?,
            })
        })
        .collect())
}

pub fn list_recent_messages(
    db: &dyn Database,
    conversation_id: &str,
    limit: usize,
) -> anyhow::Result<Vec<MessageRow>> {
    let rows = db.query_all(
        "SELECT id, conversation_id, role, content, created_at FROM chat_messages WHERE conversation_id = ?1 ORDER BY created_at DESC LIMIT ?2",
        &[conversation_id.into(), (limit as i64).into()],
    )?;
    let mut result: Vec<_> = rows
        .iter()
        .filter_map(|r| {
            Some(MessageRow {
                id: r.get_string(0).ok()?,
                conversation_id: r.get_string(1).ok()?,
                role: r.get_string(2).ok()?,
                content: r.get_string(3).ok()?,
                created_at: r.get_string(4).ok()?,
            })
        })
        .collect();
    result.reverse();
    Ok(result)
}

pub fn delete_conversation(db: &dyn Database, id: &str) -> anyhow::Result<()> {
    db.execute(
        "DELETE FROM chat_messages WHERE conversation_id = ?1",
        &[id.into()],
    )?;
    db.execute(
        "DELETE FROM chat_conversations WHERE id = ?1",
        &[id.into()],
    )?;
    Ok(())
}

pub fn list_memories(db: &dyn Database) -> anyhow::Result<Vec<MemoryRow>> {
    let rows = db.query_all(
        "SELECT id, key, content, conversation_id, created_at FROM mem_memories ORDER BY created_at DESC",
        &[],
    )?;
    Ok(rows
        .iter()
        .filter_map(|r| {
            Some(MemoryRow {
                id: r.get_string(0).ok()?,
                key: r.get_string(1).ok()?,
                content: r.get_string(2).ok()?,
                conversation_id: r.get_opt_string(3).ok()?,
                created_at: r.get_string(4).ok()?,
            })
        })
        .collect())
}

pub fn search_memories(db: &dyn Database, query: &str) -> anyhow::Result<Vec<MemorySearchResult>> {
    let rows = db.query_all(
        "SELECT m.id, m.key, m.content, fts.rank FROM mem_memories_fts fts JOIN mem_memories m ON m.id = fts.id WHERE mem_memories_fts MATCH ?1 ORDER BY fts.rank",
        &[query.into()],
    )?;
    Ok(rows
        .iter()
        .filter_map(|r| {
            Some(MemorySearchResult {
                id: r.get_string(0).ok()?,
                key: r.get_string(1).ok()?,
                content: r.get_string(2).ok()?,
                rank: r.get_f64(3).ok()?,
            })
        })
        .collect())
}

pub fn delete_memory(db: &dyn Database, id: &str) -> anyhow::Result<()> {
    db.execute("DELETE FROM mem_memories WHERE id = ?1", &[id.into()])?;
    Ok(())
}

pub fn list_tasks(db: &dyn Database) -> anyhow::Result<Vec<TaskRow>> {
    let rows = db.query_all(
        "SELECT id, description, status, due_date, conversation_id, created_at FROM task_tasks ORDER BY created_at DESC",
        &[],
    )?;
    Ok(rows
        .iter()
        .filter_map(|r| {
            Some(TaskRow {
                id: r.get_string(0).ok()?,
                description: r.get_string(1).ok()?,
                status: r.get_string(2).ok()?,
                due_date: r.get_opt_string(3).ok()?,
                conversation_id: r.get_opt_string(4).ok()?,
                created_at: r.get_string(5).ok()?,
            })
        })
        .collect())
}

pub fn get_task(db: &dyn Database, id: &str) -> anyhow::Result<Option<TaskRow>> {
    let row = db.query_opt(
        "SELECT id, description, status, due_date, conversation_id, created_at FROM task_tasks WHERE id = ?1",
        &[id.into()],
    )?;
    Ok(row.and_then(|r| {
        Some(TaskRow {
            id: r.get_string(0).ok()?,
            description: r.get_string(1).ok()?,
            status: r.get_string(2).ok()?,
            due_date: r.get_opt_string(3).ok()?,
            conversation_id: r.get_opt_string(4).ok()?,
            created_at: r.get_string(5).ok()?,
        })
    }))
}

// ── Capability queries ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredCapabilityRow {
    pub id: String,
    pub device_id: String,
    pub cap_type: String,
    pub description: String,
    pub registered_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationRow {
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

pub fn list_registered_capabilities(
    db: &dyn Database,
) -> anyhow::Result<Vec<RegisteredCapabilityRow>> {
    let rows = db.query_all(
        "SELECT id, device_id, cap_type, description, registered_at FROM cap_registered ORDER BY registered_at DESC",
        &[],
    )?;
    Ok(rows
        .iter()
        .filter_map(|r| {
            Some(RegisteredCapabilityRow {
                id: r.get_string(0).ok()?,
                device_id: r.get_string(1).ok()?,
                cap_type: r.get_string(2).ok()?,
                description: r.get_string(3).ok()?,
                registered_at: r.get_string(4).ok()?,
            })
        })
        .collect())
}

pub fn list_registered_capabilities_by_type(
    db: &dyn Database,
    cap_type: &str,
) -> anyhow::Result<Vec<RegisteredCapabilityRow>> {
    let rows = db.query_all(
        "SELECT id, device_id, cap_type, description, registered_at FROM cap_registered WHERE cap_type = ?1 ORDER BY registered_at DESC",
        &[cap_type.into()],
    )?;
    Ok(rows
        .iter()
        .filter_map(|r| {
            Some(RegisteredCapabilityRow {
                id: r.get_string(0).ok()?,
                device_id: r.get_string(1).ok()?,
                cap_type: r.get_string(2).ok()?,
                description: r.get_string(3).ok()?,
                registered_at: r.get_string(4).ok()?,
            })
        })
        .collect())
}

pub fn get_invocation(db: &dyn Database, id: &str) -> anyhow::Result<Option<InvocationRow>> {
    let row = db.query_opt(
        "SELECT id, cap_type, target_device, params, status, result, error, invoked_at, completed_at FROM cap_invocations WHERE id = ?1",
        &[id.into()],
    )?;
    Ok(row.and_then(|r| {
        Some(InvocationRow {
            id: r.get_string(0).ok()?,
            cap_type: r.get_string(1).ok()?,
            target_device: r.get_opt_string(2).ok()?,
            params: r.get_opt_string(3).ok()?,
            status: r.get_string(4).ok()?,
            result: r.get_opt_string(5).ok()?,
            error: r.get_opt_string(6).ok()?,
            invoked_at: r.get_string(7).ok()?,
            completed_at: r.get_opt_string(8).ok()?,
        })
    }))
}

pub fn list_pending_invocations(db: &dyn Database) -> anyhow::Result<Vec<InvocationRow>> {
    let rows = db.query_all(
        "SELECT id, cap_type, target_device, params, status, result, error, invoked_at, completed_at FROM cap_invocations WHERE status = 'pending' ORDER BY invoked_at ASC",
        &[],
    )?;
    Ok(rows
        .iter()
        .filter_map(|r| {
            Some(InvocationRow {
                id: r.get_string(0).ok()?,
                cap_type: r.get_string(1).ok()?,
                target_device: r.get_opt_string(2).ok()?,
                params: r.get_opt_string(3).ok()?,
                status: r.get_string(4).ok()?,
                result: r.get_opt_string(5).ok()?,
                error: r.get_opt_string(6).ok()?,
                invoked_at: r.get_string(7).ok()?,
                completed_at: r.get_opt_string(8).ok()?,
            })
        })
        .collect())
}
