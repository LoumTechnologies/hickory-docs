<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-grove/src/plugins/tasks.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-fc3abc2f9009"></a>
## `crates/hick-grove/src/plugins/tasks.rs`

<a id="f-fc3abc2f9009-TASKS_NAMESPACE"></a>
### `TASKS_NAMESPACE`

<storycode:copy id="f-fc3abc2f9009-TASKS_NAMESPACE">
use std::sync::Mutex;

use rusqlite::Connection;

use crate::namespace::NamespaceUri;
use crate::plugin::{ElementEvent, HandlerContext, NamespaceHandler, SideEffect};

const TASKS_NAMESPACE: &str = "https://example.com/vocab/task#";

</storycode:copy>
<a id="f-fc3abc2f9009-TasksSqliteStore"></a>
### `TasksSqliteStore`

SQLite-backed store for the tasks read model.

<storycode:copy id="f-fc3abc2f9009-TasksSqliteStore">
/// SQLite-backed store for the tasks read model.
pub struct TasksSqliteStore {
    conn: Mutex<Connection>,
}

</storycode:copy>
<a id="f-fc3abc2f9009-impl-TasksSqliteStore"></a>
### `impl-TasksSqliteStore`

<storycode:copy id="f-fc3abc2f9009-impl-TasksSqliteStore">
impl TasksSqliteStore {
    pub fn new_in_memory() -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                description TEXT NOT NULL DEFAULT '',
                assignee TEXT,
                due_date TEXT,
                status TEXT NOT NULL DEFAULT 'open',
                doc_id TEXT NOT NULL
            );",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn create_task(&self, id: &str, description: &str, doc_id: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO tasks (id, description, doc_id) VALUES (?1, ?2, ?3)",
            rusqlite::params![id, description, doc_id],
        )?;
        Ok(())
    }

    pub fn assign_task(&self, task_id: &str, assignee: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE tasks SET assignee = ?1 WHERE id = ?2",
            rusqlite::params![assignee, task_id],
        )?;
        Ok(())
    }

    pub fn set_due_date(&self, task_id: &str, due_date: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE tasks SET due_date = ?1 WHERE id = ?2",
            rusqlite::params![due_date, task_id],
        )?;
        Ok(())
    }

    pub fn get_task(&self, id: &str) -> anyhow::Result<Option<TaskRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, description, assignee, due_date, status, doc_id FROM tasks WHERE id = ?1",
        )?;
        let row = stmt
            .query_row(rusqlite::params![id], |row| {
                Ok(TaskRow {
                    id: row.get(0)?,
                    description: row.get(1)?,
                    assignee: row.get(2)?,
                    due_date: row.get(3)?,
                    status: row.get(4)?,
                    doc_id: row.get(5)?,
                })
            })
            .ok();
        Ok(row)
    }

    pub fn list_tasks(&self, doc_id: &str) -> anyhow::Result<Vec<TaskRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, description, assignee, due_date, status, doc_id FROM tasks WHERE doc_id = ?1",
        )?;
        let rows = stmt
            .query_map(rusqlite::params![doc_id], |row| {
                Ok(TaskRow {
                    id: row.get(0)?,
                    description: row.get(1)?,
                    assignee: row.get(2)?,
                    due_date: row.get(3)?,
                    status: row.get(4)?,
                    doc_id: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

</storycode:copy>
<a id="f-fc3abc2f9009-TaskRow"></a>
### `TaskRow`

<storycode:copy id="f-fc3abc2f9009-TaskRow">
#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskRow {
    pub id: String,
    pub description: String,
    pub assignee: Option<String>,
    pub due_date: Option<String>,
    pub status: String,
    pub doc_id: String,
}

</storycode:copy>
<a id="f-fc3abc2f9009-TasksHandler"></a>
### `TasksHandler`

Namespace handler that projects task events to SQLite.

<storycode:copy id="f-fc3abc2f9009-TasksHandler">
/// Namespace handler that projects task events to SQLite.
pub struct TasksHandler {
    namespace: NamespaceUri,
    store: std::sync::Arc<TasksSqliteStore>,
}

</storycode:copy>
<a id="f-fc3abc2f9009-impl-TasksHandler"></a>
### `impl-TasksHandler`

<storycode:copy id="f-fc3abc2f9009-impl-TasksHandler">
impl TasksHandler {
    pub fn new(store: std::sync::Arc<TasksSqliteStore>) -> Self {
        Self {
            namespace: NamespaceUri::new(TASKS_NAMESPACE),
            store,
        }
    }

    pub fn store(&self) -> &std::sync::Arc<TasksSqliteStore> {
        &self.store
    }
}

</storycode:copy>
<a id="f-fc3abc2f9009-impl-NamespaceHandler-for-TasksHandler"></a>
### `impl-NamespaceHandler for TasksHandler`

<storycode:copy id="f-fc3abc2f9009-impl-NamespaceHandler-for-TasksHandler">
impl NamespaceHandler for TasksHandler {
    fn namespace_uri(&self) -> &NamespaceUri {
        &self.namespace
    }

    fn name(&self) -> &str {
        "tasks"
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
                    self.store.create_task(&id, &desc, ctx.doc_id.as_str())?;
                    Ok(vec![SideEffect::Log(format!("Created task {}", id))])
                }
                "assign" => {
                    let to = attr_value(attributes, "to").unwrap_or_default();
                    // The task id is typically on the parent; for nested elements
                    // we look for a `ref` or `task` attribute
                    let task_id = attr_value(attributes, "ref")
                        .or_else(|| attr_value(attributes, "task"))
                        .unwrap_or_default();
                    if !task_id.is_empty() {
                        self.store.assign_task(&task_id, &to)?;
                    }
                    Ok(vec![SideEffect::Log(format!("Assigned to {}", to))])
                }
                "due" => {
                    let date = attr_value(attributes, "date").unwrap_or_default();
                    let task_id = attr_value(attributes, "ref")
                        .or_else(|| attr_value(attributes, "task"))
                        .unwrap_or_default();
                    if !task_id.is_empty() {
                        self.store.set_due_date(&task_id, &date)?;
                    }
                    Ok(vec![SideEffect::Log(format!("Due date set to {}", date))])
                }
                _ => Ok(vec![]),
            },
            _ => Ok(vec![]),
        }
    }
}

</storycode:copy>
<a id="f-fc3abc2f9009-attr_value"></a>
### `attr_value`

<storycode:copy id="f-fc3abc2f9009-attr_value">
fn attr_value(attrs: &[(String, String)], key: &str) -> Option<String> {
    attrs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}
</storycode:copy>

### Assemble `crates/hick-grove/src/plugins/tasks.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-grove/src/plugins/tasks.rs"><storycode:paste select="#f-fc3abc2f9009-TASKS_NAMESPACE"/><storycode:paste select="#f-fc3abc2f9009-TasksSqliteStore"/><storycode:paste select="#f-fc3abc2f9009-impl-TasksSqliteStore"/><storycode:paste select="#f-fc3abc2f9009-TaskRow"/><storycode:paste select="#f-fc3abc2f9009-TasksHandler"/><storycode:paste select="#f-fc3abc2f9009-impl-TasksHandler"/><storycode:paste select="#f-fc3abc2f9009-impl-NamespaceHandler-for-TasksHandler"/><storycode:paste select="#f-fc3abc2f9009-attr_value"/></storycode:file>

</storycode:doc>
