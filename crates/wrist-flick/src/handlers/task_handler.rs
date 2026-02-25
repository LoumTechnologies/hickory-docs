use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use hick_grove::namespace::NamespaceUri;
use hick_grove::plugin::{ElementEvent, HandlerContext, NamespaceHandler, SideEffect};

const TASK_NAMESPACE: &str = "https://wrist-flick.dev/ns/task#";

pub struct WfTaskSqliteStore {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WfTaskRow {
    pub id: String,
    pub description: String,
    pub status: String,
    pub due_date: Option<String>,
    pub conversation_id: Option<String>,
    pub created_at: String,
    pub doc_id: String,
}

impl WfTaskSqliteStore {
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
            "CREATE TABLE IF NOT EXISTS wf_tasks (
                id TEXT PRIMARY KEY,
                description TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'open',
                due_date TEXT,
                conversation_id TEXT,
                created_at TEXT NOT NULL,
                doc_id TEXT NOT NULL
            );",
        )?;
        Ok(())
    }

    pub fn create_task(
        &self,
        id: &str,
        description: &str,
        conversation_id: Option<&str>,
        doc_id: &str,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR REPLACE INTO wf_tasks (id, description, conversation_id, created_at, doc_id)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, description, conversation_id, &now, doc_id],
        )?;
        Ok(())
    }

    pub fn update_task(
        &self,
        id: &str,
        description: Option<&str>,
        status: Option<&str>,
        due_date: Option<&str>,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        if let Some(desc) = description {
            conn.execute(
                "UPDATE wf_tasks SET description = ?1 WHERE id = ?2",
                rusqlite::params![desc, id],
            )?;
        }
        if let Some(st) = status {
            conn.execute(
                "UPDATE wf_tasks SET status = ?1 WHERE id = ?2",
                rusqlite::params![st, id],
            )?;
        }
        if let Some(dd) = due_date {
            conn.execute(
                "UPDATE wf_tasks SET due_date = ?1 WHERE id = ?2",
                rusqlite::params![dd, id],
            )?;
        }
        Ok(())
    }

    pub fn complete_task(&self, id: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE wf_tasks SET status = 'done' WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(())
    }

    pub fn delete_task(&self, id: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM wf_tasks WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }

    pub fn get_task(&self, id: &str) -> anyhow::Result<Option<WfTaskRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, description, status, due_date, conversation_id, created_at, doc_id
             FROM wf_tasks WHERE id = ?1",
        )?;
        let row = stmt
            .query_row(rusqlite::params![id], |row| {
                Ok(WfTaskRow {
                    id: row.get(0)?,
                    description: row.get(1)?,
                    status: row.get(2)?,
                    due_date: row.get(3)?,
                    conversation_id: row.get(4)?,
                    created_at: row.get(5)?,
                    doc_id: row.get(6)?,
                })
            })
            .ok();
        Ok(row)
    }

    pub fn list_tasks(&self, doc_id: &str) -> anyhow::Result<Vec<WfTaskRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, description, status, due_date, conversation_id, created_at, doc_id
             FROM wf_tasks WHERE doc_id = ?1
             ORDER BY created_at DESC",
        )?;
        let rows = stmt
            .query_map(rusqlite::params![doc_id], |row| {
                Ok(WfTaskRow {
                    id: row.get(0)?,
                    description: row.get(1)?,
                    status: row.get(2)?,
                    due_date: row.get(3)?,
                    conversation_id: row.get(4)?,
                    created_at: row.get(5)?,
                    doc_id: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

pub struct WfTaskHandler {
    namespace: NamespaceUri,
    store: Arc<WfTaskSqliteStore>,
}

impl WfTaskHandler {
    pub fn new(store: Arc<WfTaskSqliteStore>) -> Self {
        Self {
            namespace: NamespaceUri::new(TASK_NAMESPACE),
            store,
        }
    }

    pub fn store(&self) -> &Arc<WfTaskSqliteStore> {
        &self.store
    }
}

impl NamespaceHandler for WfTaskHandler {
    fn namespace_uri(&self) -> &NamespaceUri {
        &self.namespace
    }

    fn name(&self) -> &str {
        "wf-tasks"
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
                ..
            } => match local_name.as_str() {
                "create" => {
                    let id = attr_value(attributes, "id").unwrap_or_default();
                    let desc = attr_value(attributes, "description").unwrap_or_default();
                    let conv = attr_value(attributes, "conversation");
                    self.store
                        .create_task(&id, &desc, conv.as_deref(), ctx.doc_id.as_str())?;
                    Ok(vec![SideEffect::Log(format!("Created task {}", id))])
                }
                "update" => {
                    let id = attr_value(attributes, "ref")
                        .or_else(|| attr_value(attributes, "id"))
                        .unwrap_or_default();
                    let desc = attr_value(attributes, "description");
                    let status = attr_value(attributes, "status");
                    let due = attr_value(attributes, "due_date");
                    self.store.update_task(
                        &id,
                        desc.as_deref(),
                        status.as_deref(),
                        due.as_deref(),
                    )?;
                    Ok(vec![SideEffect::Log(format!("Updated task {}", id))])
                }
                "complete" => {
                    let id = attr_value(attributes, "ref")
                        .or_else(|| attr_value(attributes, "id"))
                        .unwrap_or_default();
                    self.store.complete_task(&id)?;
                    Ok(vec![SideEffect::Log(format!("Completed task {}", id))])
                }
                "delete" => {
                    let id = attr_value(attributes, "ref")
                        .or_else(|| attr_value(attributes, "id"))
                        .unwrap_or_default();
                    self.store.delete_task(&id)?;
                    Ok(vec![SideEffect::Log(format!("Deleted task {}", id))])
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
    fn test_create_task() {
        let store = Arc::new(WfTaskSqliteStore::new_in_memory().unwrap());
        let handler = WfTaskHandler::new(store.clone());
        let ctx = make_ctx();

        let event = ElementEvent::Created {
            local_name: "create".into(),
            attributes: vec![
                ("id".into(), "task-1".into()),
                ("description".into(), "Follow up on API migration".into()),
            ],
            text_content: String::new(),
            element_id: Some("task-1".to_string()),
        };

        let effects = handler.handle_event(&event, &ctx).unwrap();
        assert_eq!(effects.len(), 1);

        let task = store.get_task("task-1").unwrap().unwrap();
        assert_eq!(task.description, "Follow up on API migration");
        assert_eq!(task.status, "open");
    }

    #[test]
    fn test_complete_task() {
        let store = Arc::new(WfTaskSqliteStore::new_in_memory().unwrap());
        let handler = WfTaskHandler::new(store.clone());
        let ctx = make_ctx();

        let create = ElementEvent::Created {
            local_name: "create".into(),
            attributes: vec![
                ("id".into(), "task-1".into()),
                ("description".into(), "Test task".into()),
            ],
            text_content: String::new(),
            element_id: Some("task-1".to_string()),
        };
        handler.handle_event(&create, &ctx).unwrap();

        let complete = ElementEvent::Created {
            local_name: "complete".into(),
            attributes: vec![("ref".into(), "task-1".into())],
            text_content: String::new(),
            element_id: None,
        };
        handler.handle_event(&complete, &ctx).unwrap();

        let task = store.get_task("task-1").unwrap().unwrap();
        assert_eq!(task.status, "done");
    }

    #[test]
    fn test_update_task() {
        let store = Arc::new(WfTaskSqliteStore::new_in_memory().unwrap());
        let handler = WfTaskHandler::new(store.clone());
        let ctx = make_ctx();

        let create = ElementEvent::Created {
            local_name: "create".into(),
            attributes: vec![
                ("id".into(), "task-1".into()),
                ("description".into(), "Original".into()),
            ],
            text_content: String::new(),
            element_id: Some("task-1".to_string()),
        };
        handler.handle_event(&create, &ctx).unwrap();

        let update = ElementEvent::Created {
            local_name: "update".into(),
            attributes: vec![
                ("ref".into(), "task-1".into()),
                ("description".into(), "Updated description".into()),
                ("due_date".into(), "2026-03-01".into()),
            ],
            text_content: String::new(),
            element_id: None,
        };
        handler.handle_event(&update, &ctx).unwrap();

        let task = store.get_task("task-1").unwrap().unwrap();
        assert_eq!(task.description, "Updated description");
        assert_eq!(task.due_date, Some("2026-03-01".into()));
    }

    #[test]
    fn test_delete_task() {
        let store = Arc::new(WfTaskSqliteStore::new_in_memory().unwrap());
        let handler = WfTaskHandler::new(store.clone());
        let ctx = make_ctx();

        let create = ElementEvent::Created {
            local_name: "create".into(),
            attributes: vec![
                ("id".into(), "task-1".into()),
                ("description".into(), "To delete".into()),
            ],
            text_content: String::new(),
            element_id: Some("task-1".to_string()),
        };
        handler.handle_event(&create, &ctx).unwrap();

        let delete = ElementEvent::Created {
            local_name: "delete".into(),
            attributes: vec![("ref".into(), "task-1".into())],
            text_content: String::new(),
            element_id: None,
        };
        handler.handle_event(&delete, &ctx).unwrap();

        assert!(store.get_task("task-1").unwrap().is_none());
    }
}
