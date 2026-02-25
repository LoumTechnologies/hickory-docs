use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use hick_grove::namespace::NamespaceUri;
use hick_grove::plugin::{ElementEvent, HandlerContext, NamespaceHandler, SideEffect};

const MEMORY_NAMESPACE: &str = "https://wrist-flick.dev/ns/memory#";

pub struct MemorySqliteStore {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryRow {
    pub id: String,
    pub key: String,
    pub content: String,
    pub conversation_id: Option<String>,
    pub created_at: String,
    pub doc_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemorySearchResult {
    pub id: String,
    pub key: String,
    pub content: String,
    pub rank: f64,
}

impl MemorySqliteStore {
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
            "CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                key TEXT NOT NULL,
                content TEXT NOT NULL,
                conversation_id TEXT,
                created_at TEXT NOT NULL,
                doc_id TEXT NOT NULL
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts
                USING fts5(content, key, content_rowid='rowid');",
        )?;
        Ok(())
    }

    pub fn create_memory(
        &self,
        id: &str,
        key: &str,
        content: &str,
        conversation_id: Option<&str>,
        doc_id: &str,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR REPLACE INTO memories (id, key, content, conversation_id, created_at, doc_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, key, content, conversation_id, &now, doc_id],
        )?;
        // Sync to FTS — use rowid from the main table
        let rowid: i64 = conn.query_row(
            "SELECT rowid FROM memories WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO memories_fts (rowid, content, key) VALUES (?1, ?2, ?3)",
            rusqlite::params![rowid, content, key],
        )?;
        Ok(())
    }

    pub fn delete_memory(&self, id: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        // Remove from FTS first
        if let Ok(rowid) = conn.query_row::<i64, _, _>(
            "SELECT rowid FROM memories WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        ) {
            conn.execute(
                "DELETE FROM memories_fts WHERE rowid = ?1",
                rusqlite::params![rowid],
            )?;
        }
        conn.execute("DELETE FROM memories WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }

    pub fn list_memories(&self, doc_id: &str) -> anyhow::Result<Vec<MemoryRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, key, content, conversation_id, created_at, doc_id
             FROM memories WHERE doc_id = ?1
             ORDER BY created_at DESC",
        )?;
        let rows = stmt
            .query_map(rusqlite::params![doc_id], |row| {
                Ok(MemoryRow {
                    id: row.get(0)?,
                    key: row.get(1)?,
                    content: row.get(2)?,
                    conversation_id: row.get(3)?,
                    created_at: row.get(4)?,
                    doc_id: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn search(&self, query: &str) -> anyhow::Result<Vec<MemorySearchResult>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT m.id, m.key, m.content, fts.rank
             FROM memories_fts fts
             JOIN memories m ON m.rowid = fts.rowid
             WHERE memories_fts MATCH ?1
             ORDER BY fts.rank",
        )?;
        let rows = stmt
            .query_map(rusqlite::params![query], |row| {
                Ok(MemorySearchResult {
                    id: row.get(0)?,
                    key: row.get(1)?,
                    content: row.get(2)?,
                    rank: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn forget_by_key(&self, key: &str, doc_id: &str) -> anyhow::Result<usize> {
        let conn = self.conn.lock().unwrap();
        // Get rowids for FTS cleanup
        let mut stmt = conn.prepare("SELECT rowid FROM memories WHERE key = ?1 AND doc_id = ?2")?;
        let rowids: Vec<i64> = stmt
            .query_map(rusqlite::params![key, doc_id], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        for rowid in &rowids {
            conn.execute(
                "DELETE FROM memories_fts WHERE rowid = ?1",
                rusqlite::params![rowid],
            )?;
        }
        let deleted = conn.execute(
            "DELETE FROM memories WHERE key = ?1 AND doc_id = ?2",
            rusqlite::params![key, doc_id],
        )?;
        Ok(deleted)
    }
}

pub struct MemoryHandler {
    namespace: NamespaceUri,
    store: Arc<MemorySqliteStore>,
}

impl MemoryHandler {
    pub fn new(store: Arc<MemorySqliteStore>) -> Self {
        Self {
            namespace: NamespaceUri::new(MEMORY_NAMESPACE),
            store,
        }
    }

    pub fn store(&self) -> &Arc<MemorySqliteStore> {
        &self.store
    }
}

impl NamespaceHandler for MemoryHandler {
    fn namespace_uri(&self) -> &NamespaceUri {
        &self.namespace
    }

    fn name(&self) -> &str {
        "memory"
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
                "fact" => {
                    let id = attr_value(attributes, "id").unwrap_or_default();
                    let key = attr_value(attributes, "key").unwrap_or_default();
                    let conversation_id = attr_value(attributes, "conversation");
                    self.store.create_memory(
                        &id,
                        &key,
                        text_content,
                        conversation_id.as_deref(),
                        ctx.doc_id.as_str(),
                    )?;
                    Ok(vec![SideEffect::Log(format!("Created memory {}", id))])
                }
                "forget" => {
                    let key = attr_value(attributes, "key").unwrap_or_default();
                    let count = self.store.forget_by_key(&key, ctx.doc_id.as_str())?;
                    Ok(vec![SideEffect::Log(format!(
                        "Forgot {} memories with key '{}'",
                        count, key
                    ))])
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
    fn test_create_memory() {
        let store = Arc::new(MemorySqliteStore::new_in_memory().unwrap());
        let handler = MemoryHandler::new(store.clone());
        let ctx = make_ctx();

        let event = ElementEvent::Created {
            local_name: "fact".into(),
            attributes: vec![
                ("id".into(), "fact-1".into()),
                ("key".into(), "api-status".into()),
            ],
            text_content: "API migration is 60% complete".into(),
            element_id: Some("fact-1".to_string()),
        };

        let effects = handler.handle_event(&event, &ctx).unwrap();
        assert_eq!(effects.len(), 1);

        let memories = store.list_memories("test-doc").unwrap();
        assert_eq!(memories.len(), 1);
        assert_eq!(memories[0].key, "api-status");
        assert_eq!(memories[0].content, "API migration is 60% complete");
    }

    #[test]
    fn test_search_memory() {
        let store = Arc::new(MemorySqliteStore::new_in_memory().unwrap());
        let handler = MemoryHandler::new(store.clone());
        let ctx = make_ctx();

        let event = ElementEvent::Created {
            local_name: "fact".into(),
            attributes: vec![
                ("id".into(), "fact-1".into()),
                ("key".into(), "api-status".into()),
            ],
            text_content: "API migration is 60% complete".into(),
            element_id: Some("fact-1".to_string()),
        };
        handler.handle_event(&event, &ctx).unwrap();

        let results = store.search("migration").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "api-status");
    }

    #[test]
    fn test_forget_memory() {
        let store = Arc::new(MemorySqliteStore::new_in_memory().unwrap());
        let handler = MemoryHandler::new(store.clone());
        let ctx = make_ctx();

        // Create
        let event = ElementEvent::Created {
            local_name: "fact".into(),
            attributes: vec![
                ("id".into(), "fact-1".into()),
                ("key".into(), "old-info".into()),
            ],
            text_content: "Some old fact".into(),
            element_id: Some("fact-1".to_string()),
        };
        handler.handle_event(&event, &ctx).unwrap();

        // Forget
        let forget = ElementEvent::Created {
            local_name: "forget".into(),
            attributes: vec![("key".into(), "old-info".into())],
            text_content: String::new(),
            element_id: None,
        };
        let effects = handler.handle_event(&forget, &ctx).unwrap();
        assert_eq!(effects.len(), 1);

        let memories = store.list_memories("test-doc").unwrap();
        assert!(memories.is_empty());
    }

    #[test]
    fn test_delete_memory() {
        let store = Arc::new(MemorySqliteStore::new_in_memory().unwrap());
        store
            .create_memory("fact-1", "key", "content", None, "test-doc")
            .unwrap();
        store.delete_memory("fact-1").unwrap();
        let memories = store.list_memories("test-doc").unwrap();
        assert!(memories.is_empty());
    }
}
