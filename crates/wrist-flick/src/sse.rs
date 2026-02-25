use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatEvent {
    MessageStart {
        message_id: String,
        role: String,
    },
    ContentDelta {
        delta: String,
    },
    MemoryCreated {
        id: String,
        key: String,
    },
    TaskCreated {
        id: String,
        description: String,
    },
    MessageEnd {
        message_id: String,
        finish_reason: String,
    },
    Error {
        message: String,
    },
}

impl ChatEvent {
    pub fn event_name(&self) -> &'static str {
        match self {
            ChatEvent::MessageStart { .. } => "message_start",
            ChatEvent::ContentDelta { .. } => "content_delta",
            ChatEvent::MemoryCreated { .. } => "memory_created",
            ChatEvent::TaskCreated { .. } => "task_created",
            ChatEvent::MessageEnd { .. } => "message_end",
            ChatEvent::Error { .. } => "error",
        }
    }
}
