//! Memory sub-app — persistent facts with full-text search.
//!
//! Event streams:
//! - `MemoryFactCreated` — a new memory fact stored
//! - `MemoryForgotten` — a memory key erased

use std::collections::BTreeMap;

use waymark_core::{
    AnyProjection, CollectionDef, CommandDef, EventStreamDef, Migration, NamespaceUri, ProjectionOp,
    SqlParam, SubApp,
};


// ── Sub-app ──────────────────────────────────────────────────────────

pub struct MemorySubApp {
    namespace: NamespaceUri,
}

impl MemorySubApp {
    pub fn new() -> Self {
        Self {
            namespace: NamespaceUri::new("https://drift-signal.dev/ns/memory#"),
        }
    }
}

impl Default for MemorySubApp {
    fn default() -> Self {
        Self::new()
    }
}

impl SubApp for MemorySubApp {
    fn name(&self) -> &str {
        "memory"
    }

    fn namespace(&self) -> &NamespaceUri {
        &self.namespace
    }

    fn ns_prefix(&self) -> &str {
        "mem"
    }

    fn event_streams(&self) -> Vec<EventStreamDef> {
        vec![EventStreamDef {
            name: "memory_events".into(),
            doc_id: "grove:memory:events".into(),
            description: "Memory fact creation and deletion events".into(),
        }]
    }

    fn commands(&self) -> Vec<CommandDef> {
        vec![
            CommandDef::new(
                "CreateFact",
                "mem:FactCreated",
                "Store a new memory fact",
            )
            .with_required(vec!["id".into(), "key".into()])
            .with_optional(vec!["conversation_id".into()]),
            CommandDef::new(
                "Forget",
                "mem:Forgotten",
                "Erase memories by key",
            )
            .with_required(vec!["key".into()]),
        ]
    }

    fn collections(&self) -> Vec<CollectionDef> {
        vec![CollectionDef::new("memories", "fact", 1)]
    }

    fn projections(&self) -> Vec<Box<dyn AnyProjection>> {
        vec![Box::new(MemoryProjection)]
    }

    fn migrations(&self) -> Vec<Migration> {
        vec![Migration::new(
            1,
            "Create memories table and FTS index",
            "CREATE TABLE IF NOT EXISTS mem_memories (
                id TEXT PRIMARY KEY,
                key TEXT NOT NULL,
                content TEXT NOT NULL DEFAULT '',
                conversation_id TEXT,
                created_at TEXT NOT NULL
            );",
        )]
    }

    fn expected_columns(&self) -> Vec<(&str, Vec<(String, String)>)> {
        vec![(
            "mem_memories",
            vec![
                ("id".into(), "TEXT PRIMARY KEY".into()),
                ("key".into(), "TEXT NOT NULL".into()),
                ("content".into(), "TEXT NOT NULL DEFAULT ''".into()),
                ("conversation_id".into(), "TEXT".into()),
                ("created_at".into(), "TEXT NOT NULL".into()),
            ],
        )]
    }

    fn fts_tables(&self) -> Vec<(&str, Vec<String>)> {
        vec![("mem_memories_fts", vec!["content".into(), "key".into()])]
    }
}

// ── Projection ───────────────────────────────────────────────────────

struct MemoryProjection;

impl AnyProjection for MemoryProjection {
    fn name(&self) -> &str {
        "memory_projection"
    }

    fn project_event(
        &self,
        tag: &str,
        attrs: &BTreeMap<String, String>,
        text: Option<&str>,
        event_id: &str,
        timestamp: i64,
    ) -> Vec<ProjectionOp> {
        let _ = event_id;
        let ts = chrono::DateTime::from_timestamp(timestamp, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default();

        match tag {
            t if t.ends_with(":FactCreated") => {
                let id = attrs.get("id").cloned().unwrap_or_default();
                let key = attrs.get("key").cloned().unwrap_or_default();
                let content = text.unwrap_or_default().to_string();
                let conv_id = attrs.get("conversation_id").cloned();
                vec![
                    ProjectionOp::sql(
                        "INSERT OR REPLACE INTO mem_memories (id, key, content, conversation_id, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                        vec![
                            SqlParam::Text(id.clone()),
                            SqlParam::Text(key.clone()),
                            SqlParam::Text(content.clone()),
                            conv_id.map_or(SqlParam::Null, SqlParam::Text),
                            SqlParam::Text(ts),
                        ],
                    ),
                    // Maintain FTS index (delete-then-insert for idempotency)
                    ProjectionOp::sql(
                        "DELETE FROM mem_memories_fts WHERE id = ?1",
                        vec![SqlParam::Text(id.clone())],
                    ),
                    ProjectionOp::sql(
                        "INSERT INTO mem_memories_fts (id, content, key) VALUES (?1, ?2, ?3)",
                        vec![
                            SqlParam::Text(id),
                            SqlParam::Text(content),
                            SqlParam::Text(key),
                        ],
                    ),
                ]
            }
            t if t.ends_with(":Forgotten") => {
                let key = attrs.get("key").cloned().unwrap_or_default();
                vec![
                    // Delete FTS rows before main table rows
                    ProjectionOp::sql(
                        "DELETE FROM mem_memories_fts WHERE id IN (SELECT id FROM mem_memories WHERE key = ?1)",
                        vec![SqlParam::Text(key.clone())],
                    ),
                    ProjectionOp::sql(
                        "DELETE FROM mem_memories WHERE key = ?1",
                        vec![SqlParam::Text(key)],
                    ),
                ]
            }
            _ => vec![],
        }
    }
}
