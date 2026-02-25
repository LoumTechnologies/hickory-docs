use std::sync::Arc;

use hick_grove::GroveEngine;
use hick_grove::namespace::DocId;

use crate::handlers::chat_handler::ChatSqliteStore;
use crate::handlers::memory_handler::MemorySqliteStore;
use crate::handlers::task_handler::WfTaskSqliteStore;
use crate::llm::StreamingLlmClient;

#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<GroveEngine>,
    pub chat_store: Arc<ChatSqliteStore>,
    pub memory_store: Arc<MemorySqliteStore>,
    pub task_store: Arc<WfTaskSqliteStore>,
    pub llm: Arc<dyn StreamingLlmClient>,
    pub doc_id: DocId,
}
