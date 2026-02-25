use std::sync::Arc;

use hick_grove::GroveEngine;
use hick_grove::command::GroveCommand;
use hick_grove::namespace::DocId;

use crate::handlers::task_handler::{WfTaskRow, WfTaskSqliteStore};

pub struct TaskService {
    engine: Arc<GroveEngine>,
    store: Arc<WfTaskSqliteStore>,
    doc_id: DocId,
}

impl TaskService {
    pub fn new(engine: Arc<GroveEngine>, store: Arc<WfTaskSqliteStore>, doc_id: DocId) -> Self {
        Self {
            engine,
            store,
            doc_id,
        }
    }

    pub fn list(&self) -> anyhow::Result<Vec<WfTaskRow>> {
        self.store.list_tasks(self.doc_id.as_str())
    }

    pub async fn create(&self, description: &str) -> anyhow::Result<String> {
        let id = format!("task-{}", uuid::Uuid::now_v7());
        let xml = format!(
            r#"<msg xmlns:task="https://wrist-flick.dev/ns/task#">
              <task:create id="{}" description="{}"/>
            </msg>"#,
            id,
            escape_xml_attr(description)
        );
        self.engine
            .process_command(GroveCommand::StructuredMessage {
                doc_id: self.doc_id.clone(),
                xml,
            })
            .await?;
        Ok(id)
    }

    pub async fn update(
        &self,
        id: &str,
        description: Option<&str>,
        status: Option<&str>,
        due_date: Option<&str>,
    ) -> anyhow::Result<()> {
        let mut attrs = format!(r#"ref="{}""#, id);
        if let Some(d) = description {
            attrs.push_str(&format!(r#" description="{}""#, escape_xml_attr(d)));
        }
        if let Some(s) = status {
            attrs.push_str(&format!(r#" status="{}""#, escape_xml_attr(s)));
        }
        if let Some(dd) = due_date {
            attrs.push_str(&format!(r#" due_date="{}""#, escape_xml_attr(dd)));
        }
        let xml = format!(
            r#"<msg xmlns:task="https://wrist-flick.dev/ns/task#">
              <task:update {}/>
            </msg>"#,
            attrs
        );
        self.engine
            .process_command(GroveCommand::StructuredMessage {
                doc_id: self.doc_id.clone(),
                xml,
            })
            .await?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let xml = format!(
            r#"<msg xmlns:task="https://wrist-flick.dev/ns/task#">
              <task:delete ref="{}"/>
            </msg>"#,
            id
        );
        self.engine
            .process_command(GroveCommand::StructuredMessage {
                doc_id: self.doc_id.clone(),
                xml,
            })
            .await?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> anyhow::Result<Option<WfTaskRow>> {
        self.store.get_task(id)
    }
}

fn escape_xml_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
