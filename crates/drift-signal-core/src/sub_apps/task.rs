//! Task sub-app — user tasks with status tracking.
//!
//! Event streams:
//! - `TaskCreated` — new task created
//! - `TaskUpdated` — task fields modified
//! - `TaskCompleted` — task marked done
//! - `TaskDeleted` — task removed

use std::collections::BTreeMap;

use waymark_core::{
    AnyProjection, CollectionDef, CommandDef, EventStreamDef, Migration, NamespaceUri, ProjectionOp,
    SqlParam, SubApp,
};

// ── Sub-app ──────────────────────────────────────────────────────────

pub struct TaskSubApp {
    namespace: NamespaceUri,
}

impl TaskSubApp {
    pub fn new() -> Self {
        Self {
            namespace: NamespaceUri::new("https://drift-signal.dev/ns/task#"),
        }
    }
}

impl Default for TaskSubApp {
    fn default() -> Self {
        Self::new()
    }
}

impl SubApp for TaskSubApp {
    fn name(&self) -> &str {
        "task"
    }

    fn namespace(&self) -> &NamespaceUri {
        &self.namespace
    }

    fn ns_prefix(&self) -> &str {
        "task"
    }

    fn event_streams(&self) -> Vec<EventStreamDef> {
        vec![EventStreamDef {
            name: "task_events".into(),
            doc_id: "grove:task:events".into(),
            description: "Task lifecycle events".into(),
        }]
    }

    fn commands(&self) -> Vec<CommandDef> {
        vec![
            CommandDef::new("CreateTask", "task:TaskCreated", "Create a new task")
                .with_required(vec!["id".into(), "description".into()])
                .with_optional(vec!["conversation_id".into(), "due_date".into()]),
            CommandDef::new("UpdateTask", "task:TaskUpdated", "Update task fields")
                .with_required(vec!["id".into()])
                .with_optional(vec![
                    "description".into(),
                    "status".into(),
                    "due_date".into(),
                ]),
            CommandDef::new("CompleteTask", "task:TaskCompleted", "Mark a task as done")
                .with_required(vec!["id".into()]),
            CommandDef::new("DeleteTask", "task:TaskDeleted", "Delete a task")
                .with_required(vec!["id".into()]),
        ]
    }

    fn collections(&self) -> Vec<CollectionDef> {
        vec![CollectionDef::new("tasks", "task", 1)]
    }

    fn projections(&self) -> Vec<Box<dyn AnyProjection>> {
        vec![Box::new(TaskProjection)]
    }

    fn migrations(&self) -> Vec<Migration> {
        vec![Migration::new(
            1,
            "Create tasks table",
            "CREATE TABLE IF NOT EXISTS task_tasks (
                id TEXT PRIMARY KEY,
                description TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'open',
                due_date TEXT,
                conversation_id TEXT,
                created_at TEXT NOT NULL
            );",
        )]
    }

    fn expected_columns(&self) -> Vec<(&str, Vec<(String, String)>)> {
        vec![(
            "task_tasks",
            vec![
                ("id".into(), "TEXT PRIMARY KEY".into()),
                ("description".into(), "TEXT NOT NULL DEFAULT ''".into()),
                ("status".into(), "TEXT NOT NULL DEFAULT 'open'".into()),
                ("due_date".into(), "TEXT".into()),
                ("conversation_id".into(), "TEXT".into()),
                ("created_at".into(), "TEXT NOT NULL".into()),
            ],
        )]
    }
}

// ── Projection ───────────────────────────────────────────────────────

struct TaskProjection;

impl AnyProjection for TaskProjection {
    fn name(&self) -> &str {
        "task_projection"
    }

    fn project_event(
        &self,
        tag: &str,
        attrs: &BTreeMap<String, String>,
        _text: Option<&str>,
        event_id: &str,
        timestamp: i64,
    ) -> Vec<ProjectionOp> {
        let _ = event_id;
        let ts = chrono::DateTime::from_timestamp(timestamp, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default();

        match tag {
            t if t.ends_with(":TaskCreated") => {
                let id = attrs.get("id").cloned().unwrap_or_default();
                let description = attrs.get("description").cloned().unwrap_or_default();
                let conv_id = attrs.get("conversation_id").cloned();
                let due_date = attrs.get("due_date").cloned();
                vec![ProjectionOp::sql(
                    "INSERT OR REPLACE INTO task_tasks (id, description, status, due_date, conversation_id, created_at) VALUES (?1, ?2, 'open', ?3, ?4, ?5)",
                    vec![
                        SqlParam::Text(id),
                        SqlParam::Text(description),
                        due_date.map_or(SqlParam::Null, SqlParam::Text),
                        conv_id.map_or(SqlParam::Null, SqlParam::Text),
                        SqlParam::Text(ts),
                    ],
                )]
            }
            t if t.ends_with(":TaskUpdated") => {
                let id = attrs.get("id").cloned().unwrap_or_default();
                let mut ops = Vec::new();
                if let Some(desc) = attrs.get("description") {
                    ops.push(ProjectionOp::sql(
                        "UPDATE task_tasks SET description = ?1 WHERE id = ?2",
                        vec![SqlParam::Text(desc.clone()), SqlParam::Text(id.clone())],
                    ));
                }
                if let Some(status) = attrs.get("status") {
                    ops.push(ProjectionOp::sql(
                        "UPDATE task_tasks SET status = ?1 WHERE id = ?2",
                        vec![SqlParam::Text(status.clone()), SqlParam::Text(id.clone())],
                    ));
                }
                if let Some(due) = attrs.get("due_date") {
                    ops.push(ProjectionOp::sql(
                        "UPDATE task_tasks SET due_date = ?1 WHERE id = ?2",
                        vec![SqlParam::Text(due.clone()), SqlParam::Text(id.clone())],
                    ));
                }
                ops
            }
            t if t.ends_with(":TaskCompleted") => {
                let id = attrs.get("id").cloned().unwrap_or_default();
                vec![ProjectionOp::sql(
                    "UPDATE task_tasks SET status = 'done' WHERE id = ?1",
                    vec![SqlParam::Text(id)],
                )]
            }
            t if t.ends_with(":TaskDeleted") => {
                let id = attrs.get("id").cloned().unwrap_or_default();
                vec![ProjectionOp::sql(
                    "DELETE FROM task_tasks WHERE id = ?1",
                    vec![SqlParam::Text(id)],
                )]
            }
            _ => vec![],
        }
    }
}
