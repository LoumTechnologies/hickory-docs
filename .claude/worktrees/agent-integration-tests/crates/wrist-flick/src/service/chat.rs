use std::pin::Pin;
use std::sync::Arc;

use futures::Stream;
use hick_grove::GroveEngine;
use hick_grove::command::GroveCommand;
use hick_grove::namespace::DocId;

use crate::handlers::chat_handler::ChatSqliteStore;
use crate::handlers::memory_handler::MemorySqliteStore;
use crate::llm::{ChatChunk, Message, Role, StreamingLlmClient};
use crate::sse::ChatEvent;

pub struct ChatService {
    engine: Arc<GroveEngine>,
    chat_store: Arc<ChatSqliteStore>,
    memory_store: Arc<MemorySqliteStore>,
    llm: Arc<dyn StreamingLlmClient>,
    doc_id: DocId,
}

impl ChatService {
    pub fn new(
        engine: Arc<GroveEngine>,
        chat_store: Arc<ChatSqliteStore>,
        memory_store: Arc<MemorySqliteStore>,
        llm: Arc<dyn StreamingLlmClient>,
        doc_id: DocId,
    ) -> Self {
        Self {
            engine,
            chat_store,
            memory_store,
            llm,
            doc_id,
        }
    }

    /// Stream a response for a user message in a conversation.
    /// Returns a stream of ChatEvents to be sent as SSE.
    pub async fn stream_response(
        &self,
        conversation_id: &str,
        user_content: &str,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = ChatEvent> + Send>>> {
        // 1. Persist user message via grove
        let user_msg_id = format!("msg-{}", uuid::Uuid::now_v7());
        let user_xml = format!(
            r#"<msg xmlns:chat="https://wrist-flick.dev/ns/chat#">
              <chat:message id="{}" role="user" conversation="{}">
                {}
              </chat:message>
            </msg>"#,
            user_msg_id,
            conversation_id,
            escape_xml(user_content)
        );
        self.engine
            .process_command(GroveCommand::StructuredMessage {
                doc_id: self.doc_id.clone(),
                xml: user_xml,
            })
            .await?;

        // 2. Search memories for context
        let keywords = extract_keywords(user_content);
        let memories = if !keywords.is_empty() {
            self.memory_store.search(&keywords).unwrap_or_default()
        } else {
            vec![]
        };

        // 3. Build message history
        let recent = self.chat_store.list_recent_messages(conversation_id, 20)?;

        let mut llm_messages = Vec::new();

        // System prompt with memories
        let mut system = String::from(
            "You are a helpful personal AI assistant called Wrist-Flick. \
             You help the user manage tasks, remember important facts, and have conversations.\n\n\
             When you want to save a fact for later, include an XML block in your response:\n\
             <mem:fact key=\"short-key\">The fact content</mem:fact>\n\n\
             When you want to create a task, include:\n\
             <task:create description=\"Task description\"/>\n",
        );

        if !memories.is_empty() {
            system.push_str("\nRelevant memories:\n");
            for mem in &memories {
                system.push_str(&format!("- [{}]: {}\n", mem.key, mem.content));
            }
        }

        llm_messages.push(Message {
            role: Role::System,
            content: system,
        });

        for msg in &recent {
            let role = match msg.role.as_str() {
                "user" => Role::User,
                "assistant" => Role::Assistant,
                _ => continue,
            };
            llm_messages.push(Message {
                role,
                content: msg.content.clone(),
            });
        }

        // 4. Stream LLM response
        let assistant_msg_id = format!("msg-{}", uuid::Uuid::now_v7());
        let stream = self.llm.complete_stream(llm_messages).await?;

        let engine = self.engine.clone();
        let doc_id = self.doc_id.clone();
        let conv_id = conversation_id.to_string();
        let msg_id = assistant_msg_id.clone();

        let event_stream = make_event_stream(stream, msg_id, conv_id, doc_id, engine);

        Ok(Box::pin(event_stream))
    }
}

fn make_event_stream(
    stream: Pin<Box<dyn Stream<Item = anyhow::Result<ChatChunk>> + Send>>,
    msg_id: String,
    conv_id: String,
    doc_id: DocId,
    engine: Arc<GroveEngine>,
) -> impl Stream<Item = ChatEvent> + Send {
    async_stream::stream! {
        use futures::StreamExt as _;

        yield ChatEvent::MessageStart {
            message_id: msg_id.clone(),
            role: "assistant".into(),
        };

        let mut full_response = String::new();
        let mut finish_reason = "stop".to_string();

        let mut stream = stream;
        while let Some(result) = stream.next().await {
            match result {
                Ok(chunk) => {
                    if !chunk.delta.is_empty() {
                        full_response.push_str(&chunk.delta);
                        yield ChatEvent::ContentDelta {
                            delta: chunk.delta,
                        };
                    }
                    if let Some(reason) = chunk.finish_reason {
                        finish_reason = reason;
                    }
                }
                Err(e) => {
                    let err_msg = format!("{}", e);
                    yield ChatEvent::Error {
                        message: err_msg,
                    };
                    return;
                }
            }
        }

        // Parse structured actions from the response
        let actions = parse_structured_actions(&full_response);

        // Build grove command for assistant message + any side effects
        let mut xml_parts = vec![format!(
            r#"<chat:message id="{}" role="assistant" conversation="{}">
              {}
            </chat:message>"#,
            msg_id,
            conv_id,
            escape_xml(&full_response)
        )];

        let mut namespaces = vec![r#"xmlns:chat="https://wrist-flick.dev/ns/chat#""#.to_string()];

        for action in &actions {
            match action {
                StructuredAction::Memory { key, content } => {
                    let fact_id = format!("fact-{}", uuid::Uuid::now_v7());
                    if !namespaces.iter().any(|n| n.contains("memory#")) {
                        namespaces.push(r#"xmlns:mem="https://wrist-flick.dev/ns/memory#""#.to_string());
                    }
                    xml_parts.push(format!(
                        r#"<mem:fact id="{}" key="{}" conversation="{}">
                          {}
                        </mem:fact>"#,
                        fact_id,
                        escape_xml(key),
                        conv_id,
                        escape_xml(content)
                    ));
                    yield ChatEvent::MemoryCreated {
                        id: fact_id,
                        key: key.clone(),
                    };
                }
                StructuredAction::Task { description } => {
                    let task_id = format!("task-{}", uuid::Uuid::now_v7());
                    if !namespaces.iter().any(|n| n.contains("task#")) {
                        namespaces.push(r#"xmlns:task="https://wrist-flick.dev/ns/task#""#.to_string());
                    }
                    xml_parts.push(format!(
                        r#"<task:create id="{}" description="{}" conversation="{}"/>"#,
                        task_id,
                        escape_xml(description),
                        conv_id,
                    ));
                    yield ChatEvent::TaskCreated {
                        id: task_id,
                        description: description.clone(),
                    };
                }
            }
        }

        let xml = format!(
            "<msg {}>\n{}\n</msg>",
            namespaces.join(" "),
            xml_parts.join("\n")
        );

        if let Err(e) = engine
            .process_command(GroveCommand::StructuredMessage {
                doc_id,
                xml,
            })
            .await
        {
            log::error!("Failed to persist assistant message: {}", e);
        }

        yield ChatEvent::MessageEnd {
            message_id: msg_id,
            finish_reason,
        };
    }
}

#[derive(Debug)]
enum StructuredAction {
    Memory { key: String, content: String },
    Task { description: String },
}

fn parse_structured_actions(response: &str) -> Vec<StructuredAction> {
    let mut actions = Vec::new();

    // Parse <mem:fact key="...">content</mem:fact>
    let mut search = response;
    while let Some(start) = search.find("<mem:fact") {
        if let Some(end) = search[start..].find("</mem:fact>") {
            let tag_content = &search[start..start + end + 11];
            if let (Some(key), Some(content)) = (
                extract_attr(tag_content, "key"),
                extract_text_content(tag_content, "mem:fact"),
            ) {
                actions.push(StructuredAction::Memory { key, content });
            }
            search = &search[start + end + 11..];
        } else {
            break;
        }
    }

    // Parse <task:create description="..."/>
    search = response;
    while let Some(start) = search.find("<task:create") {
        if let Some(end) = search[start..].find("/>") {
            let tag_content = &search[start..start + end + 2];
            if let Some(desc) = extract_attr(tag_content, "description") {
                actions.push(StructuredAction::Task { description: desc });
            }
            search = &search[start + end + 2..];
        } else if let Some(end) = search[start..].find("</task:create>") {
            let tag_content = &search[start..start + end + 14];
            if let Some(desc) = extract_attr(tag_content, "description") {
                actions.push(StructuredAction::Task { description: desc });
            }
            search = &search[start + end + 14..];
        } else {
            break;
        }
    }

    actions
}

fn extract_attr(tag: &str, attr_name: &str) -> Option<String> {
    let pattern = format!(r#"{}=""#, attr_name);
    let start = tag.find(&pattern)?;
    let after = &tag[start + pattern.len()..];
    let end = after.find('"')?;
    Some(after[..end].to_string())
}

fn extract_text_content(tag: &str, tag_name: &str) -> Option<String> {
    let open_end = tag.find('>')?;
    let close_start = tag.rfind(&format!("</{}", tag_name))?;
    Some(tag[open_end + 1..close_start].trim().to_string())
}

fn extract_keywords(text: &str) -> String {
    // Simple keyword extraction: split on whitespace, filter short/common words
    let stop_words = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "can", "shall",
        "to", "of", "in", "for", "on", "with", "at", "by", "from", "as", "into", "about", "like",
        "through", "after", "over", "between", "out", "that", "this", "these", "those", "it",
        "its", "i", "me", "my", "we", "our", "you", "your", "he", "she", "they", "what", "which",
        "who", "when", "where", "why", "how", "not", "no", "but", "and", "or", "if", "then", "so",
        "up", "just",
    ];

    text.split_whitespace()
        .filter(|w| w.len() > 2 && !stop_words.contains(&w.to_lowercase().as_str()))
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" OR ")
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_memory_action() {
        let response =
            r#"Here is info. <mem:fact key="api-status">API is 60% done</mem:fact> More text."#;
        let actions = parse_structured_actions(response);
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            StructuredAction::Memory { key, content } => {
                assert_eq!(key, "api-status");
                assert_eq!(content, "API is 60% done");
            }
            _ => panic!("Expected Memory action"),
        }
    }

    #[test]
    fn test_parse_task_action() {
        let response = r#"Done. <task:create description="Follow up on migration"/>"#;
        let actions = parse_structured_actions(response);
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            StructuredAction::Task { description } => {
                assert_eq!(description, "Follow up on migration");
            }
            _ => panic!("Expected Task action"),
        }
    }

    #[test]
    fn test_parse_multiple_actions() {
        let response = r#"Info. <mem:fact key="k1">fact1</mem:fact> <task:create description="t1"/> <mem:fact key="k2">fact2</mem:fact>"#;
        let actions = parse_structured_actions(response);
        assert_eq!(actions.len(), 3);
    }

    #[test]
    fn test_extract_keywords() {
        let text = "What is the status of the API migration project?";
        let keywords = extract_keywords(text);
        assert!(keywords.contains("status"));
        assert!(keywords.contains("API"));
        assert!(keywords.contains("migration"));
        assert!(!keywords.contains("the"));
    }
}
