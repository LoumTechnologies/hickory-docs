use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use hick_grove::namespace::NamespaceUri;
use hick_grove::plugin::{ElementEvent, HandlerContext, NamespaceHandler, SideEffect};

const CHAT_NAMESPACE: &str = "https://wrist-flick.dev/ns/chat#";

pub struct ChatSqliteStore {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationRow {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub doc_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageRow {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
    pub doc_id: String,
}

impl ChatSqliteStore {
    pub fn new_in_memory() -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::init_tables(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn new_with_path(path: &str) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        Self::init_tables(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn init_tables(conn: &Connection) -> anyhow::Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                title TEXT DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                doc_id TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                doc_id TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_messages_conv
                ON messages(conversation_id, created_at);",
        )?;
        Ok(())
    }

    pub fn create_conversation(&self, id: &str, title: &str, doc_id: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR REPLACE INTO conversations (id, title, created_at, updated_at, doc_id)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, title, &now, &now, doc_id],
        )?;
        Ok(())
    }

    pub fn list_conversations(&self, doc_id: &str) -> anyhow::Result<Vec<ConversationRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, created_at, updated_at, doc_id
             FROM conversations WHERE doc_id = ?1
             ORDER BY updated_at DESC",
        )?;
        let rows = stmt
            .query_map(rusqlite::params![doc_id], |row| {
                Ok(ConversationRow {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                    doc_id: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_conversation(&self, id: &str) -> anyhow::Result<Option<ConversationRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, created_at, updated_at, doc_id
             FROM conversations WHERE id = ?1",
        )?;
        let row = stmt
            .query_row(rusqlite::params![id], |row| {
                Ok(ConversationRow {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                    doc_id: row.get(4)?,
                })
            })
            .ok();
        Ok(row)
    }

    pub fn delete_conversation(&self, id: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM messages WHERE conversation_id = ?1",
            rusqlite::params![id],
        )?;
        conn.execute(
            "DELETE FROM conversations WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(())
    }

    pub fn create_message(
        &self,
        id: &str,
        conversation_id: &str,
        role: &str,
        content: &str,
        doc_id: &str,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR REPLACE INTO messages (id, conversation_id, role, content, created_at, doc_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, conversation_id, role, content, &now, doc_id],
        )?;
        conn.execute(
            "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
            rusqlite::params![&now, conversation_id],
        )?;
        Ok(())
    }

    pub fn list_messages(&self, conversation_id: &str) -> anyhow::Result<Vec<MessageRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, conversation_id, role, content, created_at, doc_id
             FROM messages WHERE conversation_id = ?1
             ORDER BY created_at ASC",
        )?;
        let rows = stmt
            .query_map(rusqlite::params![conversation_id], |row| {
                Ok(MessageRow {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    created_at: row.get(4)?,
                    doc_id: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn list_recent_messages(
        &self,
        conversation_id: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<MessageRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, conversation_id, role, content, created_at, doc_id
             FROM messages WHERE conversation_id = ?1
             ORDER BY created_at DESC LIMIT ?2",
        )?;
        let mut rows = stmt
            .query_map(rusqlite::params![conversation_id, limit], |row| {
                Ok(MessageRow {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    created_at: row.get(4)?,
                    doc_id: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        rows.reverse();
        Ok(rows)
    }
}

pub struct ChatHandler {
    namespace: NamespaceUri,
    store: Arc<ChatSqliteStore>,
}

impl ChatHandler {
    pub fn new(store: Arc<ChatSqliteStore>) -> Self {
        Self {
            namespace: NamespaceUri::new(CHAT_NAMESPACE),
            store,
        }
    }

    pub fn store(&self) -> &Arc<ChatSqliteStore> {
        &self.store
    }
}

impl NamespaceHandler for ChatHandler {
    fn namespace_uri(&self) -> &NamespaceUri {
        &self.namespace
    }

    fn name(&self) -> &str {
        "chat"
    }

    fn handle_event(
        &self,
        event: &ElementEvent,
        ctx: &HandlerContext,
    ) -> anyhow::Result<Vec<SideEffect>> {
        match event {
            ElementEvent::Created {
                local_name,
                attributes,
                text_content,
                ..
            } => match local_name.as_str() {
                "conversation" => {
                    let id = attr_value(attributes, "id").unwrap_or_default();
                    let title = attr_value(attributes, "title").unwrap_or_default();
                    self.store
                        .create_conversation(&id, &title, ctx.doc_id.as_str())?;
                    Ok(vec![SideEffect::Log(format!(
                        "Created conversation {}",
                        id
                    ))])
                }
                "message" => {
                    let id = attr_value(attributes, "id").unwrap_or_default();
                    let conversation = attr_value(attributes, "conversation").unwrap_or_default();
                    let role = attr_value(attributes, "role").unwrap_or_default();
                    self.store.create_message(
                        &id,
                        &conversation,
                        &role,
                        text_content,
                        ctx.doc_id.as_str(),
                    )?;
                    Ok(vec![SideEffect::Log(format!("Created message {}", id))])
                }
                _ => Ok(vec![]),
            },
            _ => Ok(vec![]),
        }
    }
}

fn attr_value(attrs: &[(String, String)], key: &str) -> Option<String> {
    attrs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hick_grove::namespace::DocId;
    use hick_grove::plugin::HandlerContext;

    fn make_ctx() -> HandlerContext {
        HandlerContext {
            doc_id: DocId::from_string("test-doc"),
        }
    }

    #[test]
    fn test_create_conversation() {
        let store = Arc::new(ChatSqliteStore::new_in_memory().unwrap());
        let handler = ChatHandler::new(store.clone());
        let ctx = make_ctx();

        let event = ElementEvent::Created {
            local_name: "conversation".into(),
            attributes: vec![
                ("id".into(), "conv-1".into()),
                ("title".into(), "Test Chat".into()),
            ],
            text_content: String::new(),
            element_id: Some("conv-1".to_string()),
        };

        let effects = handler.handle_event(&event, &ctx).unwrap();
        assert_eq!(effects.len(), 1);

        let conv = store.get_conversation("conv-1").unwrap().unwrap();
        assert_eq!(conv.title, "Test Chat");
        assert_eq!(conv.doc_id, "test-doc");
    }

    #[test]
    fn test_create_message() {
        let store = Arc::new(ChatSqliteStore::new_in_memory().unwrap());
        let handler = ChatHandler::new(store.clone());
        let ctx = make_ctx();

        // Create conversation first
        let conv_event = ElementEvent::Created {
            local_name: "conversation".into(),
            attributes: vec![("id".into(), "conv-1".into())],
            text_content: String::new(),
            element_id: Some("conv-1".to_string()),
        };
        handler.handle_event(&conv_event, &ctx).unwrap();

        // Create message
        let msg_event = ElementEvent::Created {
            local_name: "message".into(),
            attributes: vec![
                ("id".into(), "msg-1".into()),
                ("conversation".into(), "conv-1".into()),
                ("role".into(), "user".into()),
            ],
            text_content: "Hello world".into(),
            element_id: Some("msg-1".to_string()),
        };

        let effects = handler.handle_event(&msg_event, &ctx).unwrap();
        assert_eq!(effects.len(), 1);

        let messages = store.list_messages("conv-1").unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "Hello world");
    }

    #[test]
    fn test_list_recent_messages() {
        let store = Arc::new(ChatSqliteStore::new_in_memory().unwrap());
        let handler = ChatHandler::new(store.clone());
        let ctx = make_ctx();

        let conv_event = ElementEvent::Created {
            local_name: "conversation".into(),
            attributes: vec![("id".into(), "conv-1".into())],
            text_content: String::new(),
            element_id: Some("conv-1".to_string()),
        };
        handler.handle_event(&conv_event, &ctx).unwrap();

        for i in 0..5 {
            let msg_event = ElementEvent::Created {
                local_name: "message".into(),
                attributes: vec![
                    ("id".into(), format!("msg-{}", i)),
                    ("conversation".into(), "conv-1".into()),
                    ("role".into(), "user".into()),
                ],
                text_content: format!("Message {}", i),
                element_id: Some(format!("msg-{}", i)),
            };
            handler.handle_event(&msg_event, &ctx).unwrap();
        }

        let recent = store.list_recent_messages("conv-1", 3).unwrap();
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].content, "Message 2");
        assert_eq!(recent[2].content, "Message 4");
    }

    #[test]
    fn test_delete_conversation() {
        let store = Arc::new(ChatSqliteStore::new_in_memory().unwrap());
        let handler = ChatHandler::new(store.clone());
        let ctx = make_ctx();

        let conv_event = ElementEvent::Created {
            local_name: "conversation".into(),
            attributes: vec![("id".into(), "conv-1".into())],
            text_content: String::new(),
            element_id: Some("conv-1".to_string()),
        };
        handler.handle_event(&conv_event, &ctx).unwrap();

        let msg_event = ElementEvent::Created {
            local_name: "message".into(),
            attributes: vec![
                ("id".into(), "msg-1".into()),
                ("conversation".into(), "conv-1".into()),
                ("role".into(), "user".into()),
            ],
            text_content: "Hello".into(),
            element_id: Some("msg-1".to_string()),
        };
        handler.handle_event(&msg_event, &ctx).unwrap();

        store.delete_conversation("conv-1").unwrap();
        assert!(store.get_conversation("conv-1").unwrap().is_none());
        assert!(store.list_messages("conv-1").unwrap().is_empty());
    }
}
