//! Chat sub-app — conversations and messages.
//!
//! Event streams:
//! - `ChatConversationCreated` — new conversation started
//! - `ChatMessageCreated` — message added to conversation

use std::collections::BTreeMap;

use waymark_core::{
    AnyProjection, CollectionDef, CommandDef, EventStreamDef, Migration, NamespaceUri, ProjectionOp,
    SqlParam, SubApp,
};

// ── Sub-app ──────────────────────────────────────────────────────────

pub struct ChatSubApp {
    namespace: NamespaceUri,
}

impl ChatSubApp {
    pub fn new() -> Self {
        Self {
            namespace: NamespaceUri::new("https://drift-signal.dev/ns/chat#"),
        }
    }
}

impl Default for ChatSubApp {
    fn default() -> Self {
        Self::new()
    }
}

impl SubApp for ChatSubApp {
    fn name(&self) -> &str {
        "chat"
    }

    fn namespace(&self) -> &NamespaceUri {
        &self.namespace
    }

    fn ns_prefix(&self) -> &str {
        "chat"
    }

    fn event_streams(&self) -> Vec<EventStreamDef> {
        vec![EventStreamDef {
            name: "chat_events".into(),
            doc_id: "grove:chat:events".into(),
            description: "Chat conversation and message events".into(),
        }]
    }

    fn commands(&self) -> Vec<CommandDef> {
        vec![
            CommandDef::new(
                "CreateConversation",
                "chat:ConversationCreated",
                "Start a new chat conversation",
            )
            .with_required(vec!["id".into(), "title".into()]),
            CommandDef::new(
                "CreateMessage",
                "chat:MessageCreated",
                "Add a message to a conversation",
            )
            .with_required(vec![
                "id".into(),
                "conversation_id".into(),
                "role".into(),
            ]),
        ]
    }

    fn collections(&self) -> Vec<CollectionDef> {
        vec![
            CollectionDef::new("conversations", "conv", 1),
            CollectionDef::new("messages", "msg", 1),
        ]
    }

    fn projections(&self) -> Vec<Box<dyn AnyProjection>> {
        vec![Box::new(ChatProjection)]
    }

    fn migrations(&self) -> Vec<Migration> {
        vec![Migration::new(
            1,
            "Create conversations and messages tables",
            "CREATE TABLE IF NOT EXISTS chat_conversations (
                id TEXT PRIMARY KEY,
                title TEXT DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS chat_messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_chat_messages_conv
                ON chat_messages(conversation_id, created_at);",
        )]
    }

    fn expected_columns(&self) -> Vec<(&str, Vec<(String, String)>)> {
        vec![
            (
                "chat_conversations",
                vec![
                    ("id".into(), "TEXT PRIMARY KEY".into()),
                    ("title".into(), "TEXT DEFAULT ''".into()),
                    ("created_at".into(), "TEXT NOT NULL".into()),
                    ("updated_at".into(), "TEXT NOT NULL".into()),
                ],
            ),
            (
                "chat_messages",
                vec![
                    ("id".into(), "TEXT PRIMARY KEY".into()),
                    ("conversation_id".into(), "TEXT NOT NULL".into()),
                    ("role".into(), "TEXT NOT NULL".into()),
                    ("content".into(), "TEXT NOT NULL DEFAULT ''".into()),
                    ("created_at".into(), "TEXT NOT NULL".into()),
                ],
            ),
        ]
    }
}

// ── Projection ───────────────────────────────────────────────────────

struct ChatProjection;

impl AnyProjection for ChatProjection {
    fn name(&self) -> &str {
        "chat_projection"
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
            t if t.ends_with(":ConversationCreated") => {
                let id = attrs.get("id").cloned().unwrap_or_default();
                let title = attrs.get("title").cloned().unwrap_or_default();
                vec![ProjectionOp::sql(
                    "INSERT OR REPLACE INTO chat_conversations (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
                    vec![
                        SqlParam::Text(id),
                        SqlParam::Text(title),
                        SqlParam::Text(ts.clone()),
                        SqlParam::Text(ts),
                    ],
                )]
            }
            t if t.ends_with(":MessageCreated") => {
                let id = attrs.get("id").cloned().unwrap_or_default();
                let conversation_id = attrs.get("conversation_id").cloned().unwrap_or_default();
                let role = attrs.get("role").cloned().unwrap_or_default();
                let content = text.unwrap_or_default().to_string();
                vec![
                    ProjectionOp::sql(
                        "INSERT OR REPLACE INTO chat_messages (id, conversation_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                        vec![
                            SqlParam::Text(id),
                            SqlParam::Text(conversation_id.clone()),
                            SqlParam::Text(role),
                            SqlParam::Text(content),
                            SqlParam::Text(ts.clone()),
                        ],
                    ),
                    ProjectionOp::sql(
                        "UPDATE chat_conversations SET updated_at = ?1 WHERE id = ?2",
                        vec![
                            SqlParam::Text(ts),
                            SqlParam::Text(conversation_id),
                        ],
                    ),
                ]
            }
            _ => vec![],
        }
    }
}
