use std::sync::Arc;

use hick_grove::GroveEngine;
use hick_grove::command::GroveCommand;
use hick_grove::namespace::DocId;

use crate::handlers::memory_handler::{MemoryRow, MemorySearchResult, MemorySqliteStore};

pub struct MemoryService {
    engine: Arc<GroveEngine>,
    store: Arc<MemorySqliteStore>,
    doc_id: DocId,
}

impl MemoryService {
    pub fn new(engine: Arc<GroveEngine>, store: Arc<MemorySqliteStore>, doc_id: DocId) -> Self {
        Self {
            engine,
            store,
            doc_id,
        }
    }

    pub fn list(&self) -> anyhow::Result<Vec<MemoryRow>> {
        self.store.list_memories(self.doc_id.as_str())
    }

    pub async fn create(
        &self,
        key: &str,
        content: &str,
        conversation_id: Option<&str>,
    ) -> anyhow::Result<String> {
        let id = format!("fact-{}", uuid::Uuid::now_v7());
        let conv_attr = conversation_id
            .map(|c| format!(r#" conversation="{}""#, c))
            .unwrap_or_default();
        let xml = format!(
            r#"<msg xmlns:mem="https://wrist-flick.dev/ns/memory#">
              <mem:fact id="{}" key="{}"{}>
                {}
              </mem:fact>
            </msg>"#,
            id,
            escape_xml(key),
            conv_attr,
            escape_xml(content)
        );
        self.engine
            .process_command(GroveCommand::StructuredMessage {
                doc_id: self.doc_id.clone(),
                xml,
            })
            .await?;
        Ok(id)
    }

    pub async fn delete(&self, id: &str) -> anyhow::Result<()> {
        self.store.delete_memory(id)
    }

    pub fn search(&self, query: &str) -> anyhow::Result<Vec<MemorySearchResult>> {
        self.store.search(query)
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
